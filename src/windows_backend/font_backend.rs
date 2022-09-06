use std::ffi::{OsStr, OsString};
use std::rc::Rc;
use std::{mem, iter, ptr};
use std::os::windows::prelude::OsStringExt;
use std::sync::Mutex;

use glam::Affine2;
use nalgebra::Point2;
use once_cell::sync::Lazy;
use smallvec::SmallVec;
use windows::Win32::Foundation::{BOOL, RECT};
use windows::Win32::Graphics::Gdi::{LOGFONTW, FIXED_PITCH};
use windows::core::Interface;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED, IDWriteFontFamily, IDWriteFont, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_OBLIQUE, IDWriteFontFace, IDWriteFont1, IDWriteFont2, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STRETCH_NORMAL, IDWriteLocalizedStrings, DWRITE_FONT_STYLE, DWRITE_FONT_WEIGHT, DWRITE_FONT_STRETCH, DWRITE_GLYPH_OFFSET, DWRITE_GLYPH_RUN, DWRITE_MEASURING_MODE_NATURAL, DWRITE_RENDERING_MODE_NATURAL, DWRITE_TEXTURE_CLEARTYPE_3x1, DWRITE_MATRIX, DWRITE_GLYPH_METRICS, DWRITE_TEXTURE_ALIASED_1x1, IDWriteFactory2, DWRITE_GRID_FIT_MODE_DISABLED, DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE, DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE, DWRITE_RENDERING_MODE_ALIASED,
};

use crate::{Size2, Rect};
use crate::font::{FontFamily, OpenTypeFontWeight, FontSlant, OpenTypeFontWidth, Font, FontDescription, GlyphImage, GlyphImageFormat, FontAntialiasing};
use crate::generic_backend::{GenericFontFamilyBackend, GenericFontDescriptionBackend, GenericFontFunctionsBackend, GenericFontBackend, GenericGlyphImageBackend};

use super::wide_ffi_string::WideFfiString;

// There is a one-to-one correspondence between an IDWriteFont and IDWriteFontFace.
// - IDWriteFont -> IDWriteFontFace using IDWriteFont::CreateFontFace()
// - IDWriteFontFace -> IDWriteFont using IDWriteFontCollection::GetFontFromFontFace()
//
// I think the difference is that an IDWriteFont is for listing and selecting fonts, and it carries
// the metadata like name, weight, style, and overall font metrics but no glyphs. An IDWriteFontFace
// loads the font glyphs and allows getting glyph metrics, glyph indicies, and OpenType font table
// data.

// If IDWriteFactory were Send (bug in the windows crate?), this could be a global protected by a
// Mutex.
thread_local! {
    pub(crate) static DWRITE_FACTORY: IDWriteFactory = unsafe {
        mem::transmute(DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory::IID)
            .expect("failed to create IDWriteFactory"))
    }
}

fn get_dwrite_system_collection() -> IDWriteFontCollection {
    unsafe {
        DWRITE_FACTORY.with(|factory| {
            let mut font_collection = None;
            factory.GetSystemFontCollection(&mut font_collection, false)
                .expect("GetSystemFontCollection() failed");
            font_collection.unwrap() // can't be None if the expect() above passes
        })
    }
}

fn to_dwrite_style(style: FontSlant) -> DWRITE_FONT_STYLE {
    match style {
        FontSlant::Normal => DWRITE_FONT_STYLE_NORMAL,
        FontSlant::Italic => DWRITE_FONT_STYLE_ITALIC,
        FontSlant::Oblique => DWRITE_FONT_STYLE_OBLIQUE,
    }
}

fn from_dwrite_style(style: DWRITE_FONT_STYLE) -> FontSlant {
    match style {
        DWRITE_FONT_STYLE_NORMAL => FontSlant::Normal,
        DWRITE_FONT_STYLE_ITALIC => FontSlant::Italic,
        DWRITE_FONT_STYLE_OBLIQUE => FontSlant::Oblique,
        _ => FontSlant::Normal,
    }
}

struct DWriteLocalizedStringsUtil(IDWriteLocalizedStrings);

impl DWriteLocalizedStringsUtil {
    fn find_locale(&self, name: &str) -> Option<u32> {
        unsafe {
            let mut index = 0;
            let mut exists = BOOL(0);
            let wide_locale = WideFfiString::<[u16; 32]>::new(name);
            self.0.FindLocaleName(&wide_locale, &mut index, &mut exists)
                .expect("FindLocaleName() failed");
            return if exists.as_bool() {
                Some(index)
            } else {
                None
            };
        }
    }

