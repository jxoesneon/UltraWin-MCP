use async_trait::async_trait;
use futures_util::stream::{self, Stream};
use mcp_sdk_rs::error::Error;
use mcp_sdk_rs::transport::{Message, Transport};
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{
    self, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};
use tokio::sync::mpsc;
use tracing::error;

pub struct StdioTransport {
    read_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Result<Message, Error>>>>,
    write_tx: mpsc::Sender<Message>,
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new(io::stdin(), io::stdout())
    }
}

impl StdioTransport {
    pub fn new<R, W>(reader: R, writer: W) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (read_tx, read_rx) = mpsc::channel(100);
        let (write_tx, mut write_rx) = mpsc::channel(100);

        // Stdin reader task supporting both:
        // 1. Standard MCP newline-delimited JSON-RPC
        // 2. LSP Content-Length framed JSON-RPC
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {}
                    Err(e) => {
                        error!("Stdio Transport: Failed to read from stdin: {}", e);
                        break;
                    }
                }

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let body_str = if trimmed.starts_with("Content-Length:") {
                    let len_str = trimmed.trim_start_matches("Content-Length:").trim();
                    let Ok(len) = len_str.parse::<usize>() else {
                        continue;
                    };

                    // Consume separator line (e.g. \r\n)
                    let mut separator = String::new();
                    let _ = reader.read_line(&mut separator).await;

                    let mut body = vec![0u8; len];
                    if let Err(e) = reader.read_exact(&mut body).await {
                        error!("Stdio Transport: Failed to read framed body: {}", e);
                        break;
                    }
                    match String::from_utf8(body) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Stdio Transport: Non-UTF8 body: {}", e);
                            continue;
                        }
                    }
                } else {
                    trimmed.to_string()
                };

                // Normalize incoming messages:
                // 1. Standard MCP client sends "params.clientInfo" instead of "params.implementation"
                // 2. Standard MCP client sends "notifications/initialized" instead of "initialized"
                #[allow(clippy::collapsible_if)]
                let normalized_val = match serde_json::from_str::<serde_json::Value>(&body_str) {
                    Ok(mut val) => {
                        if let Some(method_val) = val.get_mut("method") {
                            if method_val.as_str() == Some("notifications/initialized") {
                                *method_val = serde_json::json!("initialized");
                            }
                        }

                        if val.get("method").and_then(|m| m.as_str()) == Some("initialize") {
                            if let Some(params) =
                                val.get_mut("params").and_then(|p| p.as_object_mut())
                            {
                                if !params.contains_key("implementation") {
                                    if let Some(client_info) = params.get("clientInfo").cloned() {
                                        params.insert("implementation".to_string(), client_info);
                                    } else {
                                        params.insert(
                                            "implementation".to_string(),
                                            serde_json::json!({
                                                "name": "unknown-client",
                                                "version": "1.0.0"
                                            }),
                                        );
                                    }
                                }
                            }
                        }
                        val
                    }
                    Err(e) => {
                        error!("Stdio Transport: JSON parse error: {}", e);
                        let _ = read_tx.send(Err(Error::Serialization(e.to_string()))).await;
                        continue;
                    }
                };

                match serde_json::from_value::<Message>(normalized_val) {
                    Ok(msg) => {
                        let _ = read_tx.send(Ok(msg)).await;
                    }
                    Err(e) => {
                        error!("Stdio Transport: Message deserialization error: {}", e);
                        let _ = read_tx.send(Err(Error::Serialization(e.to_string()))).await;
                    }
                }
            }
        });

        // Stdout writer task: output newline-delimited JSON-RPC as required by MCP stdio
        tokio::spawn(async move {
            let mut writer = writer;
            while let Some(msg) = write_rx.recv().await {
                let mut msg_val = match serde_json::to_value(&msg) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Stdio Transport: Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if let Some(result) = msg_val.get_mut("result").and_then(|r| r.as_object_mut()) {
                    if !result.contains_key("protocolVersion") {
                        result.insert(
                            "protocolVersion".to_string(),
                            serde_json::json!("2024-11-05"),
                        );
                    }
                    if !result.contains_key("serverInfo") {
                        result.insert(
                            "serverInfo".to_string(),
                            serde_json::json!({
                                "name": "ultrawin-mcp",
                                "version": env!("CARGO_PKG_VERSION")
                            }),
                        );
                    }
                    if !result.contains_key("capabilities") {
                        result.insert(
                            "capabilities".to_string(),
                            serde_json::json!({
                                "tools": {}
                            }),
                        );
                    }
                }

                if let Ok(body) = serde_json::to_string(&msg_val) {
                    let line = format!("{}\n", body);
                    if let Err(e) = writer.write_all(line.as_bytes()).await {
                        error!("Stdio Transport: Failed to write to stdout: {}", e);
                        break;
                    }
                    let _ = writer.flush().await;
                }
            }
        });

        Self {
            read_rx: Arc::new(tokio::sync::Mutex::new(read_rx)),
            write_tx,
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&self, message: Message) -> Result<(), Error> {
        self.write_tx
            .send(message)
            .await
            .map_err(|e| Error::Io(e.to_string()))
    }

    fn receive(&self) -> Pin<Box<dyn Stream<Item = Result<Message, Error>> + Send + 'static>> {
        let rx = self.read_rx.clone();
        Box::pin(stream::poll_fn(move |cx| {
            let mut rx = match rx.try_lock() {
                Ok(guard) => guard,
                Err(_) => return std::task::Poll::Pending,
            };
            rx.poll_recv(cx)
        }))
    }

    async fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_sdk_rs::protocol::{RequestId, Response};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_stdio_newline_jsonrpc() {
        let (client_write, server_read) = tokio::io::duplex(4096);
        let (server_write, mut client_read) = tokio::io::duplex(4096);

        let transport = StdioTransport::new(server_read, server_write);
        use futures_util::StreamExt;
        let mut rx = transport.receive();

        let mut client_writer = client_write;
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        });
        client_writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        client_writer.flush().await.unwrap();

        let msg = rx.next().await.unwrap().unwrap();
        match msg {
            Message::Request(r) => {
                assert_eq!(r.method, "ping");
                assert_eq!(r.id, RequestId::Number(1));
            }
            _ => panic!("Expected Request"),
        }

        let resp = Response::success(
            RequestId::Number(1),
            Some(serde_json::json!({"pong": true})),
        );
        transport.send(Message::Response(resp)).await.unwrap();

        let mut line = String::new();
        let mut client_reader = BufReader::new(&mut client_read);
        client_reader.read_line(&mut line).await.unwrap();
        let resp_val: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp_val["result"]["pong"], true);
        assert_eq!(resp_val["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(resp_val["result"]["serverInfo"]["name"], "ultrawin-mcp");
    }

    #[tokio::test]
    async fn test_stdio_content_length_and_normalizations() {
        let (client_write, server_read) = tokio::io::duplex(4096);
        let (server_write, _client_read) = tokio::io::duplex(4096);

        let transport = StdioTransport::new(server_read, server_write);
        use futures_util::StreamExt;
        let mut rx = transport.receive();

        let mut client_writer = client_write;

        // 1. Content-Length framed initialize with clientInfo
        let init_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                },
                "capabilities": {}
            }
        })
        .to_string();

        let framed = format!("Content-Length: {}\r\n\r\n{}", init_body.len(), init_body);
        client_writer.write_all(framed.as_bytes()).await.unwrap();

        // 2. Content-Length framed notifications/initialized
        let notif_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        })
        .to_string();
        let framed_notif = format!("Content-Length: {}\r\n\r\n{}", notif_body.len(), notif_body);
        client_writer
            .write_all(framed_notif.as_bytes())
            .await
            .unwrap();

        // 3. Newline initialize without clientInfo fallback
        let init_body_no_info = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 101,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }
        })
        .to_string();
        client_writer
            .write_all(format!("\n{}\n", init_body_no_info).as_bytes())
            .await
            .unwrap();

        // 4. Invalid JSON
        client_writer
            .write_all(b"not a valid json\n")
            .await
            .unwrap();
        client_writer.flush().await.unwrap();

        // Check msg 1: initialize
        let msg1 = rx.next().await.unwrap().unwrap();
        match msg1 {
            Message::Request(r) => {
                assert_eq!(r.method, "initialize");
                assert_eq!(r.id, RequestId::Number(100));
            }
            _ => panic!("Expected Request"),
        }

        // Check msg 2: initialized
        let msg2 = rx.next().await.unwrap().unwrap();
        match msg2 {
            Message::Notification(n) => {
                assert_eq!(n.method, "initialized");
            }
            _ => panic!("Expected Notification"),
        }

        // Check msg 3: initialize fallback
        let msg3 = rx.next().await.unwrap().unwrap();
        match msg3 {
            Message::Request(r) => {
                assert_eq!(r.method, "initialize");
                assert_eq!(r.id, RequestId::Number(101));
            }
            _ => panic!("Expected Request"),
        }

        // Check msg 4: error serialization
        let msg4 = rx.next().await.unwrap();
        assert!(msg4.is_err());

        // Close transport
        assert!(transport.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_default() {
        let _ = StdioTransport::default();
    }
}
