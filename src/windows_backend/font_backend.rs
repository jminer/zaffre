use std::ffi::{OsStr, OsString};
use std::{mem, iter};
use std::os::windows::prelude::OsStringExt;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use smallvec::SmallVec;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Graphics::Gdi::{LOGFONTW, FIXED_PITCH};
use windows::core::Interface;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED, IDWriteFontFamily, IDWriteFont, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_OBLIQUE, IDWriteFontFace, IDWriteFont1, IDWriteFont2, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STRETCH_NORMAL, IDWriteLocalizedStrings, DWRITE_FONT_STYLE, DWRITE_FONT_WEIGHT, DWRITE_FONT_STRETCH,
};

use crate::font::{FontFamily, OpenTypeFontWeight, FontSlant, OpenTypeFontWidth, Font, FontDescription};
use crate::generic_backend::{GenericFontFamilyBackend, GenericFontDescriptionBackend, GenericFontFunctionsBackend, GenericFontBackend};

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
}
