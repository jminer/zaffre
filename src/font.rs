use std::rc::Rc;

use crate::backend::font_backend::{FontFamilyBackend, FontDescriptionBackend, FontFunctionsBackend};
use crate::generic_backend::{GenericFontFamilyBackend, GenericFontDescriptionBackend, GenericFontFunctionsBackend};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
    ExtraBlack = 950,
}

// usWeightClass in the OpenType OS/2 table is a u16.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTypeFontWeight(pub u32);

impl From<FontWeight> for OpenTypeFontWeight {
    fn from(weight: FontWeight) -> Self {
        Self(weight as u32)
    }
}

// DWrite, Pango, and CSS use the word "style", Core Text and cairo use "slant", and the OpenType
// spec refers to it as "slope" a couple places.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStretch {
    UltraCondensed = 1,
    ExtraCondensed = 2,
    Condensed = 3,
    SemiCondensed = 4,
    Normal = 5,
    SemiExpanded = 6,
    Expanded = 7,
    ExtraExpanded = 8,
    UltraExpanded = 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTypeFontStretch(pub u32);

impl From<FontStretch> for OpenTypeFontStretch {
    fn from(stretch: FontStretch) -> Self {
        Self(stretch as u32)
    }
}

#[derive(Debug, Clone)]
pub struct FontData {
    family: String,
    weight: OpenTypeFontWeight,
    style: FontStyle,
    stretch: OpenTypeFontStretch,
    size: f32,
}

// Implementation notes: Qt has a QFont and QRawFont, and I think a QFont isn't really like anything
// in Skia or DirectWrite. A QFont can be created with a family name that doesn't exist, but the
// other libraries make you query and only create an object if you get a match. DirectWrite and Core
// Text seem to have a font metadata object and an actual font object: IDWriteFont/IDWriteFontFace,
// CTFontDescriptor/CTFont, and PangoFontFace/PangoFont. Skia doesn't seem to have a
// IDWriteFont/CTFontDescriptor like object. Maybe a CTFontDescriptor, PangoFontDescription, and
// QFont are all similar.

#[derive(Debug, Clone)]
pub struct Font(Rc<FontData>);

impl Font {

}



pub fn get_families() -> Vec<FontFamily> {
    FontFunctionsBackend::get_families()
}

pub fn get_family(name: &str) -> Option<FontFamily> {
    FontFunctionsBackend::get_family(name)
}

pub fn get_matching_font(
    family: &str,
    weight: OpenTypeFontWeight,
    style: FontStyle,
    stretch: OpenTypeFontStretch,
) -> Option<Font> {
    get_family(family).map(|f| f.get_matching_font(weight, style, stretch))
}

pub struct FontFamily<B: GenericFontFamilyBackend = FontFamilyBackend> {
    pub(crate) backend: B,
}

impl<B: GenericFontFamilyBackend> FontFamily<B> {
    pub fn get_family_name(&self) -> String { // TODO: should this be get_name()?
        self.backend.get_name()
    }

    pub fn get_matching_font(&self,
        weight: OpenTypeFontWeight,
        style: FontStyle,
        stretch: OpenTypeFontStretch,
    ) -> Font {
        todo!()
    }
}

/// The description of a font face.
pub struct FontDescription<B: GenericFontDescriptionBackend = FontDescriptionBackend> {
    backend: B,
}

impl<B: GenericFontDescriptionBackend> FontDescription<B> {

    // DWrite and Pango call it the "face name", and Core Text calls it the "style name". I like
    // "style name" better, but using "style" for italic/oblique makes it confusing.

    fn get_face_name(&self) -> String {
        self.backend.get_face_name()
    }

    fn weight(&self) -> OpenTypeFontWeight {
        self.backend.weight()
    }

    fn style(&self) -> FontStyle {
        self.backend.style()
    }

    fn stretch(&self) -> OpenTypeFontStretch {
        self.backend.stretch()
    }

    fn is_monospaced(&self) -> bool {
        self.backend.is_monospaced()
    }

    fn has_color_glyphs(&self) -> bool {
        self.backend.has_color_glyphs()
    }
}



// To render a font with a certain family, style, and size, the app should have to create a font
// object for it. The font atlas can be stored in the object, so when the app has control over
// freeing the font atlas by dropping the object. Rendering the font rotated, etc. just would not
// populate the atlas any. I'm not sure how to handle scaling, but probably just don't use the
// atlas either?

// Actually, storing the atlas in a Font object is a bad plan. It should probably be global. If you
// have a number of controls like header labels that all have a different font than the main
// interface font but are the same as each other, they should share a font atlas even though they
// have to have separate font objects.
