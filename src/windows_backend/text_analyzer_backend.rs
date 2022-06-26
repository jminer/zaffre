use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::ops::Range;
use std::rc::Rc;
use std::{iter, ptr};

use windows::Win32::Globalization::{GetUserDefaultLocaleName, GetThreadLocale, GetLocaleInfoEx, LOCALE_SNAME};
use windows::Win32::Graphics::DirectWrite::{IDWriteTextAnalysisSink, IDWriteTextAnalysisSink_Impl, DWRITE_SCRIPT_ANALYSIS, DWRITE_LINE_BREAKPOINT, IDWriteNumberSubstitution, IDWriteTextAnalysisSource, IDWriteTextAnalysisSource_Impl, DWRITE_READING_DIRECTION_LEFT_TO_RIGHT, DWRITE_NUMBER_SUBSTITUTION_METHOD_NONE};
use windows::Win32::System::SystemServices::LOCALE_NAME_MAX_LENGTH;
use windows::core::{implement, PCWSTR};

use crate::generic_backend::{GenericTextAnalyzerBackend, GenericTextAnalyzerRunBackend};
use crate::text_analyzer::{TextAnalyzerRun, TextDirection};

use super::font_backend::DWRITE_FACTORY;
use super::wide_ffi_string::WideFfiString;

// https://github.com/microsoft/DWriteShapePy/blob/main/src/cpp/TextAnalysis.cpp
// https://github.com/microsoft/Windows-classic-samples/tree/main/Samples/Win7Samples/multimedia/DirectWrite/CustomLayout

#[derive(Debug, Clone, Copy)]
struct BidiLevels {
    explicit_level: u8,
    resolved_level: u8,
}

#[derive(Debug, Clone)]
pub struct TextAnalyzerRunBackend {
    script_analysis: Option<DWRITE_SCRIPT_ANALYSIS>,
    bidi_levels: Option<BidiLevels>,
    number_substitution: Option<IDWriteNumberSubstitution>,
}

impl GenericTextAnalyzerRunBackend for TextAnalyzerRunBackend {
}

struct DWriteAnalysisSourceData {
    wide_text: *mut u16,
    len: u32,
    locale_name: [u16; LOCALE_NAME_MAX_LENGTH as usize],
    num_subst: IDWriteNumberSubstitution,
}

#[implement(IDWriteTextAnalysisSource)]
struct DWriteAnalysisSource(Rc<DWriteAnalysisSourceData>);

impl IDWriteTextAnalysisSource_Impl for DWriteAnalysisSource {
    fn GetTextAtPosition(
        &self,
        text_position: u32,
        text_string: *mut *mut u16,
        text_length: *mut u32,
    ) -> windows::core::Result<()> {
        unsafe {
            if text_position > self.0.len {
                *text_string = ptr::null_mut();
                *text_length = 0;
            } else {
                *text_string = self.0.wide_text.add(text_position as usize);
                *text_length = self.0.len - text_position;
            }
            Ok(())
        }
    }

    fn GetTextBeforePosition(
        &self,
        text_position: u32,
        text_string: *mut *mut u16,
        text_length: *mut u32,
    ) -> windows::core::Result<()> {
        unsafe {
            *text_string = self.0.wide_text;
            *text_length = text_position;
            Ok(())
        }
    }

    fn GetParagraphReadingDirection(
        &self,
    ) -> windows::Win32::Graphics::DirectWrite::DWRITE_READING_DIRECTION {
        DWRITE_READING_DIRECTION_LEFT_TO_RIGHT
    }

    fn GetLocaleName(
        &self,
        text_position: u32,
        text_length: *mut u32,
        locale_name: *mut *mut u16,
    ) -> windows::core::Result<()> {
        unsafe {
            *locale_name = self.0.locale_name.as_ptr() as *mut _;
            Ok(())
        }
    }

    fn GetNumberSubstitution(
        &self,
        text_position: u32,
        text_length: *mut u32,
        number_substitution: *mut core::option::Option<IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        unsafe {
            *number_substitution = Some(self.0.num_subst.clone());
            Ok(())
        }
    }
}

