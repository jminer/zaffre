use std::ops::Range;

use crate::generic_backend::GenericTextAnalyzerRunBackend;
use crate::backend::text_analyzer_backend::TextAnalyzerRunBackend;


#[derive(Debug, Clone, Copy)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug)]
pub struct TextAnalyzerRun<B: GenericTextAnalyzerRunBackend = TextAnalyzerRunBackend> {
    text_range: Range<usize>,
    direction: TextDirection,
    backend: B,
}

impl TextAnalyzerRun {
    pub fn text_range(&self) -> Range<usize> {
        return self.text_range.clone();
    }

    pub fn direction(&self) -> TextDirection {
        return self.direction;
    }
}

// Reusing TextAnalyzer objects improves performance because it can reuse allocations.
pub struct TextAnalyzer {
    text: String,
}

impl TextAnalyzer {

    // provide line breaks, hyphenation breaks, caret stops, and word stops cross-platform,
    // as well as shaping and glyph placement?

    // have all fn take &self, not &mut self

    pub fn new(text: String) -> Self {
        todo!()
    }

    pub fn get_runs(&self) -> Vec<TextAnalyzerRun> {

        todo!()
    }

    pub fn get_glyphs_and_positions(&self, text_range: Range<usize>, run: TextAnalyzerRun) {

    }
}
