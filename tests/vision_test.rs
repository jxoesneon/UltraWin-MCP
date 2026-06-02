use ultrawin_mcp::vision::VisionEngine;

#[test]
fn test_vision_engine_mock_init() {
    let engine = VisionEngine::new(None).expect("Failed to initialize mock VisionEngine");
    let words = engine.recognize_text(&image::DynamicImage::new_rgb8(1, 1)).expect("Failed to recognize text");
    
    assert!(!words.is_empty());
    assert_eq!(words[0].text, "UltraWin OCR");
}

#[test]
fn test_vision_engine_invalid_path() {
    let result = VisionEngine::new(Some("non_existent_model.onnx"));
    assert!(result.is_err());
}
