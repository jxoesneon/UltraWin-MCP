use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use image::ImageFormat;
use mcp_sdk_rs::error::{Error, ErrorCode};
use mcp_sdk_rs::server::{Server, ServerHandler};
use mcp_sdk_rs::types::{ClientCapabilities, Implementation, ServerCapabilities};
use serde_json::{Value, json};
use std::io::Cursor;
use std::sync::Arc;

use crate::traits::*;

pub mod lsp_transport;

pub struct UltraWinHandler {
    pub capture_engine: Option<Arc<dyn CaptureProvider>>,
    pub uia_bridge: Option<Arc<dyn UIAutomationProvider>>,
    pub input_manager: Arc<dyn InputProvider>,
    pub vision_engine: Option<Arc<dyn VisionProvider>>,
    pub cdp_bridge: Arc<dyn BrowserProvider>,
}

#[async_trait]
impl ServerHandler for UltraWinHandler {
    async fn initialize(
        &self,
        _implementation: Implementation,
        _capabilities: ClientCapabilities,
    ) -> Result<ServerCapabilities, Error> {
        Ok(ServerCapabilities {
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_method(&self, method: &str, params: Option<Value>) -> Result<Value, Error> {
        match method {
            "tools/list" => Ok(json!({
                "tools": [
                    {
                        "name": "screenshot",
                        "description": "Capture a zero-latency screenshot using DXGI",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "get_ui_tree",
                        "description": "Retrieve the accessibility tree (recursive)",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "get_focused_element",
                        "description": "Get properties of the currently focused UI element",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "find_element",
                        "description": "Find an element by name or ID and return its bounding box",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string", "description": "Name or Automation ID" }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "click",
                        "description": "Click at specific screen coordinates",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "x": { "type": "integer" },
                                "y": { "type": "integer" },
                                "button": { "type": "string", "enum": ["left", "right", "middle"] }
                            },
                            "required": ["x", "y"]
                        }
                    },
                    {
                        "name": "type_text",
                        "description": "Type text into the focused element",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string" }
                            },
                            "required": ["text"]
                        }
                    },
                    {
                        "name": "read_text",
                        "description": "Perform hardware-accelerated OCR on the current screen",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "web_query",
                        "description": "Query the browser DOM using a CSS selector (requires --remote-debugging-port=9222)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "selector": { "type": "string", "description": "CSS Selector" }
                            },
                            "required": ["selector"]
                        }
                    }
                ]
            })),
            "tools/call" => {
                let params = params.ok_or_else(|| {
                    Error::protocol(ErrorCode::InvalidParams, "Missing parameters")
                })?;
                let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::protocol(ErrorCode::InvalidParams, "Missing tool name")
                })?;
                let default_args = json!({});
                let arguments = params.get("arguments").unwrap_or(&default_args);

                match name {
                    "screenshot" => {
                        let engine = self.capture_engine.as_ref().ok_or_else(|| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                "Capture engine not available",
                            )
                        })?;
                        let frame = engine.capture_frame().map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Capture failed: {}", e),
                            )
                        })?;

                        let mut buffer = Vec::new();
                        frame
                            .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
                            .map_err(|e| {
                                Error::protocol(
                                    ErrorCode::InternalError,
                                    format!("Encoding failed: {}", e),
                                )
                            })?;

                        let b64 = general_purpose::STANDARD.encode(buffer);

                        Ok(json!({
                            "content": [
                                { "type": "text", "text": "Screenshot captured successfully." },
                                { "type": "image", "data": b64, "mimeType": "image/png" }
                            ]
                        }))
                    }
                    "click" => {
                        let x =
                            arguments.get("x").and_then(|v| v.as_i64()).ok_or_else(|| {
                                Error::protocol(ErrorCode::InvalidParams, "Missing x")
                            })? as i32;
                        let y =
                            arguments.get("y").and_then(|v| v.as_i64()).ok_or_else(|| {
                                Error::protocol(ErrorCode::InvalidParams, "Missing y")
                            })? as i32;
                        let button = arguments
                            .get("button")
                            .and_then(|v| v.as_str())
                            .unwrap_or("left");

                        self.input_manager.mouse_click(x, y, button).map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Click failed: {}", e),
                            )
                        })?;

                        Ok(json!({
                            "content": [{ "type": "text", "text": format!("Clicked {} at ({}, {})", button, x, y) }]
                        }))
                    }
                    "type_text" => {
                        let text =
                            arguments
                                .get("text")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    Error::protocol(ErrorCode::InvalidParams, "Missing text")
                                })?;
                        self.input_manager.type_text(text).map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Typing failed: {}", e),
                            )
                        })?;
                        Ok(
                            json!({ "content": [{ "type": "text", "text": format!("Typed: {}", text) }] }),
                        )
                    }
                    "get_ui_tree" => {
                        let uia = self.uia_bridge.as_ref().ok_or_else(|| {
                            Error::protocol(ErrorCode::InternalError, "UIA bridge not available")
                        })?;
                        let tree = uia.get_root_json().map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Failed to get tree: {}", e),
                            )
                        })?;
                        Ok(
                            json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&tree).unwrap_or_default() }] }),
                        )
                    }
                    "get_focused_element" => {
                        let uia = self.uia_bridge.as_ref().ok_or_else(|| {
                            Error::protocol(ErrorCode::InternalError, "UIA bridge not available")
                        })?;
                        let props = uia.get_focused_json().map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Failed to get focus: {}", e),
                            )
                        })?;
                        Ok(
                            json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&props).unwrap_or_default() }] }),
                        )
                    }
                    "find_element" => {
                        let query =
                            arguments
                                .get("query")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    Error::protocol(ErrorCode::InvalidParams, "Missing query")
                                })?;
                        let uia = self.uia_bridge.as_ref().ok_or_else(|| {
                            Error::protocol(ErrorCode::InternalError, "UIA bridge not available")
                        })?;
                        let rect = uia.find_element(query).map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Search failed: {}", e),
                            )
                        })?;
                        match rect {
                            Some(r) => Ok(
                                json!({ "content": [{ "type": "text", "text": format!("Found at {{ left: {}, top: {}, right: {}, bottom: {} }}", r.left, r.top, r.right, r.bottom) }] }),
                            ),
                            None => Ok(
                                json!({ "content": [{ "type": "text", "text": "Element not found" }] }),
                            ),
                        }
                    }
                    "read_text" => {
                        let vision = self.vision_engine.as_ref().ok_or_else(|| {
                            Error::protocol(ErrorCode::InternalError, "Vision engine not available")
                        })?;
                        let engine = self.capture_engine.as_ref().ok_or_else(|| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                "Capture engine not available",
                            )
                        })?;
                        let frame = engine.capture_frame().map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Capture failed: {}", e),
                            )
                        })?;
                        let words = vision.recognize_text(&frame).await.map_err(|e| {
                            Error::protocol(ErrorCode::InternalError, format!("OCR failed: {}", e))
                        })?;
                        Ok(
                            json!({ "content": [{ "type": "text", "text": format!("Detected words: {:?}", words) }] }),
                        )
                    }
                    "web_query" => {
                        let selector = arguments
                            .get("selector")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                Error::protocol(ErrorCode::InvalidParams, "Missing selector")
                            })?;
                        self.cdp_bridge.ensure_ready().await.map_err(|e| {
                            Error::protocol(
                                ErrorCode::InternalError,
                                format!("Browser launch failed: {}", e),
                            )
                        })?;
                        let model =
                            self.cdp_bridge
                                .query_selector(selector)
                                .await
                                .map_err(|e| {
                                    Error::protocol(
                                        ErrorCode::InternalError,
                                        format!("CDP failed: {}", e),
                                    )
                                })?;
                        Ok(
                            json!({ "content": [{ "type": "text", "text": format!("Web element model: {:?}", model) }] }),
                        )
                    }
                    _ => Err(Error::protocol(
                        ErrorCode::MethodNotFound,
                        format!("Unknown tool: {}", name),
                    )),
                }
            }
            _ => Err(Error::protocol(
                ErrorCode::MethodNotFound,
                format!("Unknown method: {}", method),
            )),
        }
    }
}

