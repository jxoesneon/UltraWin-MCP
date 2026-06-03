use crate::traits::BrowserProvider;
use crate::vision::browser;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::debug;

pub struct CdpBridge {
    host: String,
    port: u16,
}

impl CdpBridge {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }
}

#[async_trait]
impl BrowserProvider for CdpBridge {
    async fn ensure_ready(&self) -> Result<()> {
        browser::ensure_browser_ready().await
    }

    async fn query_selector(&self, selector: &str) -> Result<Value> {
        let version_url = format!("http://{}:{}/json/version", self.host, self.port);
        let resp = reqwest::get(&version_url).await?;
        let info: Value = resp.json().await?;
        let ws_url = info["webSocketDebuggerUrl"]
            .as_str()
            .ok_or_else(|| anyhow!("No webSocketDebuggerUrl found"))?;

        let (mut ws_stream, _) = connect_async(ws_url).await?;
        debug!("Connected to CDP at {}", ws_url);

        let doc_req = json!({
            "id": 1,
            "method": "DOM.getDocument",
            "params": { "depth": -1 }
        });
        ws_stream
            .send(Message::Text(doc_req.to_string().into()))
            .await?;

        let query_req = json!({
            "id": 2,
            "method": "DOM.querySelector",
            "params": {
                "nodeId": 1,
                "selector": selector
            }
        });
        ws_stream
            .send(Message::Text(query_req.to_string().into()))
            .await?;

        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let res: Value = serde_json::from_str(&text)?;
                if res["id"] == 2 {
                    if let Some(node_id) = res["result"]["nodeId"].as_i64() {
                        let box_req = json!({
                            "id": 3,
                            "method": "DOM.getBoxModel",
                            "params": { "nodeId": node_id }
                        });
                        ws_stream
                            .send(Message::Text(box_req.to_string().into()))
                            .await?;
                    }
                }
                if res["id"] == 3 {
                    return Ok(res["result"]["model"].clone());
                }
            }
        }

        Err(anyhow!("CDP Query failed or timed out"))
    }
}
