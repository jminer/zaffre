use core_text::line::CTLine;

use crate::generic_backend::{GenericTextAnalyzerBackend, GenericTextAnalyzerRunBackend};
use crate::text_analyzer::TextAnalyzerRun;



pub struct TextAnalyzerRunBackend {
}

impl GenericTextAnalyzerRunBackend for TextAnalyzerRunBackend {
}

pub struct TextAnalyzerBackend {
    cf_string: CFAttributedString,
    line: core_text::line::CTLine,
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
