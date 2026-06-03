use crate::vision::engine::DetectedWord;
use anyhow::Result;
use async_trait::async_trait;
use image::DynamicImage;
use serde_json::Value;
use windows::Win32::Foundation::RECT;

#[async_trait]
pub trait CaptureProvider: Send + Sync {
    fn capture_frame(&self) -> Result<DynamicImage>;
}

#[async_trait]
pub trait UIAutomationProvider: Send + Sync {
    fn get_root_json(&self) -> Result<Value>;
    fn get_focused_json(&self) -> Result<Value>;
    fn find_element(&self, query: &str) -> Result<Option<RECT>>;
}

#[async_trait]
pub trait InputProvider: Send + Sync {
    fn mouse_click(&self, x: i32, y: i32, button: &str) -> Result<()>;
    fn type_text(&self, text: &str) -> Result<()>;
}

#[async_trait]
pub trait VisionProvider: Send + Sync {
    async fn recognize_text(&self, image: &DynamicImage) -> Result<Vec<DetectedWord>>;
}

#[async_trait]
pub trait BrowserProvider: Send + Sync {
    async fn query_selector(&self, selector: &str) -> Result<Value>;
    async fn ensure_ready(&self) -> Result<()>;
}
