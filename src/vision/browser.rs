

use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;
use tracing::{info, warn};

pub async fn ensure_browser_ready() -> Result<()> {
    let url = "http://127.0.0.1:9222/json/version";
    
    // Check if already running
    if reqwest::get(url).await.is_ok() {
        return Ok(());
    }

    info!("Browser not detected on port 9222. Attempting to launch...");

    // Try to launch Chrome or Edge with debugging enabled
    let paths = [
        "chrome.exe",
        r#"C:\Program Files\Google\Chrome\Application\chrome.exe"#,
        r#"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"#,
        "msedge.exe",
        r#"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"#,
    ];

    let mut launched = false;
    for path in paths {
        match Command::new(path)
            .args(&["--remote-debugging-port=9222", "--headless"])
            .spawn() {
            Ok(_) => {
                info!("Launched browser from: {}", path);
                launched = true;
                break;
            }
            Err(_) => continue,
        }
    }

    if !launched {
        // Fallback: try using 'start' via cmd which might find it via registry
        match Command::new("cmd")
            .args(&["/C", "start chrome.exe --remote-debugging-port=9222 --headless"])
            .spawn() {
            Ok(_) => {
                info!("Attempted launch via 'start chrome.exe'");
            }
            Err(_) => {
                warn!("Failed to launch browser automatically. Please ensure a browser is running with --remote-debugging-port=9222");
                return Err(anyhow::anyhow!("Could not launch browser"));
            }
        }
    }

    // Retry connection for up to 5 seconds
    for i in 0..10 {
        sleep(Duration::from_millis(500)).await;
        if reqwest::get(url).await.is_ok() {
            info!("Browser is ready after {} retries", i + 1);
            return Ok(());
        }
    }

    Err(anyhow::anyhow!("Browser launched but port 9222 is not responding after 5 seconds"))
}







