use std::ptr;

use crate::generic_backend::{GenericGlyphImageSlabBackend, GenericGlyphPainterBackend};


#[derive(Debug, Clone)]
pub struct GlyphImageSlabBackend {
}

impl GenericGlyphImageSlabBackend for GlyphImageSlabBackend {
    fn new(width: u32, height: u32) -> Self {
        unsafe {
            Self {
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
