use ultrawin_mcp::server::UltraWinHandler;
use mcp_sdk_rs::server::ServerHandler;
use mcp_sdk_rs::error::{Error, ErrorCode};
use serde_json::json;

#[tokio::test]
async fn test_mcp_tools_list() {
    let handler = UltraWinHandler {
        capture_engine: None,
        uia_bridge: None,
        input_manager: ultrawin_mcp::input::InputManager::new(),
        vision_engine: None,
    };

    let result = handler.handle_method("tools/list", None).await.expect("Failed to list tools");
    let tools = result.get("tools").and_then(|t| t.as_array()).expect("No tools array");
    
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"screenshot"));
    assert!(names.contains(&"click"));
    assert!(names.contains(&"type_text"));
    assert!(names.contains(&"get_ui_tree"));
}

#[tokio::test]
async fn test_mcp_screenshot_call() {
    let handler = UltraWinHandler {
        capture_engine: None,
        uia_bridge: None,
        input_manager: ultrawin_mcp::input::InputManager::new(),
        vision_engine: None,
    };

    let params = json!({
        "name": "screenshot",
        "arguments": {}
    });

    let result = handler.handle_method("tools/call", Some(params)).await;
    
    match result {
        Ok(val) => {
            let content = val.get("content").and_then(|c| c.as_array()).unwrap();
            assert!(content.iter().any(|c| c["type"] == "image"));
        },
        Err(e) => {
            if let Error::Protocol { code, .. } = e {
                assert_eq!(code, ErrorCode::InternalError, "Should return InternalError if engine is missing");
            } else {
                panic!("Expected Protocol error, got {:?}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_mcp_click_valid() {
    let handler = UltraWinHandler {
        capture_engine: None,
        uia_bridge: None,
        input_manager: ultrawin_mcp::input::InputManager::new(),
        vision_engine: None,
    };

    let params = json!({
        "name": "click",
        "arguments": {
            "x": 100,
            "y": 200,
            "button": "left"
        }
    });

    let result = handler.handle_method("tools/call", Some(params)).await.expect("Click failed");
    let content = result.get("content").and_then(|c| c.as_array()).unwrap();
    assert!(content[0]["text"].as_str().unwrap().contains("Clicked left at (100, 200)"));
}

#[tokio::test]
async fn test_mcp_call_invalid_params() {
    let handler = UltraWinHandler {
        capture_engine: None,
        uia_bridge: None,
        input_manager: ultrawin_mcp::input::InputManager::new(),
        vision_engine: None,
    };

    let params = json!({
        "name": "click",
        "arguments": {
            "x": 100
        }
    });

    let result = handler.handle_method("tools/call", Some(params)).await;
    assert!(result.is_err());
    if let Err(Error::Protocol { code, .. }) = result {
        assert_eq!(code, ErrorCode::InvalidParams);
    } else {
        panic!("Expected Protocol error");
    }
}

#[tokio::test]
async fn test_mcp_unknown_tool() {
    let handler = UltraWinHandler {
        capture_engine: None,
        uia_bridge: None,
        input_manager: ultrawin_mcp::input::InputManager::new(),
        vision_engine: None,
    };

    let params = json!({
        "name": "non_existent_tool",
        "arguments": {}
    });

    let result = handler.handle_method("tools/call", Some(params)).await;
    assert!(result.is_err());
    if let Err(Error::Protocol { code, .. }) = result {
        assert_eq!(code, ErrorCode::MethodNotFound);
    } else {
        panic!("Expected Protocol error");
    }
}
