use ultrawin_mcp::capture::CaptureEngine;
use ultrawin_mcp::uia::UIAutomationBridge;
use ultrawin_mcp::vision::VisionEngine;
use ultrawin_mcp::server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::Arc;
use tokio::sync::mpsc;
use mcp_sdk_rs::transport::stdio::StdioTransport;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("UltraWin MCP - State-of-the-Art Windows Automation Server");

    let capture_engine = match CaptureEngine::new(0) {
        Ok(engine) => {
            info!("DXGI Capture Engine initialized (Primary Monitor)");
            Some(Arc::new(engine))
        }
        Err(e) => {
            tracing::error!("Failed to initialize DXGI Capture: {:?}", e);
            None
        }
    };

    let uia_bridge = match UIAutomationBridge::new() {
        Ok(bridge) => {
            info!("UIA3 High-Performance Bridge initialized");
            Some(Arc::new(bridge))
        }
        Err(e) => {
            tracing::error!("Failed to initialize UIA3 Bridge: {:?}", e);
            None
        }
    };
    
    let vision_engine = match VisionEngine::new(None) {
        Ok(engine) => {
            info!("Vision Engine initialized (Placeholder mode)");
            Some(Arc::new(engine))
        }
        Err(e) => {
            tracing::error!("Failed to initialize Vision Engine: {:?}", e);
            None
        }
    };

    info!("Initializing MCP Server...");

    let (_read_tx, read_rx) = mpsc::channel(100);
    let (write_tx, mut _write_rx) = mpsc::channel(100);

    let transport = Arc::new(StdioTransport::new(read_rx, write_tx));
    let server = server::build_server(transport, capture_engine, uia_bridge, vision_engine);

    info!("UltraWin MCP is ready for commands.");

    let _ = server.start().await;

    Ok(())
}
