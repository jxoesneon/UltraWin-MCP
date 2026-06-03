use ultrawin_mcp::vision::VisionEngine;

#[tokio::test]
async fn test_vision_engine_init() {
    // WinRT OCR engine should initialize successfully on any standard Windows 10+
    let engine = VisionEngine::new(None).await;
    assert!(engine.is_ok(), "VisionEngine should initialize via WinRT");
}

