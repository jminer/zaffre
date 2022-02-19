
use std::fmt::Debug;

use crate::font::{OpenTypeFontWeight, FontStyle, OpenTypeFontStretch, FontFamily, Font};

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
}

pub trait GenericFontDescriptionBackend: Debug + Clone {
    fn get_face_name(&self) -> String;

    fn weight(&self) -> OpenTypeFontWeight;

    fn style(&self) -> FontStyle;

    fn stretch(&self) -> OpenTypeFontStretch;

    fn is_monospaced(&self) -> bool;

    fn has_color_glyphs(&self) -> bool;
}
