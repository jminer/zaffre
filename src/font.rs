use glam::Affine2;
use nalgebra::Point2;
use smallvec::SmallVec;

use crate::{Rect, Size2};
use crate::backend::font_backend::{FontFamilyBackend, FontDescriptionBackend, FontFunctionsBackend, FontBackend, GlyphImageBackend};
use crate::generic_backend::{GenericFontFamilyBackend, GenericFontDescriptionBackend, GenericFontFunctionsBackend, GenericFontBackend, GenericGlyphImageBackend};

// Font.
//
// A collection of letters, numbers, punctuation, and other symbols used to set text (or related)
// matter. Although font and typeface are often used interchangeably, font refers to the physical
// embodiment (whether it’s a case of metal pieces or a computer file) while typeface refers to the
// design (the way it looks). A font is what you use, and a typeface is what you see.
//
// Style.
//
// Any given variant in a type family; the equivalent of a single font or typeface.
//
// Typeface.
//
// An artistic interpretation, or design, of a collection of alphanumeric symbols. A typeface may
// include letters, numerals, punctuation, various symbols, and more — often for multiple languages.
// A typeface is usually grouped together in a family containing individual fonts for italic, bold,
// condensed, and other variations of the primary design. Even though its original meaning is one
// single style of a type design, the term is now also commonly used to describe a type family
// (usually only with the basic styles regular, italic, bold, bold italic).
//
// - https://www.monotype.com/resources/studio/typography-terms

// A typeface is the design of the letters. A font is the carved wood or metal pieces for the
// printing press. A style is a design variant in a type family.
//
// "Font face" doesn't appear as a term at all.

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

// DWrite, Pango, and CSS use the word "style", Core Text, cairo, and Skia use "slant", and the
// OpenType spec refers to it as "slope" a couple places.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSlant {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWidth {
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
pub struct OpenTypeFontWidth(pub u32);

impl From<FontWidth> for OpenTypeFontWidth {
    fn from(width: FontWidth) -> Self {
        Self(width as u32)
    }
}

// Implementation notes: Qt has a QFont and QRawFont, and I think a QFont isn't really like anything
// in Skia or DirectWrite. A QFont can be created with a family name that doesn't exist, but the
// other libraries make you query and only create an object if you get a match. DirectWrite and Core
// Text seem to have a font metadata object and an actual font object: IDWriteFont/IDWriteFontFace,
// CTFontDescriptor/CTFont, and PangoFontFace/PangoFont. Skia doesn't seem to have a
// IDWriteFont/CTFontDescriptor like object. Maybe a CTFontDescriptor, PangoFontDescription, and
// QFont are all similar.


pub fn get_families() -> Vec<FontFamily> {
    FontFunctionsBackend::get_families()
}

pub fn get_family(name: &str) -> Option<FontFamily> {
    FontFunctionsBackend::get_family(name)
}

pub fn get_matching_font<T, U>(
    family: &str,
    weight: T,
    slant: FontSlant,
    width: U,
    size: f32,
) -> Option<Font>
where
    T: Into<OpenTypeFontWeight>,
    U: Into<OpenTypeFontWidth>,
{
    get_family(family).map(|f| f.get_matching_font(weight, slant, width, size))
}


pub struct FontFamily<B: GenericFontFamilyBackend = FontFamilyBackend> {
    pub(crate) backend: B,
}

impl<B: GenericFontFamilyBackend> FontFamily<B> {
    pub fn get_family_name(&self) -> String { // TODO: should this be get_name()?
        self.backend.get_name()
    }

    pub fn get_styles(&self) -> Vec<FontDescription> {
        self.backend.get_styles()
    }

    pub fn get_matching_font<T, U>(&self, weight: T, slant: FontSlant, width: U, size: f32) -> Font
    where
        T: Into<OpenTypeFontWeight>,
        U: Into<OpenTypeFontWidth>,
    {
        self.backend.get_matching_font(weight.into(), slant, width.into(), size)
    }
}

/// The description of a font face.
pub struct FontDescription<B: GenericFontDescriptionBackend = FontDescriptionBackend> {
    pub(crate) backend: B,
}

impl<B: GenericFontDescriptionBackend> FontDescription<B> {
    pub fn get_family_name(&self) -> String {
        self.backend.get_family_name()
    }

    // DWrite and Pango call it the "face name", and Core Text calls it the "style name".

