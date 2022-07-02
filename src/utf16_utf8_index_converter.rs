

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

pub(crate) struct Utf16Utf8IndexConverter<'a, 'b> {
    pub(crate) utf16_str: &'a [u16],
    pub(crate) utf8_str: &'b str,
    pub(crate) utf16_index: usize,
    pub(crate) utf8_index: usize,
}

impl<'a, 'b> Iterator for Utf16Utf8IndexConverter<'a, 'b> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let indexes = (self.utf16_index, self.utf8_index);
        if self.utf16_index >= self.utf16_str.len() {
            if self.utf16_index == self.utf16_str.len() {
                self.utf16_index += 1;
                return Some(indexes);
            }
            return None;
        }

        self.utf16_index += if self.utf16_str[self.utf16_index].is_utf16_surrogate() {
            2
        } else {
            1
        };

        self.utf8_index += utf8_code_point_byte_len(self.utf8_str.as_bytes()[self.utf8_index]);

        Some(indexes)
    }
}

impl<'a, 'b> Utf16Utf8IndexConverter<'a, 'b> {
    fn new(utf16: &'a [u16], utf8: &'b str) -> Self {
        Self {
            utf16_str: utf16,
            utf8_str: utf8,
            utf16_index: 0,
            utf8_index: 0,
        }
    }

    pub(crate) fn convert_to_utf8_index(&mut self, utf16_index: usize) -> usize {
        debug_assert!(utf16_index >= self.utf16_index);
        while let Some((u16_index, u8_index)) = self.next() {
            if u16_index >= utf16_index {
                debug_assert!(u16_index == utf16_index);
                return u8_index;
            }
        }
        panic!("index out of range");
    }

    pub(crate) fn convert_to_utf16_index(&mut self, utf8_index: usize) -> usize {
        debug_assert!(utf8_index >= self.utf8_index);
        while let Some((u16_index, u8_index)) = self.next() {
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
    let (u16_str, u8_str) = (
        [0x61u16, 0x6E, 0x20, 0x5DC, 0x5DC, 0x5e9, 0x5D5, 0x5DF, 0x21],
        "an ללשון!"
    );
    let mut converter = Utf16Utf8IndexConverter::new(&u16_str, u8_str);
    assert_eq!(converter.convert_to_utf8_index(0), 0);
    assert_eq!(converter.convert_to_utf8_index(1), 1);
    assert_eq!(converter.convert_to_utf8_index(2), 2);
    assert_eq!(converter.convert_to_utf8_index(3), 3);
    assert_eq!(converter.convert_to_utf8_index(4), 5);
    assert_eq!(converter.convert_to_utf8_index(5), 7);
    assert_eq!(converter.convert_to_utf8_index(7), 11);
    assert_eq!(converter.convert_to_utf8_index(8), 13);

    // utf8: \x68\x69\xf0\x9f\x99\x83\xf0\x9f\x92\x99\xf0\x9f\x92\x9a
    let (u16_str, u8_str) = (
        [0x0068, 0x0069, 0xd83d, 0xde43, 0xd83d, 0xdc99, 0xd83d, 0xdc9a],
        "hi🙃💙💚"
    );
    let converter = Utf16Utf8IndexConverter::new(&u16_str, u8_str);
    assert_eq!(converter.collect::<Vec<_>>(), &[
        (0, 0), (1, 1), (2, 2), (4, 6), (6, 10), (8, 14)
    ]);
    let mut converter = Utf16Utf8IndexConverter::new(&u16_str, u8_str);
    assert_eq!(converter.convert_to_utf8_index(2), 2);
    assert_eq!(converter.convert_to_utf8_index(4), 6);
    assert_eq!(converter.convert_to_utf8_index(8), 14);
}

#[test]
fn utf8_to_utf16_conversion_test() {
    // https://convertcodes.com/utf16-encode-decode-convert-string/

    // utf8: \x68\x69\xf0\x9f\x99\x83\xf0\x9f\x92\x99\xf0\x9f\x92\x9a
    let (u16_str, u8_str) = (
        [0x0068, 0x0069, 0xd83d, 0xde43, 0xd83d, 0xdc99, 0xd83d, 0xdc9a],
        "hi🙃💙💚"
    );
    let converter = Utf16Utf8IndexConverter::new(&u16_str, u8_str);
    assert_eq!(converter.collect::<Vec<_>>(), &[
        (0, 0), (1, 1), (2, 2), (4, 6), (6, 10), (8, 14)
    ]);
    let mut converter = Utf16Utf8IndexConverter::new(&u16_str, u8_str);
    assert_eq!(converter.convert_to_utf16_index(2), 2);
    assert_eq!(converter.convert_to_utf16_index(6), 4);
    assert_eq!(converter.convert_to_utf16_index(14), 8);
}

