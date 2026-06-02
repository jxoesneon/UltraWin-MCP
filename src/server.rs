use mcp_sdk_rs::server::{Server, ServerHandler};
use mcp_sdk_rs::error::{Error, ErrorCode};
use mcp_sdk_rs::types::{ClientCapabilities, Implementation, ServerCapabilities};
use serde_json::{json, Value};
use std::sync::Arc;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;
use image::ImageFormat;

use crate::capture::CaptureEngine;
use crate::uia::UIAutomationBridge;
use crate::input::InputManager;
use crate::vision::VisionEngine;

pub struct UltraWinHandler {
    pub capture_engine: Option<Arc<CaptureEngine>>,
    pub uia_bridge: Option<Arc<UIAutomationBridge>>,
    pub input_manager: InputManager,
    pub vision_engine: Option<Arc<VisionEngine>>,
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

    async fn handle_method(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, Error> {
        match method {
            "tools/list" => {
                Ok(json!({
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
                        }
                    ]
                }))
            }
            "tools/call" => {
                let params = params.ok_or_else(|| Error::protocol(ErrorCode::InvalidParams, "Missing parameters"))?;
                let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| Error::protocol(ErrorCode::InvalidParams, "Missing tool name"))?;
                let default_args = json!({});
                let arguments = params.get("arguments").unwrap_or(&default_args);

                match name {
                    "screenshot" => {
                        let engine = self.capture_engine.as_ref().ok_or_else(|| Error::protocol(ErrorCode::InternalError, "Capture engine not available"))?;
                        let frame = engine.capture_frame().map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Capture failed: {}", e)))?;
                        let mut buffer = Vec::new();
                        frame.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
                            .map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Encoding failed: {}", e)))?;
                        let b64 = general_purpose::STANDARD.encode(buffer);
                        Ok(json!({
                            "content": [
                                { "type": "text", "text": "Screenshot captured successfully." },
                                { "type": "image", "data": b64, "mimeType": "image/png" }
                            ]
                        }))
                    }
                    "click" => {
                        let x = arguments.get("x").and_then(|v| v.as_i64()).ok_or_else(|| Error::protocol(ErrorCode::InvalidParams, "Missing x"))? as i32;
                        let y = arguments.get("y").and_then(|v| v.as_i64()).ok_or_else(|| Error::protocol(ErrorCode::InvalidParams, "Missing y"))? as i32;
                        let button = arguments.get("button").and_then(|v| v.as_str()).unwrap_or("left");
                        self.input_manager.mouse_click(x, y, button).map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Click failed: {}", e)))?;
                        Ok(json!({ "content": [{ "type": "text", "text": format!("Clicked {} at ({}, {})", button, x, y) }] }))
                    }
                    "type_text" => {
                        let text = arguments.get("text").and_then(|v| v.as_str()).ok_or_else(|| Error::protocol(ErrorCode::InvalidParams, "Missing text"))?;
                        self.input_manager.type_text(text).map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Typing failed: {}", e)))?;
                        Ok(json!({ "content": [{ "type": "text", "text": format!("Typed: {}", text) }] }))
                    }
                    "get_ui_tree" => {
                        let uia = self.uia_bridge.as_ref().ok_or_else(|| Error::protocol(ErrorCode::InternalError, "UIA bridge not available"))?;
                        let root = uia.get_root().map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Failed to get root: {}", e)))?;
                        let tree = uia.traverse_tree(&root, 0);
                        Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&tree).unwrap_or_default() }] }))
                    }
                    "get_focused_element" => {
                        let uia = self.uia_bridge.as_ref().ok_or_else(|| Error::protocol(ErrorCode::InternalError, "UIA bridge not available"))?;
                        let element = uia.get_focused_element().map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Failed to get focused element: {}", e)))?;
                        let props = uia.traverse_tree(&element, 5);
                        Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&props).unwrap_or_default() }] }))
                    }
                    "find_element" => {
                        let query = arguments.get("query").and_then(|v| v.as_str()).ok_or_else(|| Error::protocol(ErrorCode::InvalidParams, "Missing query"))?;
                        let uia = self.uia_bridge.as_ref().ok_or_else(|| Error::protocol(ErrorCode::InternalError, "UIA bridge not available"))?;
                        let rect = uia.find_element(query).map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Search failed: {}", e)))?;
                        match rect {
                            Some(r) => Ok(json!({ "content": [{ "type": "text", "text": format!("Found at {{ left: {}, top: {}, right: {}, bottom: {} }}", r.left, r.top, r.right, r.bottom) }] })),
                            None => Ok(json!({ "content": [{ "type": "text", "text": "Element not found" }] })),
                        }
                    }
                    "read_text" => {
                        let vision = self.vision_engine.as_ref().ok_or_else(|| Error::protocol(ErrorCode::InternalError, "Vision engine not available"))?;
                        let engine = self.capture_engine.as_ref().ok_or_else(|| Error::protocol(ErrorCode::InternalError, "Capture engine not available"))?;
                        let frame = engine.capture_frame().map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Capture failed: {}", e)))?;
                        let words = vision.recognize_text(&frame).map_err(|e| Error::protocol(ErrorCode::InternalError, format!("OCR failed: {}", e)))?;
                        Ok(json!({ "content": [{ "type": "text", "text": format!("Detected words: {:?}", words) }] }))
                    }
                    _ => Err(Error::protocol(ErrorCode::MethodNotFound, format!("Unknown tool: {}", name))),
                }
            }
            _ => Err(Error::protocol(ErrorCode::MethodNotFound, format!("Unknown method: {}", method))),
        }
    }
}

pub fn build_server(
    transport: Arc<dyn mcp_sdk_rs::transport::Transport>,
    capture: Option<Arc<CaptureEngine>>,
    uia: Option<Arc<UIAutomationBridge>>,
    vision: Option<Arc<VisionEngine>>,
) -> Server {
    let handler = Arc::new(UltraWinHandler {
        capture_engine: capture,
        uia_bridge: uia,
        input_manager: InputManager::new(),
        vision_engine: vision,
    });
    Server::new(transport, handler)
}
