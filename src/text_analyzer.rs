use std::ops::Range;

use nalgebra::Point2;
use smallvec::SmallVec;

use crate::font::Font;
use crate::generic_backend::{GenericTextAnalyzerRunBackend, GenericTextAnalyzerBackend};
use crate::backend::text_analyzer_backend::{TextAnalyzerRunBackend, TextAnalyzerBackend};


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
    // Maps from characters to glyphs.
    pub cluster_map: SmallVec<[usize; 32]>,
    // Glyphs in the font.
    pub glyphs: SmallVec<[u16; 32]>,
    pub glyph_advances: SmallVec<[f32; 32]>,
    pub glyph_offsets: SmallVec<[Point2<f32>; 32]>,
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

    pub fn new(text: String) -> Self {
        Self {
            backend: TextAnalyzerBackend::new(text)
        }
    }

    pub fn text(&self) -> &str {
        self.backend.text()
    }

    pub fn get_runs(&self) -> Vec<TextAnalyzerRun> {
        self.backend.get_runs()
    }

    pub fn get_glyphs_and_positions(
        &self,
        text_range: Range<usize>,
        run: TextAnalyzerRun,
        font: &Font,
        font_size: f32,
    ) -> TextAnalyzerGlyphRun {
        debug_assert!(run.text_range().contains(&text_range.start));
        debug_assert!(run.text_range().contains(&(text_range.end - 1)));
        self.backend.get_glyphs_and_positions(text_range, run, font, font_size)
    }
}