//struct DWriteRunAnalysisSinkInner {
//    script_analysis: Vec<(Range<usize>, *const DWRITE_SCRIPT_ANALYSIS)>,
//    bidi_levels: Vec<(Range<usize>, BidiLevels)>,
//    number_substitution: Vec<(Range<usize>, IDWriteNumberSubstitution)>,
//}

struct DWriteRunAnalysisSinkData {
    runs: Cell<Vec<TextAnalyzerRun>>,
    last_run_index: Cell<usize>,
}

impl DWriteRunAnalysisSinkData {
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
                script_analysis: None,
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
        while run_end < runs.len() && runs[run_end].text_range.start < text_end {
            run_end += 1;
        }
        if runs[run_end - 1].text_range.end != text_end {
            split_run(&mut runs, run_end - 1, text_end);
        }

        self.runs.replace(runs);
        self.last_run_index.set(run_start);

        run_start..run_end
    }
}

#[implement(IDWriteTextAnalysisSink)]
struct DWriteRunAnalysisSink(Rc<DWriteRunAnalysisSinkData>);

impl IDWriteTextAnalysisSink_Impl for DWriteRunAnalysisSink {
    fn SetScriptAnalysis(
        &self,
        text_position: u32,
        text_length: u32,
        script_analysis: *const DWRITE_SCRIPT_ANALYSIS,
    ) -> windows::core::Result<()> {
        let script_analysis = unsafe { *script_analysis };
        //eprintln!("SetScriptAnalysis({}, {}, {:?})",
        //    text_position, text_length, script_analysis);

        let range = self.0.get_run_range(text_position, text_length);

        let mut runs = self.0.runs.take();
        for run in &mut runs[range] {
            run.backend.script_analysis = Some(script_analysis);
        }

        self.0.runs.replace(runs);
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
        //eprintln!("SetBidiLevel({}, {}, lv: {})", text_position, text_length, resolved_level);

        let range = self.0.get_run_range(text_position, text_length);

        let mut runs = self.0.runs.take();
        for run in &mut runs[range] {
            run.backend.bidi_levels = Some(BidiLevels {
                explicit_level,
                resolved_level,
            });
        }

        self.0.runs.replace(runs);
        Ok(())
    }