    fn get_string(&self, index: u32) -> String {
        unsafe {
            let name_len = self.0.GetStringLength(index)
                .expect("GetStringLength() failed") as usize;
            // +1 for null term
            let mut name_buf = SmallVec::<[u16; 32]>::from_elem(0, name_len + 1);
            // I don't really like that the windows crate now takes a reference, not to
            // MaybeUnit<u16>, so now it requires that the memory is initialized.
            self.0.GetString(index, &mut name_buf).expect("GetString() failed");
            name_buf.set_len(name_len);
            OsString::from_wide(&name_buf).to_string_lossy().into_owned()
        }
    }
}

pub(crate) struct FontFunctionsBackend;
impl GenericFontFunctionsBackend for FontFunctionsBackend {
    fn get_families() -> Vec<crate::font::FontFamily> {
        unsafe {
            let collection = get_dwrite_system_collection();
            let count = collection.GetFontFamilyCount() as usize;
            let mut families = Vec::with_capacity(count);
            for i in 0..count {
                let font_family = collection.GetFontFamily(i as u32);
                if let Ok(font_family) = font_family {
                    families.push(FontFamily {
                        backend: FontFamilyBackend {
                            family: font_family,
                        },
                    });
                }
            }
            families
        }
    }

    fn get_family(name: &str) -> Option<crate::font::FontFamily> {
        unsafe {
            let collection = get_dwrite_system_collection();
            let mut index = 0;
            let mut exists = BOOL(0);
            let wide_name = WideFfiString::<[u16; 32]>::new(name);
            if collection.FindFamilyName(&wide_name, &mut index, &mut exists).is_err() {
                return None;
            }
            if !exists.as_bool() {
                return None;
            }
            let family = match collection.GetFontFamily(index) {
                Ok(f) => f,
                Err(_) => return None,
            };
            Some(FontFamily {
                backend: FontFamilyBackend { family }
            })
        }
    }
}


#[derive(Debug, Clone)]
pub struct FontFamilyBackend {
    family: IDWriteFontFamily,
}

impl GenericFontFamilyBackend for FontFamilyBackend {
    fn get_name(&self) -> String {
        unsafe {
            let names = DWriteLocalizedStringsUtil(self.family.GetFamilyNames()
                .expect("GetFamilyNames() failed"));
            let index = names.find_locale("en-us").unwrap_or(0);
            names.get_string(index)
        }
    }

    fn get_styles(&self) -> Vec<FontDescription> {
        unsafe {
            let mut styles = Vec::new();
            let font_list = self.family.GetMatchingFonts(
                DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL)
                .expect("IDWriteFontFamily.GetMatchingFonts() failed");
            for i in 0..font_list.GetFontCount() {
                let dwrite_font = font_list.GetFont(i)
                    .expect("IDWriteFontList.GetFont() failed");
                styles.push(FontDescription {
                    backend: FontDescriptionBackend {
                        font_desc: dwrite_font,
                    }
                });
            }
            styles
        }
    }

    fn get_matching_font(&self,
        weight: OpenTypeFontWeight,
        style: FontSlant,
        width: OpenTypeFontWidth,
        size: f32,
    ) -> Font {
        let dwrite_style = to_dwrite_style(style);
        unsafe {
            let dwrite_font = self.family.GetFirstMatchingFont(
                    DWRITE_FONT_WEIGHT(weight.0 as i32),
                    DWRITE_FONT_STRETCH(width.0 as i32),
                    dwrite_style
                ).expect("GetFirstMatchingFont() failed");
            let font_face = dwrite_font.CreateFontFace().expect("CreateFontFace() failed");
            Font {
                backend: FontBackend {
                    font_face,
                    size,
                }
            }
        }
    }


}

#[derive(Debug, Clone)]
pub struct FontDescriptionBackend {
    font_desc: IDWriteFont,
}

impl GenericFontDescriptionBackend for FontDescriptionBackend {
    fn get_family_name(&self) -> String {
        unsafe {
            let family = self.font_desc.GetFontFamily().expect("GetFontFamily() failed");
            let names = DWriteLocalizedStringsUtil(family.GetFamilyNames()
                .expect("GetFamilyNames() failed"));
            let index = names.find_locale("en-us").unwrap_or(0);
            names.get_string(index)
        }
    }

