use mcp_sdk_rs::transport::{Transport, Message};
use mcp_sdk_rs::Error;
use async_trait::async_trait;
use futures_util::stream::{self, Stream};
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, AsyncRead, AsyncWrite};
use tokio::sync::mpsc;
use tracing::{error};

pub struct LspTransport {
    read_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Result<Message, Error>>>>,
    write_tx: mpsc::Sender<Message>,
}

impl Default for LspTransport {
    fn default() -> Self {
        Self::new(io::stdin(), io::stdout())
    }
}

impl LspTransport {
    pub fn new<R, W>(reader: R, writer: W) -> Self 
    where 
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (read_tx, read_rx) = mpsc::channel(100);
        let (write_tx, mut write_rx) = mpsc::channel(100);

        // Stdin reader task
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            loop {
                let mut line = String::new();
                if let Err(e) = reader.read_line(&mut line).await {
                    error!("LSP Transport: Failed to read header: {}", e);
                    break;
                }
                if line.is_empty() { break; }

                let trimmed = line.trim();
                if trimmed.starts_with("Content-Length: ") {
                    let Ok(len) = trimmed.trim_start_matches("Content-Length: ").parse::<usize>() else {
                        continue;
                    };

                    let mut separator = String::new();
                    let _ = reader.read_line(&mut separator).await;

                    let mut body = vec![0u8; len];
                    if let Err(e) = reader.read_exact(&mut body).await {
                        error!("LSP Transport: Failed to read body: {}", e);
                        break;
                    }

                    match serde_json::from_slice::<Message>(&body) {
                        Ok(msg) => {
                            let _ = read_tx.send(Ok(msg)).await;
                        }
                        Err(e) => {
                            error!("LSP Transport: Serialization error: {}", e);
                            let _ = read_tx.send(Err(Error::Serialization(e.to_string()))).await;
                        }
                    }
                }
            }
        });

        // Stdout writer task
        tokio::spawn(async move {
            let mut writer = writer;
            while let Some(msg) = write_rx.recv().await {
                if let Ok(body) = serde_json::to_string(&msg) {
                    let header = format!("Content-Length: {}\r\n\r\n", body.len());
                    if let Err(e) = writer.write_all(header.as_bytes()).await {
                        error!("LSP Transport: Failed to write header: {}", e);
                        break;
                    }
                    if let Err(e) = writer.write_all(body.as_bytes()).await {
                        error!("LSP Transport: Failed to write body: {}", e);
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
impl Transport for LspTransport {
    async fn send(&self, message: Message) -> Result<(), Error> {
        self.write_tx.send(message).await.map_err(|e| Error::Io(e.to_string()))
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
    use tokio::io::duplex;
    use serde_json::json;
    use futures_util::StreamExt;
    use mcp_sdk_rs::protocol::RequestId;

    #[tokio::test]
    async fn test_lsp_roundtrip() {
        let (client_read, server_write) = duplex(1024);
        let (server_read, client_write) = duplex(1024);

        let transport = LspTransport::new(server_read, server_write);
        
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "test",
            "params": {}
        });
        let body = serde_json::to_string(&msg).unwrap();
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        
        let mut client_write = client_write;
        client_write.write_all(header.as_bytes()).await.unwrap();
        client_write.write_all(body.as_bytes()).await.unwrap();
        client_write.flush().await.unwrap();

        let mut stream = transport.receive();
        let received = stream.next().await.unwrap().unwrap();
        
        if let Message::Request(req) = received {
            assert_eq!(req.method, "test");
        } else {
            panic!("Expected request");
        }

        let resp = Message::Response(mcp_sdk_rs::protocol::Response::success(RequestId::Number(1), Some(json!({"ok": true}))));
        transport.send(resp).await.unwrap();

        let mut client_read = BufReader::new(client_read);
        let mut line = String::new();
        client_read.read_line(&mut line).await.unwrap();
        assert!(line.contains("Content-Length:"));
        let len: usize = line.trim_start_matches("Content-Length: ").trim().parse().unwrap();
        
        client_read.read_line(&mut line).await.unwrap(); // empty line
        
        let mut body_buf = vec![0u8; len];
        client_read.read_exact(&mut body_buf).await.unwrap();
        let resp_json: serde_json::Value = serde_json::from_slice(&body_buf).unwrap();
        assert_eq!(resp_json["result"]["ok"], true);
    }
}
