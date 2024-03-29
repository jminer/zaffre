

fn is_utf8_code_point_first_byte(byte: u8) -> bool {
    (byte >> 6) != 0b10
}

fn utf8_code_point_byte_len(first_byte: u8) -> usize {
    // I tried indexing an array instead of a match, but the match is many fewer instructions.
    match first_byte.leading_ones() {
        0 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        _ => 0, // shouldn't happen; just return zero
    }
}

pub(crate) struct UtfIndexConverter<'a, 'b> {
    pub(crate) utf8_str: &'b str,
    pub(crate) utf16_str: &'a [u16],
    pub(crate) initial: bool,
    pub(crate) utf8_index: usize,
    pub(crate) utf16_index: usize,
}

impl<'a, 'b> Iterator for UtfIndexConverter<'a, 'b> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.utf16_index >= self.utf16_str.len() {
            debug_assert!(self.utf16_index == self.utf16_str.len());
            return None;
        }
        if !self.initial {
            self.utf16_index += if self.utf16_str[self.utf16_index].is_utf16_surrogate() {
                2
            } else {
                1
            };

            self.utf8_index += utf8_code_point_byte_len(self.utf8_str.as_bytes()[self.utf8_index]);
        }
        self.initial = false;

        Some((self.utf8_index, self.utf16_index))
    }
}

impl<'a, 'b> UtfIndexConverter<'a, 'b> {
    pub(crate) fn new(utf8: &'b str, utf16: &'a [u16]) -> Self {
        Self {
            utf8_str: utf8,
            utf16_str: utf16,
            initial: true,
            utf8_index: 0,
            utf16_index: 0,
        }
    }

    pub(crate) fn convert_to_utf8_index(&mut self, utf16_index: usize) -> usize {
        debug_assert!(utf16_index >= self.utf16_index);
        // Support converting the same index multiple times.
        if utf16_index == self.utf16_index {
            return self.utf8_index;
        }
        while let Some((u8_index, u16_index)) = self.next() {
            if u16_index >= utf16_index {
                debug_assert!(u16_index == utf16_index);
                return u8_index;
            }
        }
        panic!("index out of range");
    }

    pub(crate) fn convert_to_utf16_index(&mut self, utf8_index: usize) -> usize {
        debug_assert!(utf8_index >= self.utf8_index);
        // Support converting the same index multiple times.
        if utf8_index == self.utf8_index {
            return self.utf16_index;
        }
        while let Some((u8_index, u16_index)) = self.next() {
            if u8_index >= utf8_index {
                debug_assert!(u8_index == utf8_index);
                return u16_index;
            }
        }
        panic!("index out of range");
    }
}

#[test]
fn utf16_to_utf8_conversion_test() {
    // https://convertcodes.com/utf16-encode-decode-convert-string/

    // ללשון
    // utf8: \x61\x6e\x20\xd7\x9c\xd7\x9c\xd7\xa9\xd7\x95\xd7\x9f\x21
    let (u8_str, u16_str) = (
        "an ללשון!",
        [0x61u16, 0x6E, 0x20, 0x5DC, 0x5DC, 0x5e9, 0x5D5, 0x5DF, 0x21]
    );
    let mut converter = UtfIndexConverter::new(u8_str, &u16_str);
    assert_eq!(converter.convert_to_utf8_index(0), 0);
    assert_eq!(converter.convert_to_utf8_index(1), 1);
    assert_eq!(converter.convert_to_utf8_index(2), 2);
    assert_eq!(converter.convert_to_utf8_index(3), 3);
    assert_eq!(converter.convert_to_utf8_index(4), 5);
    assert_eq!(converter.convert_to_utf8_index(5), 7);
    assert_eq!(converter.convert_to_utf8_index(7), 11);
    assert_eq!(converter.convert_to_utf8_index(8), 13);

    // utf8: \x68\x69\xf0\x9f\x99\x83\xf0\x9f\x92\x99\xf0\x9f\x92\x9a
    let (u8_str, u16_str) = (
        "hi🙃💙💚",
        [0x0068, 0x0069, 0xd83d, 0xde43, 0xd83d, 0xdc99, 0xd83d, 0xdc9a]
    );
    let converter = UtfIndexConverter::new(u8_str, &u16_str);
    assert_eq!(converter.collect::<Vec<_>>(), &[
        (0, 0), (1, 1), (2, 2), (6, 4), (10, 6), (14, 8)
    ]);
    let mut converter = UtfIndexConverter::new(u8_str, &u16_str);
    assert_eq!(converter.convert_to_utf8_index(2), 2);
    assert_eq!(converter.convert_to_utf8_index(2), 2);
    assert_eq!(converter.convert_to_utf8_index(4), 6);
    assert_eq!(converter.convert_to_utf8_index(8), 14);
}

