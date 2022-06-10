use crate::generic_backend::{GenericTextAnalyzerBackend, GenericTextAnalyzerRunBackend};
use crate::text_analyzer::TextAnalyzerRun;



#[derive(Debug, Clone, Copy)]
pub struct TextAnalyzerRunBackend {

}

impl GenericTextAnalyzerRunBackend for TextAnalyzerRunBackend {
}

#[derive(Debug, Clone, Copy)]
pub struct TextAnalyzerBackend {

}

impl GenericTextAnalyzerBackend for TextAnalyzerBackend {
    fn new(text: String) -> Self {
        todo!()
    }

    fn get_runs(&self) -> Vec<TextAnalyzerRun> {
        todo!()
    }
}