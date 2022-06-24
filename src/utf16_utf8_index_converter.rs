

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

struct Utf16Utf8IndexConverter<'a, 'b> {
    utf16_str: &'a [u16],
    utf8_str: &'b str,
    utf16_index: usize,
    utf8_index: usize,
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

    fn convert(&mut self, utf16_index: usize) -> usize {
        debug_assert!(utf16_index >= self.utf16_index);
        while let Some((u16_index, u8_index)) = self.next() {
            if u16_index >= utf16_index {
                debug_assert!(u16_index == utf16_index);
                return u8_index;
            }
        }
        panic!("index out of range");
    }
}

#[test]
fn utf16_utf8_conversion_test() {
    // ללשון
    let (u16_str, u8_str) = ([0x61u16, 0x6E, 0x20, 0x5DC, 0x5DC, 0x5e9, 0x5D5, 0x5DF, 0x21], "an ללשון!");
    let mut converter = Utf16Utf8IndexConverter::new(&u16_str, u8_str);
    assert_eq!(converter.convert(0), 0);
    assert_eq!(converter.convert(1), 1);
    assert_eq!(converter.convert(2), 2);
    assert_eq!(converter.convert(3), 3);
    assert_eq!(converter.convert(4), 5);
    assert_eq!(converter.convert(5), 7);
    assert_eq!(converter.convert(7), 11);
    assert_eq!(converter.convert(8), 13);
}
