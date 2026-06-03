use crate::traits::CaptureProvider;
use anyhow::{Context, Result, anyhow};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::sync::Mutex;
use tracing::warn;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_1};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CREATE_DEVICE_FLAG, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11CreateDevice, ID3D11Device,
    ID3D11DeviceContext, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_FRAME_INFO, IDXGIAdapter1,
    IDXGIFactory1, IDXGIOutput, IDXGIOutput1, IDXGIOutputDuplication,
};
use windows::Win32::Graphics::Gdi::{
    BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use windows::core::Interface;

pub struct CaptureEngine {
    display_index: usize,
    state: Mutex<Option<CaptureState>>,
}

struct CaptureState {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
}

unsafe impl Send for CaptureEngine {}
unsafe impl Sync for CaptureEngine {}

impl CaptureProvider for CaptureEngine {
    fn capture_frame(&self) -> Result<DynamicImage> {
        match self.capture_frame_dxgi() {
            Ok(img) => Ok(img),
            Err(e) => {
                warn!("DXGI Capture failed: {}. Falling back to GDI.", e);
                self.capture_frame_gdi()
            }
        }
    }
}

impl CaptureEngine {
    pub fn new(display_index: usize) -> Result<Self> {
        let engine = Self {
            display_index,
            state: Mutex::new(None),
        };
        let _ = engine.ensure_initialized();
        Ok(engine)
    }

    fn ensure_initialized(&self) -> Result<()> {
        let mut state_guard = self.state.lock().unwrap();
        if state_guard.is_some() {
            return Ok(());
        }

        unsafe {
            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            let mut feature_level = D3D_FEATURE_LEVEL_11_1;

            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_FLAG(0),
                Some(&[D3D_FEATURE_LEVEL_11_1]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                Some(&mut feature_level),
                Some(&mut context),
            )
            .context("Failed to create D3D11 device")?;

            let device = device.unwrap();
            let context = context.unwrap();
            let factory: IDXGIFactory1 =
                CreateDXGIFactory1().context("Failed to create DXGI Factory")?;
            let adapter: IDXGIAdapter1 =
                factory.EnumAdapters1(0).context("Failed to enum adapter")?;
            let output: IDXGIOutput = adapter
                .EnumOutputs(self.display_index as u32)
                .context("Failed to enum output")?;
            let output1: IDXGIOutput1 = output
                .cast()
                .context("Failed to cast output to IDXGIOutput1")?;
            let duplication = output1
                .DuplicateOutput(&device)
                .context("Failed to duplicate output")?;

            *state_guard = Some(CaptureState {
                device,
                context,
                duplication,
            });
            Ok(())
        }
    }

    fn capture_frame_dxgi(&self) -> Result<DynamicImage> {
        self.ensure_initialized()?;

        let state_guard = self.state.lock().unwrap();
        let state = state_guard
            .as_ref()
            .ok_or_else(|| anyhow!("Capture state not initialized"))?;

        unsafe {
            let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut resource: Option<windows::Win32::Graphics::Dxgi::IDXGIResource> = None;

            state
                .duplication
                .AcquireNextFrame(100, &mut frame_info, &mut resource)
                .map_err(|e| {
                    if e.code() == DXGI_ERROR_WAIT_TIMEOUT {
                        anyhow!("Timeout waiting for frame")
                    } else {
                        anyhow!("Failed to acquire next frame: {:?}", e)
                    }
                })?;

            let resource = resource.context("AcquireNextFrame returned null resource")?;
            let texture: ID3D11Texture2D = resource
                .cast()
                .context("Failed to cast resource to ID3D11Texture2D")?;

            let mut texture_desc = D3D11_TEXTURE2D_DESC::default();
            texture.GetDesc(&mut texture_desc);

            let mut staging_desc = texture_desc;
            staging_desc.Usage = D3D11_USAGE_STAGING;
            staging_desc.BindFlags = 0;
            staging_desc.CPUAccessFlags = 0x10000; // D3D11_CPU_ACCESS_READ
            staging_desc.MiscFlags = 0;

            let mut staging_texture: Option<ID3D11Texture2D> = None;
            state
                .device
                .CreateTexture2D(&staging_desc, None, Some(&mut staging_texture))
                .context("Failed to create staging texture")?;
            let staging_texture = staging_texture.unwrap();

            state.context.CopyResource(&staging_texture, &texture);

            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            let map_res =
                state
                    .context
                    .Map(&staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped));

            if map_res.is_err() {
                let _ = state.duplication.ReleaseFrame();
                return Err(anyhow!("Failed to map staging texture"));
            }

            let width = texture_desc.Width;
            let height = texture_desc.Height;
            let pitch = mapped.RowPitch as usize;

            // Call the unsafe function since we are in an unsafe block
            let img = convert_bgra_to_rgba(mapped.pData as *const u8, width, height, pitch)?;

            state.context.Unmap(&staging_texture, 0);
            let _ = state.duplication.ReleaseFrame();

            Ok(img)
        }
    }

    fn capture_frame_gdi(&self) -> Result<DynamicImage> {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            let h_dc_screen = GetDC(None);
            let h_dc_mem = CreateCompatibleDC(h_dc_screen);
            let h_bitmap = CreateCompatibleBitmap(h_dc_screen, width, height);

            let old_obj = SelectObject(h_dc_mem, h_bitmap);
            BitBlt(h_dc_mem, 0, 0, width, height, h_dc_screen, 0, 0, SRCCOPY)?;

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0, // BI_RGB
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut buffer = vec![0u8; (width * height * 4) as usize];
            GetDIBits(
                h_dc_screen,
                h_bitmap,
                0,
                height as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            let img = process_gdi_buffer(buffer, width as u32, height as u32)?;

            let _ = SelectObject(h_dc_mem, old_obj);
            let _ = DeleteDC(h_dc_mem);
            ReleaseDC(None, h_dc_screen);
            let _ = DeleteObject(h_bitmap);

            Ok(img)
        }
    }
}

/// # Safety
/// `data_ptr` must point to a valid BGRA buffer of at least `pitch * height` bytes.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn convert_bgra_to_rgba(
    data_ptr: *const u8,
    width: u32,
    height: u32,
    pitch: usize,
) -> Result<DynamicImage> {
    let data = std::slice::from_raw_parts(data_ptr, pitch * height as usize);
    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let row_start = (y as usize) * pitch;
        let row_end = row_start + (width as usize * 4);
        if row_end > data.len() {
            return Err(anyhow!("Row pitch out of bounds"));
        }
        let row = &data[row_start..row_end];
        for chunk in row.chunks_exact(4) {
            rgba_data.push(chunk[2]);
            rgba_data.push(chunk[1]);
            rgba_data.push(chunk[0]);
            rgba_data.push(chunk[3]);
        }
    }
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_data)
        .context("Failed to create image buffer")?;
    Ok(DynamicImage::ImageRgba8(img))
}

pub fn process_gdi_buffer(mut buffer: Vec<u8>, width: u32, height: u32) -> Result<DynamicImage> {
    for chunk in buffer.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, buffer)
        .context("Failed to create GDI image buffer")?;
    Ok(DynamicImage::ImageRgba8(img))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_processing() {
        let buf = vec![0, 1, 2, 3]; // BGRA
        let img = process_gdi_buffer(buf, 1, 1).unwrap();
        assert_eq!(img.as_rgba8().unwrap().get_pixel(0, 0).0, [2, 1, 0, 3]);
    }
}
