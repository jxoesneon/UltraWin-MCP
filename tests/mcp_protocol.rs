use ultrawin_mcp::server::UltraWinHandler;
use ultrawin_mcp::traits::*;
use mcp_sdk_rs::server::ServerHandler;
use mcp_sdk_rs::error::{Error, ErrorCode};
use serde_json::{json, Value};
use std::sync::Arc;
use async_trait::async_trait;
use image::DynamicImage;
use ultrawin_mcp::vision::engine::DetectedWord;
use windows::Win32::Foundation::RECT;

// MOCKS
pub struct MockCapture;
#[async_trait]
impl CaptureProvider for MockCapture {
    fn capture_frame(&self) -> anyhow::Result<DynamicImage> {
        Ok(DynamicImage::new_rgba8(1, 1))
    }
}

pub struct MockUIA;
#[async_trait]
impl UIAutomationProvider for MockUIA {
    fn get_root_json(&self) -> anyhow::Result<Value> { Ok(json!({"name": "Root"})) }
    fn get_focused_json(&self) -> anyhow::Result<Value> { Ok(json!({"name": "Focus"})) }
    fn find_element(&self, _query: &str) -> anyhow::Result<Option<RECT>> { 
        Ok(Some(RECT { left: 10, top: 10, right: 100, bottom: 100 })) 
    }
}

pub struct MockInput;
#[async_trait]
impl InputProvider for MockInput {
    fn mouse_click(&self, _x: i32, _y: i32, _button: &str) -> anyhow::Result<()> { Ok(()) }
    fn type_text(&self, _text: &str) -> anyhow::Result<()> { Ok(()) }
}

pub struct MockVision;
#[async_trait]
impl VisionProvider for MockVision {
    async fn recognize_text(&self, _image: &DynamicImage) -> anyhow::Result<Vec<DetectedWord>> {
        Ok(vec![DetectedWord { text: "Mock".to_string(), x: 0, y: 0, width: 10, height: 10 }])
    }
}

pub struct MockBrowser;
#[async_trait]
impl BrowserProvider for MockBrowser {
    async fn query_selector(&self, _selector: &str) -> anyhow::Result<Value> { Ok(json!({"x": 50})) }
    async fn ensure_ready(&self) -> anyhow::Result<()> { Ok(()) }
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
async fn test_all_tools() {
    let handler = create_handler();

    // 1. screenshot
    let res = handler.handle_method("tools/call", Some(json!({"name": "screenshot"}))).await.unwrap();
    assert!(res["content"][1]["type"] == "image");

    // 2. click
    let res = handler.handle_method("tools/call", Some(json!({"name": "click", "arguments": {"x": 1, "y": 1}}))).await.unwrap();
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Clicked"));

    // 3. type_text
    let res = handler.handle_method("tools/call", Some(json!({"name": "type_text", "arguments": {"text": "hi"}}))).await.unwrap();
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Typed"));

    // 4. get_ui_tree
    let res = handler.handle_method("tools/call", Some(json!({"name": "get_ui_tree"}))).await.unwrap();
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Root"));

    // 5. get_focused_element
    let res = handler.handle_method("tools/call", Some(json!({"name": "get_focused_element"}))).await.unwrap();
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Focus"));

    // 6. find_element
    let res = handler.handle_method("tools/call", Some(json!({"name": "find_element", "arguments": {"query": "test"}}))).await.unwrap();
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Found at"));

    // 7. read_text
    let res = handler.handle_method("tools/call", Some(json!({"name": "read_text"}))).await.unwrap();
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Mock"));

    // 8. web_query
    let res = handler.handle_method("tools/call", Some(json!({"name": "web_query", "arguments": {"selector": "a"}}))).await.unwrap();
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Web element model"));
}

#[tokio::test]
async fn test_error_cases() {
    let handler = create_handler();

    // Missing tool name
    let res = handler.handle_method("tools/call", Some(json!({"arguments": {}}))).await;
    match res {
        Err(Error::Protocol { code, .. }) => assert_eq!(code, ErrorCode::InvalidParams),
        _ => panic!("Expected Protocol InvalidParams error"),
    }

    // Unknown method
    let res = handler.handle_method("unknown", None).await;
    match res {
        Err(Error::Protocol { code, .. }) => assert_eq!(code, ErrorCode::MethodNotFound),
        _ => panic!("Expected Protocol MethodNotFound error"),
    }
}

