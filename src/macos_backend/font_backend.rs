use std::ptr;
use std::ffi::c_void;

use core_text::font_collection;
use core_text::font_descriptor::CTFontDescriptor;

use crate::generic_backend::GenericFontFunctionsBackend;
use crate::generic_backend::GenericFontFamilyBackend;
use crate::font::FontFamily;


// CTFontCollectionCreateFromAvailableFonts
// CTFontCollectionCreateMatchingFontDescriptors
// CTFontCollectionCreateMatchingFontDescriptorsForFamily

// CTFontDescriptorCopyAttribute
//   kCTFontFamilyNameAttribute
//   kCTFontStyleNameAttribute
//   kCTFontTraitsAttribute

//
// Traits:
//   kCTFontWeightTrait
//   kCTFontWidthTrait
//   kCTFontSlantTrait
//   kCTFontSymbolicTrait
// Symbolic traits:
//   kCTFontTraitItalic
//   kCTFontTraitBold
//   kCTFontTraitMonoSpace
//   kCTFontTraitColorGlyphs


// For making the CTFontCollection clone, since it isn't reference counted like
// IDWriteFontCollection and PangoFontFamily, just wrap it in an Rc even though the other backends
// don't need to.


// A font name kCTFontNameAttribute is like "Helvetica-BoldMT" whereas the font family name is
// "Helvetica".

// From CTFontDescriptorCreateMatchingFontDescriptors: "If descriptor itself is normalized, then the
// array will contain only one item: the original descriptor. In the context of font descriptors,
// normalized infers that the input values were matched up with actual existing fonts, and the
// descriptors for those existing fonts are the returned normalized descriptors."

fn compare_font_descriptors_family_name(desc1: CTFontDescriptor, desc2: CTFontDescriptor, data: *const c_void) {
    let family1 = CTFontDescriptorCopyAttribute(desc1, kCTFontFamilyNameAttribute);
    let family2 = CTFontDescriptorCopyAttribute(desc1, kCTFontFamilyNameAttribute);
    let result = todo!();
    result
}

pub(crate) struct FontFunctionsBackend;
impl GenericFontFunctionsBackend for FontFunctionsBackend {
    fn get_families() -> Vec<FontFamily> {
        unsafe {
            let collection = font_collection::create_for_all_families();
            let descriptor_array = CTFontCollectionCreateMatchingFontDescriptorsSortedWithCallback(collection);
            for i in 0..CFArrayGetCount(descriptor_array) {
                let descriptor = CFArrayGetValueAtIndex(descriptor_array, i);
            }
            // TODO: how is memory management done? I probably need to call CFRelease()?
        }
    }

    fn get_family(name: &str) -> Option<FontFamily> {
        unsafe {
            Some(FontFamily {
                backend: FontFamilyBackend { family, name: todo!(), descriptors: todo!() }
            })
        }
    }
}


#[derive(Debug, Clone)]
pub struct FontFamilyBackend {
    name: String,
    descriptors: Vec<CTFontDescriptor>,
}

impl GenericFontFamilyBackend for FontFamilyBackend {
    fn get_name(&self) -> String {
        self.name.clone()
    }
}
