use std::ffi::{OsStr, OsString};
use std::mem;
use std::os::windows::prelude::OsStringExt;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use smallvec::SmallVec;
use windows::Win32::Foundation::{PWSTR, BOOL};
use windows::core::Interface;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED, IDWriteFontFamily, IDWriteFont,
};

use crate::ffi_string::WideFfiString;
use crate::font::FontFamily;
use crate::generic_backend::{GenericFontFamilyBackend, GenericFontDescriptionBackend, GenericFontFunctionsBackend};

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

pub(crate) struct FontFunctionsBackend;
impl GenericFontFunctionsBackend for FontFunctionsBackend {
    fn get_families() -> Vec<crate::font::FontFamily> {
        unsafe {
            let collection = DWRITE_FACTORY.with(|factory| {
                let mut font_collection = None;
                factory.GetSystemFontCollection(&mut font_collection, false)
                    .expect("GetSystemFontCollection() failed");
                font_collection.unwrap() // can't be None if the expect() above passes
            });
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
            let collection = DWRITE_FACTORY.with(|factory| {
                let mut font_collection = None;
                factory.GetSystemFontCollection(&mut font_collection, false)
                    .expect("GetSystemFontCollection() failed");
                font_collection.unwrap() // can't be None if the expect() above passes
            });
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
            let names = self.family.GetFamilyNames()
                .expect("GetFamilyNames() failed");
            let mut index = 0;
            let mut exists = BOOL(0);
            let wide_locale = WideFfiString::<[u16; 32]>::new("en-us");
            names.FindLocaleName(&wide_locale, &mut index, &mut exists)
                .expect("FindLocaleName() failed");
            if !exists.as_bool() {
                index = 0;
            }
            let name_len = names.GetStringLength(index).expect("GetStringLength() failed") as usize;
            let mut name_buf = SmallVec::<[u16; 32]>::new();
            name_buf.reserve_exact(name_len + 1); // +1 for null term
            names.GetString(index, PWSTR(name_buf.as_mut_ptr()), name_buf.capacity() as u32)
                .expect("GetString() failed");
            name_buf.set_len(name_len);
            OsString::from_wide(&name_buf).to_string_lossy().into_owned()
        }
    }
}


#[derive(Debug, Clone)]
pub struct FontDescriptionBackend {
    font_desc: IDWriteFont,
}

impl GenericFontDescriptionBackend for FontDescriptionBackend {
    fn get_face_name(&self) -> String {
        todo!()
    }

    fn weight(&self) -> crate::font::OpenTypeFontWeight {
        todo!()
    }

    fn style(&self) -> crate::font::FontStyle {
        todo!()
    }

    fn stretch(&self) -> crate::font::OpenTypeFontStretch {
        todo!()
    }

    fn is_monospaced(&self) -> bool {
        todo!()
    }

    fn has_color_glyphs(&self) -> bool {
        todo!()
    }
}
