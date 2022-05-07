use std::rc::Rc;

use crate::backend::glyph_painter::{GlyphImageSlabBackend, GlyphPainterBackend};
use crate::generic_backend::{GenericGlyphImageSlabBackend, GenericGlyphPainterBackend};

const SLAB_HEIGHT_FACTOR: u32 = 16;

struct GlyphImageSlab<B: GenericGlyphImageSlabBackend = GlyphImageSlabBackend> {
    allocation_count: u32,
    allocated_height: u32,
    width: u32,
    height: u32,
    backend: B,
}

impl<B: GenericGlyphImageSlabBackend> GlyphImageSlab<B> {
    fn new(width: u32) -> Self {
        let height = width * SLAB_HEIGHT_FACTOR;
        Self {
            allocation_count: 0,
            allocated_height: 0,
            width,
            height,
            backend: B::new(width, height),
        }
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

pub struct GlyphPainter<B: GenericGlyphPainterBackend = GlyphPainterBackend> {
    image_buckets: GlyphImageFreeList,
    backend: B,
}

impl<B: GenericGlyphPainterBackend> GlyphPainter<B> {
    fn new() -> GlyphPainter<B> {
        Self {
            image_buckets: GlyphImageFreeList::new(),
            backend: B::new(),
        }
    }

    fn draw_glyphs(&mut self, glyphs: &[u16]) -> Vec<GlyphImageRef> {
        // TODO: return a SmallVec or StaticVec?

        // Have each glyph image width be a power of 2? (min 8)
        todo!()
    }
}

struct GlyphImageRef {
    image: Rc<GlyphImageSlab>,
    width: u32, // in pixels
    stride: u32, // in pixels
    //
    data: *mut u8,
}

impl GlyphImageRef {

}
