
use std::fmt::Debug;
use std::ops::Range;
use std::rc::Rc;

use glam::Affine2;
use nalgebra::Point2;
use smallvec::SmallVec;

use crate::font::{OpenTypeFontWeight, FontSlant, OpenTypeFontWidth, FontFamily, Font, FontDescription, GlyphImage};
use crate::text::FormattedString;
use crate::text_analyzer::{TextAnalyzerRun, TextAnalyzer, TextAnalyzerGlyphRun};

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
        size: f32,
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

    fn get_font(&self, size: f32) -> Font;
}

pub trait GenericFontBackend: Debug + Clone {
    fn size(&self) -> f32;

    fn description(&self) -> FontDescription;

    fn draw_glyphs(
        &self,
        glyphs: &[u16],
        offsets: &[Point2<f32>],
        transform: Affine2,
    ) -> SmallVec<[GlyphImage; 32]>;
}

pub trait GenericGlyphImageBackend: Debug + Clone {
}

// endregion:

// region: text_analyzer

pub trait GenericTextAnalyzerRunBackend: Debug + Clone {
}

pub trait GenericTextAnalyzerBackend: Debug {
    fn new() -> Self;

    fn text(&self) -> &String;

    fn set_text(&mut self, text: &String);

    fn get_runs(&self) -> Vec<TextAnalyzerRun>;

    fn get_glyphs_and_positions(
        &self,
        text_range: Range<usize>,
        run: TextAnalyzerRun,
        font: &Font,
    ) -> TextAnalyzerGlyphRun;

}

// endregion:
