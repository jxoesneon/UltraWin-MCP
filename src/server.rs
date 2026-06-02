use mcp_sdk_rs::server::{Server, ServerHandler};
use mcp_sdk_rs::error::{Error, ErrorCode};
use mcp_sdk_rs::types::{ClientCapabilities, Implementation, ServerCapabilities};
use serde_json::{json, Value};
use std::sync::Arc;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;
use image::ImageFormat;
use windows::Win32::UI::Accessibility::*;

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
                            "description": "Capture the screen",
                            "inputSchema": { "type": "object", "properties": {} }
                        },
                        {
                            "name": "get_ui_tree",
                            "description": "Retrieve the accessibility tree",
                            "inputSchema": { "type": "object", "properties": {} }
                        },
                        {
                            "name": "click",
                            "description": "Click at specific coordinates",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "x": { "type": "integer" },
                                    "y": { "type": "integer" },
                                    "button": { "type": "string", "enum": ["left", "right", "middle"] }
                                },
                                "required": ["x", "y"]
                            }
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

                        Ok(json!({
                            "content": [{ "type": "text", "text": format!("Clicked {} at ({}, {})", button, x, y) }]
                        }))
                    }
                    "get_ui_tree" => {
                        let uia = self.uia_bridge.as_ref().ok_or_else(|| Error::protocol(ErrorCode::InternalError, "UIA bridge not available"))?;
                        let root = uia.get_root().map_err(|e| Error::protocol(ErrorCode::InternalError, format!("Failed to get root: {}", e)))?;
                        
                        let mut elements = Vec::new();
                        unsafe {
                            let name = root.CurrentName().unwrap_or_default().to_string();
                            let automation_id = root.CurrentAutomationId().unwrap_or_default().to_string();
                            elements.push(json!({
                                "name": name,
                                "automation_id": automation_id,
                                "type": "Root"
                            }));
                        }

                        Ok(json!({
                            "content": [{ "type": "text", "text": format!("UI Tree Root: {:?}", elements) }]
                        }))
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
