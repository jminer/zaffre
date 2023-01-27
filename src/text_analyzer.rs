use std::ops::Range;
use std::rc::Rc;

use bit_vec::BitVec;
use nalgebra::Point2;
use smallvec::SmallVec;

use crate::font::Font;
use crate::generic_backend::{GenericTextAnalyzerRunBackend, GenericTextAnalyzerBackend};
use crate::backend::text_analyzer_backend::{TextAnalyzerRunBackend, TextAnalyzerBackend};
use crate::text::FormattedString;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone)]
pub struct TextAnalyzerRun<B: GenericTextAnalyzerRunBackend = TextAnalyzerRunBackend> {
    pub(crate) text_range: Range<usize>,
    pub(crate) direction: TextDirection,
    pub(crate) backend: B,
}

impl TextAnalyzerRun {
    pub fn text_range(&self) -> Range<usize> {
        return self.text_range.clone();
    }

    pub fn direction(&self) -> TextDirection {
        return self.direction;
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TextAnalyzerGlyphRun {
    pub run: TextAnalyzerRun,
    // TODO: should I have text_range and font in here like this? The caller could always define
    // their own struct if they wanted to keep them together.
    pub text_range: Range<usize>,
    pub font: Font,
    // Maps from characters to glyphs. The values always increase monotonically.
    pub cluster_map: SmallVec<[usize; 32]>,
    // Glyphs in the font. Glyphs are overall in logical order but are in visual order within a
    // cluster.
    pub glyphs: SmallVec<[u16; 32]>,
    pub glyph_advances: SmallVec<[f32; 32]>,
    pub glyph_offsets: SmallVec<[Point2<f32>; 32]>,
}

impl TextAnalyzerGlyphRun {
    pub fn split_off(&mut self, text_index: usize) -> Self {
        fn smallvec_split_off<A: smallvec::Array>(v: &mut SmallVec<A>, index: usize) -> SmallVec<A>
        where
            A::Item: Copy,
        {
            let new_vec = SmallVec::from_slice(&v[index..]);
            v.truncate(index);
            new_vec
        }

        let mut new_cluster_map = smallvec_split_off(&mut self.cluster_map, text_index);
        let glyph_split_index = *new_cluster_map.first()
            .expect("can't split off an empty glyph run");
        for i in &mut new_cluster_map {
            *i -= glyph_split_index;
        }

        let new_text_range = text_index..self.text_range.end;
        self.text_range = 0..text_index;

        Self {
            run: self.run.clone(),
            text_range: new_text_range,
            font: self.font.clone(),
            cluster_map: new_cluster_map,
            glyphs: smallvec_split_off(&mut self.glyphs, glyph_split_index),
            glyph_advances: smallvec_split_off(&mut self.glyph_advances, glyph_split_index),
            glyph_offsets: smallvec_split_off(&mut self.glyph_offsets, glyph_split_index),
        }
    }
}

/// There is currently no support for custom mapping from Unicode characters to glyphs. Also, there
/// is no support for typographic features, but support will probably be added in the future.
///
/// Reusing TextAnalyzer objects improves performance because it can reuse allocations.
pub struct TextAnalyzer<B: GenericTextAnalyzerBackend = TextAnalyzerBackend> {
    backend: B,
}

impl TextAnalyzer {

    // provide line breaks, hyphenation breaks, caret stops, and word stops cross-platform,
    // as well as shaping and glyph placement?

    // have all fn take &self, not &mut self

    pub fn new() -> Self {
        Self {
            backend: TextAnalyzerBackend::new()
        }
    }

    pub fn text(&self) -> &str {
        self.backend.text()
    }

    pub fn set_text_from(&mut self, text: &String) {
        self.backend.set_text(text)
    }

    pub fn get_runs(&self) -> Vec<TextAnalyzerRun> {
        self.backend.get_runs()
    }

    pub fn get_glyphs_and_positions(
        &self,
        text_range: Range<usize>,
        run: TextAnalyzerRun,
        font: &Font,
    ) -> TextAnalyzerGlyphRun {
        debug_assert!(run.text_range().contains(&text_range.start));
        debug_assert!(run.text_range().contains(&(text_range.end - 1)));
        self.backend.get_glyphs_and_positions(text_range, run, font)
    }

    // Returns a bit vector where each bit represents whether the character index in the text is a
    // valid line break position. For multibyte characters, every bit cooresponding to the character
    // is the same value.
    pub fn get_line_breaks(&self) -> BitVec {
        self.backend.get_line_breaks()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use bit_vec::BitVec;

    use crate::font;
    use crate::text_analyzer::{TextAnalyzer, TextDirection};

    #[test]
    fn basic_english_line_breaks() {
        let mut analyzer = TextAnalyzer::new();
        analyzer.set_text_from(&"I like peaches".to_owned());
        let line_breaks = analyzer.get_line_breaks();

        assert_eq!(line_breaks.len(), analyzer.text().len());

        let mut expected_breaks = BitVec::from_elem(14, false);
        expected_breaks.set(2, true);
        expected_breaks.set(7, true);
        assert_eq!(&line_breaks, &expected_breaks);
    }

    #[test]
    fn basic_hebrew_line_breaks() {
        let mut analyzer = TextAnalyzer::new();
        analyzer.set_text_from(&"האב אוהב".to_owned());
        let line_breaks = analyzer.get_line_breaks();

        assert_eq!(line_breaks.len(), analyzer.text().len());

        let mut expected_breaks = BitVec::from_elem(15, false);
        expected_breaks.set(7, true);
        expected_breaks.set(8, true);
        assert_eq!(&line_breaks, &expected_breaks);
    }

    #[test]
    fn basic_english_glyph_run() {
        let font_family = font::get_family("DejaVu Sans")
            .expect("couldn't find font");
        let font = font_family.get_styles()[0].get_font(20.0);

        let mut analyzer = TextAnalyzer::new();
        analyzer.set_text_from(&"Apple".to_owned());
        let runs = analyzer.get_runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::LeftToRight);
        let glyph_run =
            analyzer.get_glyphs_and_positions(runs[0].text_range(), runs[0].clone(), &font);

        assert_eq!(&glyph_run.cluster_map[..], &[0, 1, 2, 3, 4]);

        assert_eq!(&glyph_run.glyphs[..], &[
            font.get_glyph('A'),
            font.get_glyph('p'),
            font.get_glyph('p'),
            font.get_glyph('l'),
            font.get_glyph('e'),
        ]);

        // These are the widths returned from the DirectWrite backend.
        assert_abs_diff_eq!(&glyph_run.glyph_advances[0], &13.68, epsilon = 0.5);
        assert_abs_diff_eq!(&glyph_run.glyph_advances[3], &5.56, epsilon = 0.5);
    }

    #[test]
    fn basic_hebrew_glyph_run() {
        let font_family = font::get_family("DejaVu Sans")
            .expect("couldn't find font");
        let font = font_family.get_styles()[0].get_font(20.0);

        // Hebrew characters are two bytes each in UTF-8.
        let mut analyzer = TextAnalyzer::new();
        analyzer.set_text_from(&"עברית".to_owned());
        let runs = analyzer.get_runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::RightToLeft);
        let glyph_run =
            analyzer.get_glyphs_and_positions(runs[0].text_range(), runs[0].clone(), &font);

        assert_eq!(&glyph_run.cluster_map[..], &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);

        assert_eq!(&glyph_run.glyphs[..], &[
            font.get_glyph('ע'),
            font.get_glyph('ב'),
            font.get_glyph('ר'),
            font.get_glyph('י'),
            font.get_glyph('ת'),
        ]);

        // These are the widths returned from the DirectWrite backend.
        assert_abs_diff_eq!(&glyph_run.glyph_advances[0], &12.52, epsilon = 0.5);
        assert_abs_diff_eq!(&glyph_run.glyph_advances[3], &4.47, epsilon = 0.5);
    }

    #[test]
    fn basic_devanagari_glyph_run() {
        let font_family = font::get_family("Noto Sans Devanagari")
            .expect("couldn't find font");
        let font = font_family.get_styles()[0].get_font(20.0);

        // https://en.wikipedia.org/wiki/Devanagari#Vowel_diacritics
        //
        // I chose kā ki for the test string. It has 4 code points, each 3 bytes. The vowel
        // codepoints come after the consonant codepoints. They result in 4 glyphs, but not in the
        // same order because the second vowel glyph comes before the second consonant.
        let mut analyzer = TextAnalyzer::new();
        analyzer.set_text_from(&"काकि".to_owned());
        let runs = analyzer.get_runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::LeftToRight);
        let glyph_run =
            analyzer.get_glyphs_and_positions(runs[0].text_range(), runs[0].clone(), &font);

        assert_eq!(&glyph_run.cluster_map[..], &[0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2]);

        // For some reason, ि doesn't result in the same glyph here even though it's the same
        // codepoint.
        assert_eq!(glyph_run.glyphs.len(), 4);
        assert_eq!(glyph_run.glyphs[0], font.get_glyph('क'));
        assert_eq!(glyph_run.glyphs[1], font.get_glyph('ा'));
        //assert_eq!(glyph_run.glyphs[2], font.get_glyph('ि'));
        assert_eq!(glyph_run.glyphs[3], font.get_glyph('क'));

        // These are the widths returned from the DirectWrite backend.
        assert_abs_diff_eq!(&glyph_run.glyph_advances[0], &15.24, epsilon = 0.5);
        assert_abs_diff_eq!(&glyph_run.glyph_advances[1], &5.18, epsilon = 0.5);
        assert_abs_diff_eq!(&glyph_run.glyph_advances[2], &5.18, epsilon = 0.5);
        assert_abs_diff_eq!(&glyph_run.glyph_advances[3], &15.24, epsilon = 0.5);
    }