#[test]
fn utf8_to_utf16_conversion_test() {
    // https://convertcodes.com/utf16-encode-decode-convert-string/

    // utf8: \x68\x69\xf0\x9f\x99\x83\xf0\x9f\x92\x99\xf0\x9f\x92\x9a
    let (u8_str, u16_str) = (
        "hi🙃💙💚",
        [0x0068, 0x0069, 0xd83d, 0xde43, 0xd83d, 0xdc99, 0xd83d, 0xdc9a]
    );
    let converter = UtfIndexConverter::new(u8_str, &u16_str);
    assert_eq!(converter.collect::<Vec<_>>(), &[
        (0, 0), (1, 1), (2, 2), (6, 4), (10, 6), (14, 8)
    ]);
    let mut converter = UtfIndexConverter::new(u8_str, &u16_str);
    assert_eq!(converter.convert_to_utf16_index(2), 2);
    assert_eq!(converter.convert_to_utf16_index(2), 2);
    assert_eq!(converter.convert_to_utf16_index(6), 4);
    assert_eq!(converter.convert_to_utf16_index(14), 8);
}

pub(crate) struct Utf16IndexesForUtf8<'a, 'b> {
    pub(crate) utf8_str: &'b str,
    pub(crate) utf16_str: &'a [u16],
    pub(crate) initial: bool,
    pub(crate) utf8_index: usize,
    pub(crate) utf16_index: usize,
}

impl<'a, 'b> Utf16IndexesForUtf8<'a, 'b> {
    pub(crate) fn new(
        utf8_str: &'b str,
        utf16_str: &'a [u16],
    ) -> Self {
        Self {
            utf8_str,
            utf16_str,
            initial: true,
            utf8_index: 0,
            utf16_index: 0,
        }
    }
}

impl<'a, 'b> Iterator for Utf16IndexesForUtf8<'a, 'b> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.utf8_index + 1 >= self.utf8_str.len() {
            debug_assert!(self.utf8_index + 1 == self.utf8_str.len());
            return None;
        }
        if !self.initial {
            self.utf8_index += 1;
            if is_utf8_code_point_first_byte(self.utf8_str.as_bytes()[self.utf8_index]) {
                self.utf16_index += if self.utf16_str[self.utf16_index].is_utf16_surrogate() {
                    2
                } else {
                    1
                };
            }
        }
        self.initial = false;

        Some(self.utf16_index)
    }
}

#[test]
fn utf16_index_for_utf8_test() {
    // https://convertcodes.com/utf16-encode-decode-convert-string/

    // utf8: \x68\x69\x20\xd7\x90\xd7\x91\xd7\x90
    let (u8_str, u16_str) = (
        "hi אבא",
        [0x0068, 0x0069, 0x0020, 0x05d0, 0x05d1, 0x05d0]
    );
    let converter = Utf16IndexesForUtf8::new(u8_str, &u16_str);
    assert_eq!(converter.collect::<Vec<_>>(), &[
        0, 1, 2, 3, 3, 4, 4, 5, 5
    ]);
}

#[test]
fn utf16_index_for_utf8_test_with_emoji() {
    // https://convertcodes.com/utf16-encode-decode-convert-string/

    // utf8: \x68\x69\xf0\x9f\x99\x83\xf0\x9f\x92\x99\xf0\x9f\x92\x9a
    let (u8_str, u16_str) = (
        "hi🙃💙💚",
        [0x0068, 0x0069, 0xd83d, 0xde43, 0xd83d, 0xdc99, 0xd83d, 0xdc9a]
    );
    let converter = Utf16IndexesForUtf8::new(u8_str, &u16_str);
    assert_eq!(converter.collect::<Vec<_>>(), &[
        0, 1, 2, 2, 2, 2, 4, 4, 4, 4, 6, 6, 6, 6
    ]);
}
