use std::fmt::Debug;

use core_foundation::attributed_string::CFAttributedString;
use core_text::line::CTLine;

use crate::generic_backend::{GenericTextAnalyzerBackend, GenericTextAnalyzerRunBackend};
use crate::text_analyzer::TextAnalyzerRun;


#[derive(Debug, Clone)]
pub struct TextAnalyzerRunBackend {
}

impl GenericTextAnalyzerRunBackend for TextAnalyzerRunBackend {
}

#[derive(Clone)]
pub struct TextAnalyzerBackend {
    text: String,
    cf_string: CFAttributedString,
    line: core_text::line::CTLine,
}

impl Debug for TextAnalyzerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextAnalyzerBackend").field("text", self.text).finish()
    }
}

impl GenericTextAnalyzerBackend for TextAnalyzerBackend {
    fn new(text: String) -> Self {
        let cf_string;
        let line = CTLine::new_with_attributed_string(cf_string);
        todo!()
    }

    fn get_runs(&self) -> Vec<TextAnalyzerRun> {
        let ct_runs = self.line.glyph_runs();
        let runs = Vec::with_capacity(ct_runs.len() as usize);
        runs
    }
}