    fn get_style_name(&self) -> String {
        unsafe {
            let names = DWriteLocalizedStringsUtil(self.font_desc.GetFaceNames()
                .expect("GetFaceNames() failed"));
            let index = names.find_locale("en-us").unwrap_or(0);
            names.get_string(index)
        }
    }

    fn weight(&self) -> OpenTypeFontWeight {
        unsafe {
            OpenTypeFontWeight(self.font_desc.GetWeight().0 as u32)
        }
    }

    fn slant(&self) -> FontSlant {
        unsafe {
            from_dwrite_style(self.font_desc.GetStyle())
        }
    }

    fn width(&self) -> OpenTypeFontWidth {
        unsafe {
            OpenTypeFontWidth(self.font_desc.GetStretch().0 as u32)
        }
    }

    fn is_monospaced(&self) -> bool {
        // IDWriteFont1 was added in Windows 8 and an update for Windows 7
        let font1 = self.font_desc.cast::<IDWriteFont1>();
        if let Ok(font1) = font1 {
            return unsafe { font1.IsMonospacedFont().as_bool() };
        }
        unsafe {
            DWRITE_FACTORY.with(|factory| {
                let gdi_interop = factory.GetGdiInterop().expect("GetGdiInterop() failed");
                let mut logfont: LOGFONTW = mem::zeroed();
                let mut is_system: BOOL = BOOL(0);
                gdi_interop.ConvertFontToLOGFONT(&self.font_desc, &mut logfont, &mut is_system)
                    .expect("ConvertFontToLOGFONT() failed");
                logfont.lfPitchAndFamily & 0x3 == FIXED_PITCH as u8
            })
        }
    }

    fn has_color_glyphs(&self) -> bool {
        // IDWriteFont2 was added in Windows 8.1
        let font2 = self.font_desc.cast::<IDWriteFont2>();
        if let Ok(font2) = font2 {
            return unsafe { font2.IsColorFont().as_bool() };
        }
        false
    }

