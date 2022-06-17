use std::collections::HashMap;
use std::ptr;
use std::ffi::c_void;

use core_foundation::array::CFArray;
use core_foundation::dictionary::CFDictionary;
use core_foundation::dictionary::CFDictionaryCreate;
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation::set::CFSet;
use core_foundation::string::CFString;
use core_text::font::CTFont;
use core_text::font_collection;
use core_text::font_descriptor;
use core_text::font_descriptor::CTFontDescriptor;
use core_text::font_descriptor::CTFontDescriptorCreateMatchingFontDescriptors;
use core_text::font_descriptor::kCTFontFamilyNameAttribute;

use crate::font;
use crate::font::FontSlant;
use crate::font::OpenTypeFontWeight;
use crate::font::OpenTypeFontWidth;
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

// fn compare_font_descriptors_family_name(desc1: CTFontDescriptor, desc2: CTFontDescriptor, data: *const c_void) {
//     let family1 = CTFontDescriptorCopyAttribute(desc1, kCTFontFamilyNameAttribute);
//     let family2 = CTFontDescriptorCopyAttribute(desc1, kCTFontFamilyNameAttribute);
//     let result = todo!();
//     result
// }

pub fn descriptors_create_for_family(family: &str) -> Vec<CTFontDescriptor> {
    unsafe {
        let family_attr = CFString::wrap_under_get_rule(kCTFontFamilyNameAttribute);
        let family_name: CFString = family.parse().unwrap();
        let specified_attrs = CFDictionary::from_CFType_pairs(&[
            (family_attr.clone(), family_name.as_CFType())
        ]);

        let wildcard_desc: CTFontDescriptor =
            font_descriptor::new_from_attributes(&specified_attrs);
        let mandatory_attrs = CFSet::from_slice(&[ family_attr.as_CFType() ]);
        let matched_descs = CTFontDescriptorCreateMatchingFontDescriptors(
                wildcard_desc.as_concrete_TypeRef(),
                mandatory_attrs.as_concrete_TypeRef());
        if matched_descs.is_null() {
            return Vec::new();
        }
        let matched_descs = CFArray::wrap_under_create_rule(matched_descs);
        matched_descs.into_iter().collect::<Vec<_>>()
    }
}

pub(crate) struct FontFunctionsBackend;
impl GenericFontFunctionsBackend for FontFunctionsBackend {
    fn get_families() -> Vec<FontFamily> {
        let mut families_hash: HashMap<String, Vec<CTFontDescriptor>> = HashMap::with_capacity(40);
        let collection = font_collection::create_for_all_families();
        let descriptor_array = collection.get_descriptors();// CTFontCollectionCreateMatchingFontDescriptorsSortedWithCallback(collection);
        if let Some(descriptor_array) = descriptor_array {
            for i in 0..descriptor_array.len() {
                let descriptor = descriptor_array.get(i);
                if let Some(descriptor) = descriptor {
                    let family = descriptor.family_name();
                    let family_descriptors = families_hash.entry(&family).or_insert_with(Vec::new());
                    family_descriptors.push(descriptor);
                }
            }
        }
        let font_families: Vec<_> = families_hash.into_iter().map(|(family_name, descriptors)| {
            FontFamily {
                backend: FontFamilyBackend {
                    name: family_name,
                    descriptors,
                },
            }
        }).collect();
        font_families
        // TODO: I think the Rust libs handle memory management, but I need to make sure there is an autorelease pool
    }

    fn get_family(name: &str) -> Option<FontFamily> {
        let descriptors = descriptors_create_for_family(name);
        unsafe {
            Some(FontFamily {
                backend: FontFamilyBackend {
                    name: name.to_owned(),
                    descriptors,
                },
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

#[derive(Debug, Clone)]
pub struct FontDescriptionBackend {
    font_desc: CTFontDescriptor,
}

impl GenericFontDescriptionBackend for FontDescriptionBackend {
    fn get_family_name(&self) -> String {
        unsafe {
            todo!()
        }
    }

    fn get_style_name(&self) -> String {
        unsafe {
            todo!()
        }
    }

    fn weight(&self) -> OpenTypeFontWeight {
        unsafe {
            todo!()
        }
    }

    fn slant(&self) -> FontSlant {
        unsafe {
            todo!()
        }
    }

    fn width(&self) -> OpenTypeFontWidth {
        unsafe {
            todo!()
        }
    }

    fn is_monospaced(&self) -> bool {
            todo!()
    }

    fn has_color_glyphs(&self) -> bool {
            todo!()
    }

    fn get_font(&self) -> Font {
        unsafe {
            todo!()
        }
    }
}
