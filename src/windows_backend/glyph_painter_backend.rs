use std::ptr;

use windows::Win32::Graphics::Direct2D::Common::{D2D1_PIXEL_FORMAT, D2D1_ALPHA_MODE_PREMULTIPLIED};
use windows::Win32::Graphics::Direct2D::{ID2D1RenderTarget, ID2D1Factory, D2D1CreateFactory, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FACTORY_OPTIONS, D2D1_DEBUG_LEVEL_INFORMATION, D2D1_DEBUG_LEVEL_NONE, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_FEATURE_LEVEL_DEFAULT};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_A8_UNORM;
use windows::Win32::Graphics::Imaging::{IWICBitmap, IWICImagingFactory, GUID_WICPixelFormat8bppAlpha, WICBitmapCacheOnDemand};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::core::Interface;

use crate::generic_backend::{GenericGlyphImageSlabBackend, GenericGlyphPainterBackend};

thread_local! {
    pub(crate) static D2D_FACTORY: ID2D1Factory = unsafe {
        let options: D2D1_FACTORY_OPTIONS = D2D1_FACTORY_OPTIONS {
            debugLevel: if cfg!(debug_assertions) {
                D2D1_DEBUG_LEVEL_INFORMATION
            } else {
                D2D1_DEBUG_LEVEL_NONE
            },
        };
        let mut factory: *mut ID2D1Factory = ptr::null_mut();
        D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &ID2D1Factory::IID, &options,
            &mut factory as *mut *mut ID2D1Factory as *mut _)
            .expect("failed to create ID2D1Factory");
        factory.read()
    }
}

thread_local! {
    pub(crate) static WIC_IMAGING_FACTORY: IWICImagingFactory = unsafe {
        CoCreateInstance(&IWICImagingFactory::IID, None, CLSCTX_INPROC_SERVER)
            .expect("failed to create IWICImagingFactory")
    }
}


#[derive(Debug, Clone)]
pub struct GlyphImageSlabBackend {
    bitmap: IWICBitmap,
    render_target: ID2D1RenderTarget,
}

impl GenericGlyphImageSlabBackend for GlyphImageSlabBackend {
    fn new(width: u32, height: u32) -> Self {
        unsafe {
            let bitmap = WIC_IMAGING_FACTORY.with(|wic_factory| {
                wic_factory.CreateBitmap(width, height, &GUID_WICPixelFormat8bppAlpha, WICBitmapCacheOnDemand)
                    .expect("failed to create IWICBitmap")
            });
            let bitmap_clone = bitmap.clone();
            let render_target = D2D_FACTORY.with(|d2d_factory| {
                let properties = D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 0.0,
                    dpiY: 0.0,
                    usage: D2D1_RENDER_TARGET_USAGE_NONE,
                    minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
                };
                d2d_factory.CreateWicBitmapRenderTarget(bitmap_clone, &properties)
                    .expect("failed to create WicBitmapRenderTarget")
            });
            Self {
                bitmap,
                render_target,
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct GlyphPainterBackend {

}

impl GenericGlyphPainterBackend for GlyphPainterBackend {
    fn new() -> Self {
        Self { }
    }
}
