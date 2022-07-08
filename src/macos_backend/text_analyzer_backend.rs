use std::fmt::Debug;

use core_foundation::attributed_string::CFAttributedString;
use core_text::line::CTLine;

use crate::generic_backend::{GenericTextAnalyzerBackend, GenericTextAnalyzerRunBackend};
use crate::text_analyzer::TextAnalyzerRun;


#[derive(Debug, Clone)]
pub struct TextAnalyzerRunBackend {
    // leave empty I guess
}

impl GenericTextAnalyzerRunBackend for TextAnalyzerRunBackend {
}

#[derive(Clone)]
pub struct TextAnalyzerBackend {
    text: String,
    cf_string: CFAttributedString,
}

impl Debug for TextAnalyzerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextAnalyzerBackend").field("text", self.text).finish()
    }
}

impl GenericTextAnalyzerBackend for TextAnalyzerBackend {
    fn new(text: String) -> Self {
        todo!()
    }

    fn get_runs(&self) -> Vec<TextAnalyzerRun> {
        let line = CTLine::new_with_attributed_string(self.cf_string);
        let ct_runs = line.glyph_runs();
        let runs = Vec::with_capacity(ct_runs.len() as usize);
        runs
    }

    // To get glyphs, we have to create a new CTLine for every call (or drastically change the
    // TextAnalyzer interface).
}
