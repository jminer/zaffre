use std::cell::Cell;
use std::iter;

use windows::Win32::Graphics::DirectWrite::{IDWriteTextAnalysisSink, IDWriteTextAnalysisSink_Impl, DWRITE_SCRIPT_ANALYSIS, DWRITE_LINE_BREAKPOINT, IDWriteNumberSubstitution};
use windows::core::implement;

use crate::generic_backend::{GenericTextAnalyzerBackend, GenericTextAnalyzerRunBackend};
use crate::text_analyzer::TextAnalyzerRun;


#[derive(Debug, Clone)]
struct BidiLevels {
    explicit_level: u8,
    resolved_level: u8,
}

#[derive(Debug, Clone)]
pub struct TextAnalyzerRunBackend {
    script_analysis: *const DWRITE_SCRIPT_ANALYSIS,
    bidi_levels: Option<BidiLevels>,
    number_substitution: Option<IDWriteNumberSubstitution>,
}

impl GenericTextAnalyzerRunBackend for TextAnalyzerRunBackend {
}

#[implement(IDWriteTextAnalysisSink)]
struct DWriteRunAnalysisSink {
    runs: Cell<Vec<TextAnalyzerRun>>,
}

impl IDWriteTextAnalysisSink_Impl for DWriteRunAnalysisSink {
    fn SetScriptAnalysis(
        &self,
        text_position: u32,
        text_length: u32,
        script_analysis: *const DWRITE_SCRIPT_ANALYSIS,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn SetLineBreakpoints(
        &self,
        _text_position: u32,
        _text_length: u32,
        _line_breakpoints: *const DWRITE_LINE_BREAKPOINT,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn SetBidiLevel(
        &self,
        text_position: u32,
        text_length: u32,
        explicit_level: u8,
        resolved_level: u8,
    ) -> windows::core::Result<()> {

        Ok(())
    }

    fn SetNumberSubstitution(
        &self,
        text_position: u32,
        text_length: u32,
        number_substitution: &Option<IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
}

#[implement(IDWriteTextAnalysisSink)]
struct DWriteLineBreakAnalysisSink {
    breakpoints: Cell<Vec<Option<DWRITE_LINE_BREAKPOINT>>>,
}

impl DWriteLineBreakAnalysisSink {
    fn new() -> Self {
        Self {
            breakpoints: Default::default(),
        }
    }

    fn clear_and_resize(&self, new_len: usize) {
        let mut breakpoints = self.breakpoints.replace(Vec::new());
        breakpoints.clear();
        breakpoints.resize(new_len, None);
        self.breakpoints.replace(breakpoints);
    }
}

impl IDWriteTextAnalysisSink_Impl for DWriteLineBreakAnalysisSink {
    fn SetScriptAnalysis(
        &self,
        _text_position: u32,
        _text_length: u32,
        _script_analysis: *const DWRITE_SCRIPT_ANALYSIS,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn SetLineBreakpoints(
        &self,
        text_position: u32,
        text_length: u32,
        line_breakpoints: *const DWRITE_LINE_BREAKPOINT,
    ) -> windows::core::Result<()> {
        let mut breakpoints = self.breakpoints.replace(Vec::new());
        let end_index = (text_position + text_length) as usize;

        for i in text_position as usize..end_index {
            breakpoints[i] = Some(unsafe { line_breakpoints.add(i).read() });
        }

        self.breakpoints.replace(breakpoints);

        Ok(())
    }

    fn SetBidiLevel(
        &self,
        _text_position: u32,
        _text_length: u32,
        _explicit_level: u8,
        _resolved_level: u8,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn SetNumberSubstitution(
        &self,
        _text_position: u32,
        _text_length: u32,
        _number_substitution: &Option<IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
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