
use std::fmt::Debug;

use crate::font::{OpenTypeFontWeight, FontStyle, OpenTypeFontStretch, FontFamily, Font, FontDescription};

pub(crate) trait GenericFontFunctionsBackend {
    fn get_families() -> Vec<FontFamily> {
        todo!()
    }

    fn get_family(name: &str) -> Option<FontFamily> {
        todo!()
    }

}

pub trait GenericFontFamilyBackend: Debug + Clone {
    fn get_name(&self) -> String;

    fn get_matching_font(&self,
        weight: OpenTypeFontWeight,
        style: FontStyle,
        stretch: OpenTypeFontStretch,
    ) -> Font;
}

pub trait GenericFontBackend: Debug + Clone {
    fn description(&self) -> FontDescription;
}

pub trait GenericFontDescriptionBackend: Debug + Clone {
    fn get_face_name(&self) -> String;

    fn weight(&self) -> OpenTypeFontWeight;

    fn style(&self) -> FontStyle;

    fn stretch(&self) -> OpenTypeFontStretch;

    fn is_monospaced(&self) -> bool;

    fn has_color_glyphs(&self) -> bool;
}

pub(crate) trait GenericGlyphImageSlabBackend: Debug + Clone {
    fn new(width: u32, height: u32) -> Self;
}

pub trait GenericGlyphPainterBackend: Debug + Clone {
    fn new() -> Self;
}
