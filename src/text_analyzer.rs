use std::ops::Range;
use std::rc::Rc;

use nalgebra::Point2;
use smallvec::SmallVec;

use crate::font::Font;
use crate::generic_backend::{GenericTextAnalyzerRunBackend, GenericTextAnalyzerBackend};
use crate::backend::text_analyzer_backend::{TextAnalyzerRunBackend, TextAnalyzerBackend};
use crate::text::FormattedString;


#[derive(Debug, Clone, Copy)]
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
    // Maps from characters to glyphs.
    pub cluster_map: SmallVec<[usize; 32]>,
    // Glyphs in the font.
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
}
