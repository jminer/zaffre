use std::rc::Rc;


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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTypeFontWeight(pub u32);

impl From<FontWeight> for OpenTypeFontWeight {
    fn from(weight: FontWeight) -> Self {
        Self(weight as u32)
    }
}

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
// in Skia, DirectWrite, or Core Text. A QFont can be created with a family name that doesn't exist,
// but the other libraries make you query and only create an object if you get a match. DirectWrite
// and Core Text seem to have a font metadata object and an actual font object:
// IDWriteFont/IDWriteFontFace and CTFontDescriptor/CTFont. Skia doesn't seem to have a
// IDWriteFont/CTFontDescriptor like object, and I don't see why it is required. I'd like to try to
// get by without it.

#[derive(Debug, Clone)]
pub struct Font(Rc<FontData>);

impl Font {

}



// Names for this object:
// DirectWrite: FontCollection
// Core Text: FontCollection
// Skia: SkFontMgr

#[derive(Debug, Clone)]
struct FontCollectionData {

}

#[derive(Debug, Clone)]
pub struct FontCollection(Rc<FontCollectionData>);

impl FontCollection {
    fn system() -> Self {
        Self(Rc::new(todo!()))
    }
}

pub struct FontFamily {

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