    fn get_font(&self, size: f32) -> Font {
        unsafe {
            let font_face = self.font_desc.CreateFontFace().expect("CreateFontFace() failed");
            Font {
                backend: FontBackend {
                    font_face,
                    size,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FontBackend {
    pub(crate) font_face: IDWriteFontFace,
    pub(crate) size: f32,
}

impl GenericFontBackend for FontBackend {
    fn size(&self) -> f32 {
        self.size
    }

    fn description(&self) -> FontDescription {
        unsafe {
            let collection = get_dwrite_system_collection();
            let font_desc = collection.GetFontFromFontFace(&self.font_face)
                .expect("GetFontFromFontFace() failed");
            FontDescription {
                backend: FontDescriptionBackend {
                    font_desc
                }
            }
        }
    }

    fn get_glyph(&self, c: char) -> u16 {
        unsafe {
            let mut glyph: u16 = 0;
            self.font_face.GetGlyphIndices(&(c as u32), 1, &mut glyph)
                .expect("GetGlyphIndices() failed");
            glyph
        }
    }

    fn draw_glyphs(
        &self,
        glyphs: &[u16],
        offsets: &[nalgebra::Point2<f32>],
        transform: glam::Affine2,
    ) -> SmallVec<[GlyphImage; 32]> {
        // According to Skia source, DWRITE_TEXTURE_ALIASED_1x1 is now misnamed and must be used
        // with grayscale antialiasing too.
        //
        // I think on Windows 7, I may turn off antialiasing since DWrite doesn't support grayscale
        // AA. Or I could render ClearType and convert the image.

        // TODO: get design metrics and allocate a GlyphImage for each glyph,
        // then group them by which GlyphImages are in the same slab, and render all of those in
        // one CreateGlyphRunAnalysis() and CreateAlphaTexture() call.
        unsafe {
            // Memory allocators are pretty fast. For now, let's try just allocating the memory we
            // need instead of using the GlyphImageSlab. And then we can render all glyphs in one
            // call.
            let antialiasing = FontAntialiasing::Grayscale;

            let mut font_metrics = Default::default();
            self.font_face.GetMetrics(&mut font_metrics);

            // I think the design glyph metrics might not exactly match rendered glyphs because of
            // hinting. font-kit creates a GlyphRunAnalysis for each glyph and calls
            // GetAlphaTextureBounds(). However, if we round up, we should be safe to use the design
            // metrics. It really doesn't matter if the returned glyph image is slightly larger than
            // the drawn pixels. And we save time not creating a bunch of extra GlyphRunAnalysis
            // objects.
            let mut glyph_metrics =
                SmallVec::<[DWRITE_GLYPH_METRICS; 32]>::with_capacity(glyphs.len());
            self.font_face.GetDesignGlyphMetrics(
                glyphs.as_ptr(), glyphs.len() as u32, glyph_metrics.as_mut_ptr(), false
            ).expect("GetDesignGlyphMetrics() failed");
            glyph_metrics.set_len(glyphs.len());

            let mut glyph_advances = SmallVec::<[f32; 32]>::new();
            glyph_advances.resize(glyphs.len(), 0.0);

            let mut max_width = 0u32;
            let mut y = 0u32;
            let design_units_to_px = |x|
                x as f32 / font_metrics.designUnitsPerEm as f32 * self.size();
            let mut glyph_offsets =
                SmallVec::<[DWRITE_GLYPH_OFFSET; 32]>::with_capacity(glyphs.len());
            let mut glyph_images = SmallVec::<[GlyphImage; 32]>::with_capacity(glyphs.len());
            glyph_images.resize(glyphs.len(), GlyphImage {
                format: GlyphImageFormat::Alpha1x1,
                data_ptr: ptr::null_mut(),
                bounding_size: Size2::new(0, 0),
                stride: 0,
                baseline_origin: Point2::new(0.0, 0.0),
                backend: GlyphImageBackend { data: Default::default() },
            });
            dbg!(offsets);
            let mut i = 0;
            for metrics in glyph_metrics {
                let offset = offsets[i];
                assert!(offset.x < 1.0);
                assert!(offset.y < 1.0);
                dbg!(metrics);
                dbg!(font_metrics);

                // https://docs.microsoft.com/en-us/windows/win32/directwrite/glyphs-and-glyph-runs#glyph-metrics
                let advance_offset = f32::ceil(-design_units_to_px(metrics.leftSideBearing));
                // The vertical origin can be pretty different than the ascent.
                let ascender_offset = f32::ceil(
                    design_units_to_px(metrics.verticalOriginY - metrics.topSideBearing)
                );

                // let width = f32::ceil(
                //     design_units_to_px(metrics.advanceWidth as i32 - horiz_bearing_du)
                // ) as u32;
                // let height = f32::ceil(
                //     self.size() - design_units_to_px(vert_bearing_du)
                // ) as u32;

                // These are calculated based on advance_offset and ascender_offset so that their
                // rounding is also included in the width and height.
                let width = f32::ceil(advance_offset + design_units_to_px(metrics.advanceWidth as i32 - metrics.rightSideBearing)) as u32;
                let height = f32::ceil(ascender_offset + design_units_to_px(metrics.advanceHeight as i32 - metrics.verticalOriginY - metrics.bottomSideBearing)) as u32;

                max_width = max_width.max(width);

                glyph_offsets.push(DWRITE_GLYPH_OFFSET {
                    advanceOffset: advance_offset + offset.x,
                    ascenderOffset: -(y as f32 + ascender_offset) + offset.y,
                });
                glyph_images[i].data_ptr = y as *mut u8; // kind of hacky, but more efficient
                glyph_images[i].bounding_size = Size2::new(width, height);
                glyph_images[i].baseline_origin = Point2::new(advance_offset, ascender_offset);
                y += height;

                i += 1;
            }
            dbg!(&glyph_offsets);

            let glyph_run = DWRITE_GLYPH_RUN {
                fontFace: Some(self.font_face.clone()),
                fontEmSize: self.size(),
                glyphCount: glyphs.len() as u32,
                glyphIndices: glyphs.as_ptr(),
                glyphAdvances: glyph_advances.as_ptr(),
                glyphOffsets: glyph_offsets.as_ptr(),
                isSideways: false.into(),
                bidiLevel: 0,
            };
            let rendering_mode = if antialiasing == FontAntialiasing::None {
                DWRITE_RENDERING_MODE_ALIASED
            } else {
                DWRITE_RENDERING_MODE_NATURAL
            };
            let mut aa_mode = if antialiasing == FontAntialiasing::Subpixel {
                DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE
            } else {
                DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE
            };
            let glyph_run_analysis = DWRITE_FACTORY.with(|factory| {
                // let factory2 = factory.cast::<IDWriteFactory2>();
                // if let Ok(factory2) = factory2 {
                //     factory2.CreateGlyphRunAnalysis2(
                //         &glyph_run,
                //         &to_dwrite_matrix(transform),
                //         rendering_mode,
                //         DWRITE_MEASURING_MODE_NATURAL,
                //         DWRITE_GRID_FIT_MODE_DISABLED,
                //         aa_mode,
                //         0.0,
                //         0.0
                //     ).expect("CreateGlyphRunAnalysis() failed")
                // } else {
                    // Windows 7 DWrite only supported ClearType.
                    aa_mode = DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE;
                    factory.CreateGlyphRunAnalysis(
                        &glyph_run,
                        1.0,
                        &to_dwrite_matrix(transform),
                        rendering_mode,
                        DWRITE_MEASURING_MODE_NATURAL,
                        0.0,
                        0.0
                    ).expect("CreateGlyphRunAnalysis() failed")
                // }
            });

            // This loop is kind of a second part of the one above, filling out the rest of the
            // GlyphImage structs. It's just down here because it depends on what AA this version of
            // DWrite supports.
            let aa_width = if aa_mode == DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE { 3 } else { 1 };
            let mut data: Vec<u8> = Vec::new();
            data.resize((max_width * y) as usize * aa_width, 0);
            let data_ptr = data.as_mut_ptr();
            let data = Rc::new(data);
            for image in &mut glyph_images {
                image.format = if aa_mode == DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE {
                    GlyphImageFormat::Alpha3x1
                } else {
                    GlyphImageFormat::Alpha1x1
                };
                image.stride = max_width;
                let y = image.data_ptr as usize;
                image.data_ptr = data_ptr.add(
                    max_width as usize * y * image.format.pixel_size());
                image.backend.data = data.clone();
            }

            let texture_type = match rendering_mode {
                DWRITE_RENDERING_MODE_ALIASED => DWRITE_TEXTURE_ALIASED_1x1,
                DWRITE_RENDERING_MODE_NATURAL | _ => DWRITE_TEXTURE_CLEARTYPE_3x1,
            };
            let bounds = RECT {
                left: 0,
                right: max_width as i32,
                top: 0,
                bottom: y as i32,
            };
            dbg!(glyph_run_analysis.GetAlphaTextureBounds(texture_type));
            glyph_run_analysis.CreateAlphaTexture(
                texture_type, &bounds, data_ptr, data.capacity() as u32
            ).expect("CreateAlphaTexture() failed");
            //dbg!(&data);

            // On Windows 7, convert subpixel AA to grayscale if needed.
            if antialiasing == FontAntialiasing::Grayscale &&
                aa_mode == DWRITE_TEXT_ANTIALIAS_MODE_CLEARTYPE
            {
                for glyph_image in &mut glyph_images {
                    if glyph_image.format == GlyphImageFormat::Alpha3x1 {
                        for y in 0..glyph_image.bounding_size.height {
                            let mut src_ptr = glyph_image.data_ptr
                                .add((y * glyph_image.stride * 3) as usize);
                            let mut dest_ptr = glyph_image.data_ptr
                                .add((y * glyph_image.stride) as usize);
                            for _ in 0..glyph_image.bounding_size.width {
                                let alpha_sum =
                                    *src_ptr.add(0) as u32 +
                                    *src_ptr.add(1) as u32 +
                                    *src_ptr.add(2) as u32;
                                *dest_ptr = (alpha_sum / 3) as u8;

                                src_ptr = src_ptr.add(3);
                                dest_ptr = dest_ptr.add(1);
                            }
                        }
                        glyph_image.format = GlyphImageFormat::Alpha1x1;
                    }
                }
            }

            glyph_images
        }
    }

}


#[derive(Debug, Clone)]
pub struct GlyphImageBackend {
    data: Rc<Vec<u8>>,
}

impl GenericGlyphImageBackend for GlyphImageBackend {
}

fn to_dwrite_matrix(transform: Affine2) -> DWRITE_MATRIX {
    let m = transform.to_cols_array();
    DWRITE_MATRIX { m11: m[0], m12: m[1], m21: m[2], m22: m[3], dx: m[4], dy: m[5] }
}
