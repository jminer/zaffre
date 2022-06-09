use std::ops::Range;


#[derive(Debug, Clone, Copy)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug)]
pub struct TextAnalyzerRun {
    text_range: Range<usize>,
    direction: TextDirection,
}

impl TextAnalyzerRun {
    pub fn text_range(&self) -> Range<usize> {
        return self.text_range.clone();
    }

    pub fn direction(&self) -> TextDirection {
        return self.direction;
    }
}

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