pub fn build_server(
    transport: Arc<dyn mcp_sdk_rs::transport::Transport>,
    capture: Option<Arc<dyn CaptureProvider>>,
    uia: Option<Arc<dyn UIAutomationProvider>>,
    vision: Option<Arc<dyn VisionProvider>>,
    input: Arc<dyn InputProvider>,
    cdp: Arc<dyn BrowserProvider>,
) -> Server {
    let handler = Arc::new(UltraWinHandler {
        capture_engine: capture,
        uia_bridge: uia,
        input_manager: input,
        vision_engine: vision,
        cdp_bridge: cdp,
    });
    Server::new(transport, handler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vision::engine::DetectedWord;
    use image::DynamicImage;
    use windows::Win32::Foundation::RECT;

    pub struct MockCapture;
    impl CaptureProvider for MockCapture {
        fn capture_frame(&self) -> anyhow::Result<DynamicImage> {
            Ok(DynamicImage::new_rgba8(1, 1))
        }
    }

    pub struct MockUIA;
    impl UIAutomationProvider for MockUIA {
        fn get_root_json(&self) -> anyhow::Result<Value> {
            Ok(json!({"name": "Root"}))
        }
        fn get_focused_json(&self) -> anyhow::Result<Value> {
            Ok(json!({"name": "Focus"}))
        }
        fn find_element(&self, _query: &str) -> anyhow::Result<Option<RECT>> {
            Ok(Some(RECT {
                left: 10,
                top: 10,
                right: 100,
                bottom: 100,
            }))
        }
    }

    pub struct MockInput;
    impl InputProvider for MockInput {
        fn mouse_click(&self, _x: i32, _y: i32, _button: &str) -> anyhow::Result<()> {
            Ok(())
        }
        fn type_text(&self, _text: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }

    pub struct MockVision;
    #[async_trait]
    impl VisionProvider for MockVision {
        async fn recognize_text(&self, _image: &DynamicImage) -> anyhow::Result<Vec<DetectedWord>> {
            Ok(vec![DetectedWord {
                text: "Mock".to_string(),
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            }])
        }
    }

    pub struct MockBrowser;
    #[async_trait]
    impl BrowserProvider for MockBrowser {
        async fn query_selector(&self, _selector: &str) -> anyhow::Result<Value> {
            Ok(json!({"x": 50}))
        }
        async fn ensure_ready(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn create_handler() -> UltraWinHandler {
        UltraWinHandler {
            capture_engine: Some(Arc::new(MockCapture)),
            uia_bridge: Some(Arc::new(MockUIA)),
            input_manager: Arc::new(MockInput),
            vision_engine: Some(Arc::new(MockVision)),
            cdp_bridge: Arc::new(MockBrowser),
        }
    }

    #[tokio::test]
    async fn test_tools_exhaustive() {
        let handler = create_handler();

        let res = handler.handle_method("tools/list", None).await.unwrap();
        assert!(res.get("tools").is_some());

        let res = handler
            .handle_method("tools/call", Some(json!({"name": "screenshot"})))
            .await
            .unwrap();
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("successfully")
        );

        let res = handler
            .handle_method(
                "tools/call",
                Some(json!({"name": "click", "arguments": {"x": 1, "y": 1}})),
            )
            .await
            .unwrap();
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Clicked")
        );

        let res = handler
            .handle_method(
                "tools/call",
                Some(json!({"name": "type_text", "arguments": {"text": "hi"}})),
            )
            .await
            .unwrap();
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Typed")
        );

        let res = handler
            .handle_method("tools/call", Some(json!({"name": "get_ui_tree"})))
            .await
            .unwrap();
        assert!(res["content"][0]["text"].as_str().unwrap().contains("Root"));

        let res = handler
            .handle_method("tools/call", Some(json!({"name": "get_focused_element"})))
            .await
            .unwrap();
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Focus")
        );

        let res = handler
            .handle_method(
                "tools/call",
                Some(json!({"name": "find_element", "arguments": {"query": "test"}})),
            )
            .await
            .unwrap();
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Found")
        );

        let res = handler
            .handle_method("tools/call", Some(json!({"name": "read_text"})))
            .await
            .unwrap();
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Detected")
        );

        let res = handler
            .handle_method(
                "tools/call",
                Some(json!({"name": "web_query", "arguments": {"selector": "a"}})),
            )
            .await
            .unwrap();
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Web element model")
        );
    }
}
