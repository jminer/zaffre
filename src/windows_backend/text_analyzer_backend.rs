use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::{iter, ptr};

use windows::Win32::Graphics::DirectWrite::{IDWriteTextAnalysisSink, IDWriteTextAnalysisSink_Impl, DWRITE_SCRIPT_ANALYSIS, DWRITE_LINE_BREAKPOINT, IDWriteNumberSubstitution};
use windows::core::implement;

use crate::generic_backend::{GenericTextAnalyzerBackend, GenericTextAnalyzerRunBackend};
use crate::text_analyzer::{TextAnalyzerRun, TextDirection};


#[derive(Debug, Clone, Copy)]
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

//struct DWriteRunAnalysisSinkInner {
//    script_analysis: Vec<(Range<usize>, *const DWRITE_SCRIPT_ANALYSIS)>,
//    bidi_levels: Vec<(Range<usize>, BidiLevels)>,
//    number_substitution: Vec<(Range<usize>, IDWriteNumberSubstitution)>,
//}

#[implement(IDWriteTextAnalysisSink)]
struct DWriteRunAnalysisSink {
    runs: Cell<Vec<TextAnalyzerRun>>,
    last_run_index: Cell<usize>,

    //inner: RefCell<DWriteRunAnalysisSinkInner>,
}

impl DWriteRunAnalysisSink {
    fn new() -> Self {
        Self {
            runs: Default::default(),
            last_run_index: Cell::new(0),
        }
    }

    fn clear_and_resize(&self, len: usize) {
        let mut runs = self.runs.replace(Vec::new());

        runs.clear();
        runs.push(TextAnalyzerRun {
            text_range: 0..len,
            direction: TextDirection::LeftToRight,
            backend: TextAnalyzerRunBackend {
                script_analysis: ptr::null(),
                bidi_levels: None,
                number_substitution:  None,
            },
        });

        self.runs.replace(runs);
    }

    //fn build_runs(&self, runs: &mut Vec<TextAnalyzerRun>) {
    //    // Loop through the three arrays, processing one element from one array each iteration,
    //    // adding one element to the output array.
    //    let inner = self.inner.borrow_mut();
//
    //    runs.clear();
    //    let mut next_run =TextAnalyzerRun {
    //        text_range: 0..0,
    //        direction: TextDirection::LeftToRight,
    //        backend: TextAnalyzerRunBackend {
    //            script_analysis: ptr::null(),
    //            bidi_levels: None,
    //            number_substitution:  None,
    //        },
    //    };
//
    //    let mut i = 0;
    //    let mut j = 0;
    //    let mut k = 0;
    //    while i < inner.script_analysis.len() &&
    //        j < inner.bidi_levels.len() &&
    //        k < inner.number_substitution.len()
    //    {
    //        let sa_start = inner.script_analysis.get(i).map(|(r, _)| r.start);
    //        let bidi_start = inner.bidi_levels.get(i).map(|(r, _)| r.start);
    //        let ns_start = inner.number_substitution.get(i).map(|(r, _)| r.start);
    //    }
    //}

    fn get_run_range(&self, text_position: u32, text_length: u32) -> Range<usize> {
        let text_position = text_position as usize;
        let text_length = text_length as usize;
        let text_end = text_position + text_length;

        let mut runs = self.runs.take();
        let last_run_index = self.last_run_index.get();

        fn split_run(runs: &mut Vec<TextAnalyzerRun>, run_index: usize, text_index: usize) {
            debug_assert!(runs[run_index].text_range.start < text_index);
            debug_assert!(runs[run_index].text_range.end > text_index);

            let mut new_run = runs[run_index].clone();
            runs[run_index].text_range.end = text_index;
            new_run.text_range.start = text_index;
            runs.insert(run_index + 1, new_run);
        }

        debug_assert!(!runs.is_empty());
        // The CustomLayout DirectWrite sample says that the analyzers usually move forward. If the
        // position is forward, we start from the stored index, and if the analyzer moved backwards,
        // we search from the beginning.
        let mut run_start = if text_position < runs[last_run_index].text_range.start {
            0
        } else {
            last_run_index
        };
        while runs[run_start].text_range.end <= text_position {
            run_start += 1;
        }
        if runs[run_start].text_range.start != text_position {
            split_run(&mut runs, run_start, text_position);
            run_start += 1;
        }

        let mut run_end = run_start;
        while runs[run_end].text_range.end <= text_end {
            run_end += 1;
        }
        if runs[run_end - 1].text_range.end != text_end {
            split_run(&mut runs, run_end, text_end);
            run_end += 1;
        }

        self.runs.replace(runs);
        self.last_run_index.set(run_start);

        run_start..run_end
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use windows::Win32::Graphics::DirectWrite::IDWriteTextAnalysisSink_Impl;

    use super::DWriteRunAnalysisSink;

    fn set_bidi_levels(sink: &DWriteRunAnalysisSink, level_range_calls: &[(u8, Range<usize>)]) {
        for (level, range) in level_range_calls {
            sink.SetBidiLevel(range.start as u32, (range.end - range.start) as u32, 0, *level)
                .unwrap();
        }
    }

    fn get_bidi_levels(sink: &DWriteRunAnalysisSink) -> Vec<(u8, Range<usize>)> {
        let runs = sink.runs.take();

        let level_range_calls: Vec<_> = runs.iter().map(|run|
            (run.backend.bidi_levels
                .map(|bidi_levels| bidi_levels.resolved_level)
                .unwrap_or(0),
                run.text_range.clone()
            )
        ).collect();

        sink.runs.replace(runs);

        level_range_calls
    }

    #[test]
    fn test_dwrite_run_analysis_sink_1() {
        let sink = DWriteRunAnalysisSink::new();
        sink.clear_and_resize(23);
        let bidi_level_range_calls = &[
            (2, 4..7),
            (3, 0..4),
            (4, 7..12),
            (5, 15..20),
            (6, 10..18),
        ];
        set_bidi_levels(&sink, bidi_level_range_calls);
        assert_eq!(get_bidi_levels(&sink), &[
            (3, 0..4),
            (2, 4..7),
            (4, 7..10),
            (6, 10..12),
            (6, 12..15),
            (6, 15..18),
            (5, 18..20),
            (0, 20..23),
        ]);
    }

}

impl IDWriteTextAnalysisSink_Impl for DWriteRunAnalysisSink {
    fn SetScriptAnalysis(
        &self,
        text_position: u32,
        text_length: u32,
        script_analysis: *const DWRITE_SCRIPT_ANALYSIS,
    ) -> windows::core::Result<()> {
        println!("SetScriptAnalysis({}, {})", text_position, text_length);
        let mut runs = self.runs.take();

        self.runs.replace(runs);
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
        println!("SetBidiLevel({}, {})", text_position, text_length);

        let range = self.get_run_range(text_position, text_length);

        let mut runs = self.runs.take();
        for run in &mut runs[range] {
            run.backend.bidi_levels = Some(BidiLevels {
                explicit_level,
                resolved_level,
            });
        }

        self.runs.replace(runs);
        Ok(())
    }

    fn SetNumberSubstitution(
        &self,
        text_position: u32,
        text_length: u32,
        number_substitution: &Option<IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        println!("SetNumberSubstitution({}, {})", text_position, text_length);
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
        let mut breakpoints = self.breakpoints.take();
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
        let text_end = (text_position + text_length) as usize;

        for i in text_position as usize..text_end {
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