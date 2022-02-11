
use std::fmt::Debug;

use crate::font::{OpenTypeFontWeight, FontStyle, OpenTypeFontStretch, FontFamily};

pub trait GenericFontCollectionBackend: Debug + Clone {
    fn system() -> Self;

    //fn get_families(&self) -> Vec<FontFamily>;
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
