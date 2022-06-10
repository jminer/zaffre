use crate::generic_backend::GenericTextAnalyzerBackend;



pub struct TextAnalyzerRunBackend {

}

impl GenericTextAnalyzerRunBackend for TextAnalyzerRunBackend {
}

pub struct TextAnalyzerBackend {

}

impl GenericTextAnalyzerBackend for TextAnalyzerBackend {
    fn new(text: String) -> Self {
        todo!()
    }

    fn get_runs(&self) -> Vec<TextAnalyzerRun> {
        //CTLineGe
        todo!()
    }
}