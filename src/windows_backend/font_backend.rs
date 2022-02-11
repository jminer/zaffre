use std::mem;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use windows::core::Interface;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED, IDWriteFontFamily, IDWriteFont,
};

use crate::generic_backend::{GenericFontCollectionBackend, GenericFontFamilyBackend, GenericFontDescriptionBackend};

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

#[derive(Debug, Clone)]
pub struct FontCollectionBackend {
    collection: IDWriteFontCollection,
}

impl GenericFontCollectionBackend for FontCollectionBackend {
    fn system() -> Self {
        unsafe {
            let collection = DWRITE_FACTORY.with(|factory| {
                let mut font_collection = None;
                factory.GetSystemFontCollection(&mut font_collection, false)
                    .expect("GetSystemFontCollection() failed");
                font_collection.unwrap() // can't be None if the expect() above passes
            });
            Self { collection }
        }
    }

}


#[derive(Debug, Clone)]
pub struct FontFamilyBackend {
    family: IDWriteFontFamily,
}

impl GenericFontFamilyBackend for FontFamilyBackend {
    fn get_name(&self) -> String {
        todo!()
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
