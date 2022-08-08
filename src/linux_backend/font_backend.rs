
// No way to load a font from memory:
// https://gitlab.freedesktop.org/fontconfig/fontconfig/-/issues/12
// Although, he does mention using a Linux API to give a filename to memory, so that could work.



// It's possible to have FreeType render a glyph to the bitmap in a glyph slot. But it's also
// possible to call FT_Load_Glyph(), then FT_Get_Glyph() to copy the glyph out of the face. Then
// it's a separate object unrelated to the face that you can render using FT_Glyph_To_Bitmap().
// There is an example under
// https://freetype.org/freetype2/docs/reference/ft2-glyph_management.html#ft_glyph_to_bitmap
