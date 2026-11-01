#![allow(
    unknown_lints,
    clippy::unknown_lints,
    clippy::chunks_exact_to_as_chunks,
    clippy::double_must_use,
    clippy::collapsible_if
)]

mod capture;
mod input;
mod server;
mod traits;
mod uia;
mod vision;

use crate::capture::CaptureEngine;
use crate::input::InputManager;
use crate::server::stdio_transport::StdioTransport;
use crate::uia::UIAutomationBridge;
use crate::vision::VisionEngine;
use crate::vision::cdp::CdpBridge;
use std::sync::Arc;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_server().await
}

pub async fn run_server() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

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
            Some(bridge)
        }
        Err(e) => {
            tracing::error!("Failed to initialize UIA3 Bridge: {:?}", e);
            None
        }
    };

    let vision_engine = match VisionEngine::new(None).await {
        Ok(engine) => {
            info!("Vision Engine initialized");
            Some(Arc::new(engine))
        }
        Err(e) => {
            tracing::error!("Failed to initialize Vision Engine: {:?}", e);
            None
        }
    };

    let input_manager = Arc::new(InputManager::new());
    let cdp_bridge = Arc::new(CdpBridge::new("127.0.0.1", 9222));

    info!("Initializing MCP Server with Standard Stdio Transport...");

    let transport = Arc::new(StdioTransport::new(tokio::io::stdin(), tokio::io::stdout()));
    let server = server::build_server(
        transport,
        capture_engine.map(|e| e as Arc<dyn crate::traits::CaptureProvider>),
        uia_bridge.map(|b| Arc::new(b) as Arc<dyn crate::traits::UIAutomationProvider>),
        vision_engine.map(|v| v as Arc<dyn crate::traits::VisionProvider>),
        input_manager as Arc<dyn crate::traits::InputProvider>,
        cdp_bridge as Arc<dyn crate::traits::BrowserProvider>,
    );

    info!("UltraWin MCP is ready for commands.");

    let _ = server.start().await;

    Ok(())
}
