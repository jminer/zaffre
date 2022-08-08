

pub(crate) trait GenericGlyphImageSlabBackend: Debug + Clone {
    type MemoryRef: GenericGlyphImageMemoryRef;

    fn new(width: u32, height: u32) -> Self;

    fn data(&self, y_range: Range<u32>) -> Self::MemoryRef;
}



const SLAB_HEIGHT_FACTOR: u32 = 16;

struct GlyphImageSlab<B: GenericGlyphImageSlabBackend = GlyphImageSlabBackend> {
    allocation_count: Cell<u32>,
    allocated_height: Cell<u32>,
    width: u32,
    height: u32,
    backend: B,
}

impl<B: GenericGlyphImageSlabBackend> GlyphImageSlab<B> {
    fn new(width: u32) -> Self {
        let height = width * SLAB_HEIGHT_FACTOR;
        Self {
            allocation_count: Cell::new(0),
            allocated_height: Cell::new(0),
            width,
            height,
            backend: B::new(width, height),
        }
    }

    fn data_ptr(&self) -> *mut u8 {
        self.backend
    }
}

struct GlyphImageFreeListBucket {
    width: u32,
    images: Vec<Rc<GlyphImageSlab>>,
}

impl GlyphImageFreeListBucket {
    fn new(width: u32) -> Self {
        Self { width, images: Vec::new() }
    }

    fn allocate(&mut self, width: u32, height: u32) -> GlyphImageRef {
        assert!(width < self.width);
        loop {
            if let Some(slab) = self.images.last() {
                assert!(slab.height >= height);
                let free_height = slab.height - slab.allocated_height;
                slab.allocation_count.update(|n| n + 1);
                slab.allocated_height.update(|h| h + height);
                return GlyphImageRef {
                    image: slab.clone(),
                    width: width,
                    stride: self.width,
                    data: todo!(),
                };
            } else {
                self.images.push(Rc::new(GlyphImageSlab::new(self.width)));
                // With the new image slab, the next loop iteration will definitely succeed.
            }
        }
    }
}

#[derive(Default)]
struct GlyphImageFreeList(Vec<GlyphImageFreeListBucket>);

impl GlyphImageFreeList {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn allocate_image_ref(&mut self, width: u32, height: u32) -> GlyphImageRef {
        self.0[Self::width_to_index(width)].allocate(width, height)
    }

    fn width_to_index(width: u32) -> usize {
        // Subtracting 1 from the width will always give one index too small, so then we can add 1.
        let index = (width.max(8) - 1).log2() + 1;
        index.saturating_sub(3) as usize // skip the three smallest widths: 1, 2, and 4
    }

    fn index_to_width(index: usize) -> u32 {
        2u32.pow(index as u32 + 3)
    }

}

#[test]
fn test_glyph_free_list_width_to_index() {
    assert_eq!(GlyphImageFreeList::width_to_index(4), 0);
    assert_eq!(GlyphImageFreeList::width_to_index(7), 0);
    assert_eq!(GlyphImageFreeList::width_to_index(8), 0);
    assert_eq!(GlyphImageFreeList::width_to_index(9), 1);
    assert_eq!(GlyphImageFreeList::width_to_index(15), 1);
    assert_eq!(GlyphImageFreeList::width_to_index(16), 1);
    assert_eq!(GlyphImageFreeList::width_to_index(17), 2);
}

#[test]
fn test_glyph_free_list_index_to_width() {
    assert_eq!(GlyphImageFreeList::index_to_width(0), 8);
    assert_eq!(GlyphImageFreeList::index_to_width(1), 16);
    assert_eq!(GlyphImageFreeList::index_to_width(2), 32);
}

pub(crate) struct GlyphImageRef {
    image: Rc<GlyphImageSlab>,
    height: u32,
}

impl Drop for GlyphImageRef {
    fn drop(&mut self) {
        self.image.allocation_count.update(|n| n - 1);
        self.image.allocated_height.update(|h| h - self.height);
    }
}

// Windows backend:


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
    type MemoryRef;

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

    fn data(&self) -> *mut u8 {
        self.bitmap.Lock()
    }
}