    fn SetNumberSubstitution(
        &self,
        text_position: u32,
        text_length: u32,
        number_substitution: &Option<IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        //eprintln!("SetNumberSubstitution({}, {})", text_position, text_length);

        let range = self.0.get_run_range(text_position, text_length);

        let mut runs = self.0.runs.take();
        if let Some(num_subst) = number_substitution {
            for run in &mut runs[range] {
                run.backend.number_substitution = Some(num_subst.clone());
            }
        }

        self.0.runs.replace(runs);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;
    use std::rc::Rc;

    use windows::Win32::Graphics::DirectWrite::IDWriteTextAnalysisSink_Impl;

    use crate::backend::text_analyzer_backend::DWriteRunAnalysisSinkData;

    use super::DWriteRunAnalysisSink;

    fn set_bidi_levels(sink: &DWriteRunAnalysisSink, level_range_calls: &[(u8, Range<usize>)]) {
        for (level, range) in level_range_calls {
            sink.SetBidiLevel(range.start as u32, (range.end - range.start) as u32, 0, *level)
                .unwrap();
        }
    }

    fn get_bidi_levels(sink: &DWriteRunAnalysisSinkData) -> Vec<(u8, Range<usize>)> {
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
        let sink_data = Rc::new(DWriteRunAnalysisSinkData::new());
        let sink = DWriteRunAnalysisSink(sink_data.clone());
        sink_data.clear_and_resize(16);
        let bidi_level_range_calls = &[
            (0, 0..16),
        ];
        set_bidi_levels(&sink, bidi_level_range_calls);
        assert_eq!(get_bidi_levels(&sink_data), &[
            (0, 0..16),
        ]);
    }

    #[test]
    fn test_dwrite_run_analysis_sink_2() {
        let sink_data = Rc::new(DWriteRunAnalysisSinkData::new());
        let sink = DWriteRunAnalysisSink(sink_data.clone());
        sink_data.clear_and_resize(23);
        let bidi_level_range_calls = &[
            (2, 0..6),
        ];
        set_bidi_levels(&sink, bidi_level_range_calls);
        assert_eq!(get_bidi_levels(&sink_data), &[
            (2, 0..6),
            (0, 6..23),
        ]);
    }

    #[test]
    fn test_dwrite_run_analysis_sink_3() {
        let sink_data = Rc::new(DWriteRunAnalysisSinkData::new());
        let sink = DWriteRunAnalysisSink(sink_data.clone());
        sink_data.clear_and_resize(23);
        let bidi_level_range_calls = &[
            (2, 4..7),
            (3, 0..4),
            (4, 7..12),
            (5, 15..20),
            (6, 10..18),
        ];
        set_bidi_levels(&sink, bidi_level_range_calls);
        assert_eq!(get_bidi_levels(&sink_data), &[
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

pub struct TextAnalyzerBackend {
    text: String,
    wide_text: WideFfiString<[u16; 8]>,
    //analysis_source: DWriteAnalysisSource,
    run_analysis_sink_data: Rc<DWriteRunAnalysisSinkData>,
    run_analysis_sink: IDWriteTextAnalysisSink,
}

impl Debug for TextAnalyzerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextAnalyzerBackend")
            .field("text", &self.text)
            .finish()
    }
}

impl GenericTextAnalyzerBackend for TextAnalyzerBackend {
    fn new(text: String) -> Self {
        let wide_text = WideFfiString::new(&text);
        let run_analysis_sink_data = Rc::new(DWriteRunAnalysisSinkData::new());
        let run_analysis_sink: IDWriteTextAnalysisSink =
            DWriteRunAnalysisSink(run_analysis_sink_data.clone()).into();
        Self {
            text,
            wide_text,
            run_analysis_sink_data,
            run_analysis_sink,
        }
    }

    fn text(&self) -> &str {
        &self.text
    }

    // TODO: should probably return an iterator so that the caller doesn't have to allocate
    // or maybe a &mut Vec?
    fn get_runs(&self) -> Vec<TextAnalyzerRun> {
        unsafe {
            self.run_analysis_sink_data.clear_and_resize(self.wide_text.len());
            let analyzer = DWRITE_FACTORY.with(|factory|
                factory.CreateTextAnalyzer().expect("CreateTextAnalyzer() failed")
            );

            let mut locale_name = [0; LOCALE_NAME_MAX_LENGTH as usize];
            // Get the thread locale every time in case it changes.
            let locale = GetThreadLocale();
            GetLocaleInfoEx(
                PCWSTR(ptr::null()), LOCALE_SNAME,
                &mut locale_name[..]
            );

            let num_subst = DWRITE_FACTORY.with(|factory|
                factory.CreateNumberSubstitution(DWRITE_NUMBER_SUBSTITUTION_METHOD_NONE,
                    PCWSTR(locale_name.as_ptr()),
                    true
                ).expect("CreateNumberSubstitution() failed")
            );
            let analysis_source_data = Rc::new(DWriteAnalysisSourceData {
                wide_text: self.wide_text.as_ptr() as *mut _,
                len: self.wide_text.len() as u32,
                locale_name,
                num_subst,
            });
            let analysis_source = DWriteAnalysisSource(analysis_source_data);

            let analysis_source: IDWriteTextAnalysisSource = analysis_source.into();

            analyzer.AnalyzeScript(
                analysis_source.clone(),
                0, self.wide_text.len() as u32,
                self.run_analysis_sink.clone()
            ).expect("AnalyzeScript() failed");
            analyzer.AnalyzeBidi(
                analysis_source.clone(),
                0, self.wide_text.len() as u32,
                self.run_analysis_sink.clone()
            ).expect("AnalyzeBidi() failed");
            analyzer.AnalyzeNumberSubstitution(
                analysis_source.clone(),
                0, self.wide_text.len() as u32,
                self.run_analysis_sink.clone()
            ).expect("AnalyzeNumberSubstitution() failed");

            let runs = self.run_analysis_sink_data.runs.take();
            self.run_analysis_sink_data.runs.set(runs.clone()); // TODO: get rid of clone

            // TODO: loop through run indexes and convert from UTF-16 to UTF-8

            runs
        }
    }
}