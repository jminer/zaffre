
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