    #[test]
    fn basic_multiple_glyph_runs() {
        let font_family = font::get_family("DejaVu Sans")
            .expect("couldn't find font");
        let font = font_family.get_styles()[0].get_font(20.0);

        let mut analyzer = TextAnalyzer::new();
        analyzer.set_text_from(&"добрий день".to_owned());
        let runs = analyzer.get_runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::LeftToRight);
        let glyph_run0 =
            analyzer.get_glyphs_and_positions(0..13, runs[0].clone(), &font);
        let glyph_run1 =
            analyzer.get_glyphs_and_positions(13..21, runs[0].clone(), &font);

        assert_eq!(&glyph_run0.cluster_map[..], &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
        assert_eq!(&glyph_run1.cluster_map[..], &[0, 0, 1, 1, 2, 2, 3, 3]);

        assert_eq!(&glyph_run0.glyphs[..], &[
            font.get_glyph('д'),
            font.get_glyph('о'),
            font.get_glyph('б'),
            font.get_glyph('р'),
            font.get_glyph('и'),
            font.get_glyph('й'),
            font.get_glyph(' '),
        ]);
        assert_eq!(&glyph_run1.glyphs[..], &[
            font.get_glyph('д'),
            font.get_glyph('е'),
            font.get_glyph('н'),
            font.get_glyph('ь'),
        ]);
    }
}