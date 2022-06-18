
use std::fmt::Debug;

use crate::font::{OpenTypeFontWeight, FontSlant, OpenTypeFontWidth, FontFamily, Font, FontDescription};
use crate::text_analyzer::{TextAnalyzerRun, TextAnalyzer};

// region: font

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

    fn get_styles(&self) -> Vec<FontDescription>;

    fn get_matching_font(&self,
        weight: OpenTypeFontWeight,
        slant: FontSlant,
        width: OpenTypeFontWidth,
    ) -> Font;
}

pub trait GenericFontDescriptionBackend: Debug + Clone {
    fn get_family_name(&self) -> String;

    fn get_style_name(&self) -> String;

    fn weight(&self) -> OpenTypeFontWeight;

    fn slant(&self) -> FontSlant;

    fn width(&self) -> OpenTypeFontWidth;

    fn is_monospaced(&self) -> bool;

    fn has_color_glyphs(&self) -> bool;

    fn get_font(&self) -> Font;
}

pub trait GenericFontBackend: Debug + Clone {
    fn description(&self) -> FontDescription;
}

// endregion:

// region: glyph_painter

pub(crate) trait GenericGlyphImageSlabBackend: Debug + Clone {
    fn new(width: u32, height: u32) -> Self;
}

pub trait GenericGlyphPainterBackend: Debug + Clone {
    fn new() -> Self;
}

// endregion:

// region: text_analyzer

pub trait GenericTextAnalyzerRunBackend: Debug + Clone {
}

pub trait GenericTextAnalyzerBackend: Debug + Clone {
    fn new(text: String) -> Self;

    fn text(&self) -> &str;

    fn get_runs(&self) -> Vec<TextAnalyzerRun>;

}

// endregion:
