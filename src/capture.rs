use anyhow::{Context, Result, anyhow};
use image::{DynamicImage, ImageBuffer, Rgba};
use windows::core::Interface;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_1};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_CREATE_DEVICE_FLAG,
    D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, IDXGIAdapter1, IDXGIOutput, IDXGIOutput1,
    IDXGIOutputDuplication, DXGI_OUTPUT_DESC, DXGI_OUTDUPL_FRAME_INFO,
    DXGI_ERROR_WAIT_TIMEOUT,
};

pub struct CaptureEngine {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
    _output_desc: DXGI_OUTPUT_DESC,
}

// SAFETY: Windows DXGI and D3D11 objects are thread-safe or handled correctly via internal ref counting.
unsafe impl Send for CaptureEngine {}
unsafe impl Sync for CaptureEngine {}

impl CaptureEngine {
    pub fn new(display_index: usize) -> Result<Self> {
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
            ).context("Failed to create D3D11 device")?;

            let device = device.unwrap();
            let context = context.unwrap();

            let factory: IDXGIFactory1 = CreateDXGIFactory1().context("Failed to create DXGI Factory")?;
            let adapter: IDXGIAdapter1 = factory.EnumAdapters1(0).context("Failed to enum adapter")?;
            let output: IDXGIOutput = adapter.EnumOutputs(display_index as u32).context("Failed to enum output")?;
            let output1: IDXGIOutput1 = output.cast().context("Failed to cast output to IDXGIOutput1")?;

            let duplication = output1.DuplicateOutput(&device).context("Failed to duplicate output")?;
            let desc = output.GetDesc().context("Failed to get output desc")?;

            Ok(Self {
                device,
                context,
                duplication,
                _output_desc: desc,
            })
        }
    }

    pub fn capture_frame(&self) -> Result<DynamicImage> {
        unsafe {
            let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut resource: Option<windows::Win32::Graphics::Dxgi::IDXGIResource> = None;

            self.duplication.AcquireNextFrame(100, &mut frame_info, &mut resource)
                .map_err(|e| {
                    if e.code() == DXGI_ERROR_WAIT_TIMEOUT {
                        anyhow!("Timeout waiting for frame")
                    } else {
                        anyhow!("Failed to acquire next frame: {:?}", e)
                    }
                })?;

            let resource = resource.context("AcquireNextFrame returned null resource")?;
            let texture: ID3D11Texture2D = resource.cast().context("Failed to cast resource to ID3D11Texture2D")?;

            let mut texture_desc = D3D11_TEXTURE2D_DESC::default();
            texture.GetDesc(&mut texture_desc);

            let mut staging_desc = texture_desc;
            staging_desc.Usage = D3D11_USAGE_STAGING;
            staging_desc.BindFlags = 0;
            staging_desc.CPUAccessFlags = 0x10000; // D3D11_CPU_ACCESS_READ
            staging_desc.MiscFlags = 0;

            let mut staging_texture: Option<ID3D11Texture2D> = None;
            self.device.CreateTexture2D(&staging_desc, None, Some(&mut staging_texture))
                .context("Failed to create staging texture")?;
            let staging_texture = staging_texture.unwrap();

            self.context.CopyResource(&staging_texture, &texture);

            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            self.context.Map(&staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                .context("Failed to map staging texture")?;

            let width = texture_desc.Width;
            let height = texture_desc.Height;
            let pitch = mapped.RowPitch as usize;
            let data = std::slice::from_raw_parts(mapped.pData as *const u8, pitch * height as usize);

            let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
            for y in 0..height {
                let row_start = (y as usize) * pitch;
                let row_end = row_start + (width as usize * 4);
                if row_end > data.len() {
                    return Err(anyhow!("Row pitch out of bounds"));
                }
                let row = &data[row_start..row_end];
                for chunk in row.chunks_exact(4) {
                    rgba_data.push(chunk[2]); // R
                    rgba_data.push(chunk[1]); // G
                    rgba_data.push(chunk[0]); // B
                    rgba_data.push(chunk[3]); // A
                }
            }

            self.context.Unmap(&staging_texture, 0);
            self.duplication.ReleaseFrame().context("Failed to release frame")?;

            let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_data)
                .context("Failed to create image buffer")?;

            Ok(DynamicImage::ImageRgba8(img))
        }
    }
}