    pub fn get_style_name(&self) -> String {
        self.backend.get_style_name()
    }

    pub fn weight(&self) -> OpenTypeFontWeight {
        self.backend.weight()
    }

    pub fn slant(&self) -> FontSlant {
        self.backend.slant()
    }

    pub fn width(&self) -> OpenTypeFontWidth {
        self.backend.width()
    }

    pub fn is_monospaced(&self) -> bool {
        self.backend.is_monospaced()
    }

    pub fn has_color_glyphs(&self) -> bool {
        self.backend.has_color_glyphs()
    }

    pub fn get_font(&self, size: f32) -> Font {
        // Core Text, Pango, Skia, System.Drawing.Font, and QFont all put the size in their Font
        // type. DirectWrite seems to be the only one that doesn't. It instead has a separate
        // parameter to GetGlyphPlacements().
        self.backend.get_font(size)
    }
}

#[derive(Debug, Clone)]
pub struct Font<B: GenericFontBackend = FontBackend> {
    pub(crate) backend: B,
}

impl Font {
    pub fn size(&self) -> f32 {
        self.backend.size()
    }

    pub fn description(&self) -> FontDescription {
        self.backend.description()
    }

    // This function is mainly for debugging and testing purposes. For proper text rendering, you
    // need to use a shaping engine to generate the glyphs for a string.
    pub fn get_glyph(&self, c: char) -> u16 {
        self.backend.get_glyph(c)
    }

    pub fn draw_glyphs(
        &self,
        glyphs: &[u16],
        offsets: &[Point2<f32>],
        transform: Affine2,
    ) -> SmallVec<[GlyphImage; 32]> {
        // It seems like it's possible to get a glyph rendered into app-chosen memory using
        // IDWriteGlyphRunAnalysis::CreateAlphaTexture() and CGBitmapContextCreate(). However,
        // FreeType doesn't support it. It only renders to an FT_Bitmap, which you can then access
        // its buffer pointer. So it's easy to access the bitmap memory, but you can't allocate it.
        //
        // This function returns a GlyphImage so that the caller can submit a GPU command to copy it
        // to a GPU image (font atlas), and when the command is finished, the GPU can notify the CPU
        // and the GlyphImage can be dropped.
        self.backend.draw_glyphs(glyphs, offsets, transform)
    }

    // For each glyph, gets the bounding rectangle of the glyph's shape, relative to the glyphs's
    // layout box. Microsoft calls the bounding rectangle the "black box."
    //pub fn get_glyph_bounding_rects(&self, glyphs: &[u16]) -> Vec<Rect<f32>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FontAntialiasing {
    None,
    Grayscale,
    Subpixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GlyphImageFormat {
    // Each pixel has an alpha value stored in one byte.
    Alpha1x1,
    // Each pixel has alpha values stored in three bytes, used for subpixel antialiasing.
    Alpha3x1,
    // Each pixel has a full 32-bit color image (used for rendering emoji).
    RgbaColor,
    // Each pixel has a full 32-bit color image (used for rendering emoji).
    BgraColor,
}

impl GlyphImageFormat {
    pub(crate) fn pixel_size(self) -> usize {
        match self {
            GlyphImageFormat::Alpha1x1 => 1,
            GlyphImageFormat::Alpha3x1 => 3,
            GlyphImageFormat::RgbaColor => 4,
            GlyphImageFormat::BgraColor => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlyphImage<B: GenericGlyphImageBackend = GlyphImageBackend> {
    pub(crate) format: GlyphImageFormat,
    pub(crate) data_ptr: *mut u8,
    // The data pointer points to the top-left corner of the image, so only a size is needed.
    pub(crate) bounding_size: Size2<u32>, // in pixels
    pub(crate) stride: u32, // in pixels
    pub(crate) baseline_origin: Point2<f32>,
    pub(crate) backend: B,
}

impl<B: GenericGlyphImageBackend> GlyphImage<B> {
    pub(crate) fn format(&self) -> GlyphImageFormat {
        self.format
    }

    pub(crate) fn data_ptr(&self) -> *mut u8 {
        self.data_ptr
    }

    pub(crate) fn bounding_size(&self) -> Size2<u32> {
        self.bounding_size
    }

    // in pixels
    pub(crate) fn stride(&self) -> u32 {
        self.stride
    }

    pub(crate) fn baseline_origin(&self) -> Point2<f32> {
        self.baseline_origin
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
