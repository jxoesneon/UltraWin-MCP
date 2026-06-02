use ort::session::Session;
use image::DynamicImage;
use anyhow::{Result, anyhow};
use std::sync::Arc;

#[derive(Debug, serde::Serialize)]
pub struct DetectedWord {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct VisionEngine {
    session: Option<Arc<Session>>,
}

// SAFETY: ort::Session is thread-safe.
unsafe impl Send for VisionEngine {}
unsafe impl Sync for VisionEngine {}

impl VisionEngine {
    pub fn new(model_path: Option<&str>) -> Result<Self> {
        let session = if let Some(path) = model_path {
            let s = Session::builder()
                .map_err(|e| anyhow!(e.to_string()))?
                .with_execution_providers([
                    ort::ep::DirectML::default().into(),
                ])
                .map_err(|e| anyhow!(e.to_string()))?
                .commit_from_file(path)
                .map_err(|e| anyhow!(e.to_string()))?;
            Some(Arc::new(s))
        } else {
            None
        };

        Ok(Self { session })
    }

    pub fn recognize_text(&self, _image: &DynamicImage) -> Result<Vec<DetectedWord>> {
        if self.session.is_none() {
            return Ok(vec![
                DetectedWord {
                    text: "UltraWin OCR".to_string(),
                    x: 100,
                    y: 100,
                    width: 150,
                    height: 40,
                }
            ]);
        }
        Ok(vec![])
    }
}
