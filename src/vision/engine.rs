use image::{DynamicImage, ImageFormat};
use anyhow::{Result, anyhow};
use std::io::Cursor;
use windows::Media::Ocr::OcrEngine;
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Storage::Streams::InMemoryRandomAccessStream;
use async_trait::async_trait;
use crate::traits::VisionProvider;

#[derive(Debug, serde::Serialize, Clone)]
pub struct DetectedWord {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct VisionEngine {
    engine: OcrEngine,
}

unsafe impl Send for VisionEngine {}
unsafe impl Sync for VisionEngine {}

#[async_trait]
impl VisionProvider for VisionEngine {
    
    async fn recognize_text(&self, image: &DynamicImage) -> Result<Vec<DetectedWord>> {
        let mut buffer = Vec::new();
        image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Bmp)
            .map_err(|e| anyhow!("Failed to encode image for OCR: {}", e))?;

        let stream = InMemoryRandomAccessStream::new()?;
        let writer = stream.GetOutputStreamAt(0)?;
        
        let data_writer = windows::Storage::Streams::DataWriter::CreateDataWriter(&writer)?;
        data_writer.WriteBytes(&buffer)?;
        data_writer.StoreAsync()?.get()?;
        data_writer.FlushAsync()?.get()?;

        let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
        let software_bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

        let result = self.engine.RecognizeAsync(&software_bitmap)?.get()?;
        
        let mut words = Vec::new();
        for line in result.Lines()? {
            for word in line.Words()? {
                let rect = word.BoundingRect()?;
                words.push(DetectedWord {
                    text: word.Text()?.to_string(),
                    x: rect.X as i32,
                    y: rect.Y as i32,
                    width: rect.Width as i32,
                    height: rect.Height as i32,
                });
            }
        }

        Ok(words)
    }
}

impl VisionEngine {
    
    pub async fn new(_model_path: Option<&str>) -> Result<Self> {
        let engine = OcrEngine::TryCreateFromUserProfileLanguages()
            .map_err(|e| anyhow!("Failed to create WinRT OCR Engine: {}", e))?;
        
        Ok(Self { engine })
    }
}






