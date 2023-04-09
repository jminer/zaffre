use std::cell::Cell;
use std::iter;
use std::rc::Rc;

use bit_vec::BitVec;
use nalgebra::Point2;
use smallvec::SmallVec;

use crate::font::Font;
use crate::text::FormattedString;
use crate::{Painter, Rect, Color};
use crate::text_analyzer::{TextAnalyzer, TextAnalyzerGlyphRun, TextDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    // Defaults to left for LTR text and to right for RTL text.
    Natural,
    Left,
    Center,
    Right,
    // Text will be expanded so that it fills the available space and will be aligned to the edge on
    // both the left and right sides. The last line of justified text will be natural aligned.
    Justify,
}

#[derive(Debug, Clone)]
pub(crate) struct TextLayoutRun {
    glyph_run: TextAnalyzerGlyphRun,
    // The start of the run relative to the beginning of the line. The start of the run is on the
    // left for LTR text and on the right for RTL text. Whether the paragraph direction is LTR or
    // RTL doesn't have any affect.
    start_x: f32,
    // The end x isn't needed for drawing the run, but it makes it easier/faster to find which run a
    // point is in for hit testing.
    end_x: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextLayoutLine {
    bounds: Rect<f32>,
    // The baseline of the line relative to the top edge of the line's rect.
    baseline: f32,
    // The index of the first run in this line.
    first_run: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlyphPosition {
    run_index: usize,
    // The number of code units from the beginning of the current run.
    text_index: usize,
    // Indicates whether the glyph is the first glyph in a cluster. If not, you likely want to
    // ignore the text_index.
    is_cluster_start: bool,
    // The number of glyphs from the beginning of the current run.
    glyph_index: usize,
}

impl Default for GlyphPosition {
    fn default() -> Self {
        Self {
            run_index: 0,
            text_index: 0,
            is_cluster_start: true,
            glyph_index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlyphIter {
    pos: GlyphPosition,
}

impl GlyphIter {
    fn new() -> Self {
        Self {
            pos: GlyphPosition::default(),
        }
    }

    fn next(&mut self, runs: &[TextAnalyzerGlyphRun]) -> Option<GlyphPosition> {
        let ret = self.pos;

        if ret.run_index >= runs.len() {
            return None;
        }

        self.pos.glyph_index += 1;
        // Default to false, but then set to true any time the text index is increased.
        self.pos.is_cluster_start = false;
        if self.pos.glyph_index >= runs[self.pos.run_index].glyphs.len() {
            self.pos.run_index += 1;
            self.pos.glyph_index = 0;
            self.pos.text_index = 0;
            self.pos.is_cluster_start = true;
        } else {
            // We need to find if we need to increase the text index or if the new glyph is still
            // part of the same cluster of glyphs.
            let mut test_text_index = self.pos.text_index;
            loop {
                if test_text_index >= runs[self.pos.run_index].cluster_map.len() {
                    break;
                }
                let test_glyph_index = runs[self.pos.run_index].cluster_map[test_text_index];
                if test_glyph_index > self.pos.glyph_index {
                    break;
                }
                if test_glyph_index == self.pos.glyph_index {
                    self.pos.text_index = test_text_index;
                    self.pos.is_cluster_start = true;
                    break;
                }
                test_text_index += 1;
            }
        }

        Some(ret)
    }

    fn reset(&mut self, pos: GlyphPosition) {
        self.pos = pos;
    }
}


#[cfg(test)]
mod glyph_iter_tests {
    use crate::font;
    use crate::text_analyzer::{TextAnalyzer, TextAnalyzerGlyphRun};

    use super::GlyphIter;

    fn get_text_analyzer_glyph_runs(font_family: &str, text: &str) -> Vec<TextAnalyzerGlyphRun> {
        let font_family = font::get_family(font_family)
            .expect("couldn't find font");
        let font = font_family.get_styles()[0].get_font(15.0);

        let mut analyzer = TextAnalyzer::new();
        analyzer.set_text_from(&text.to_owned());
        let runs = analyzer.get_runs();
        runs.iter().map(|run|
            analyzer.get_glyphs_and_positions(run.text_range(), run.clone(), &font)
        ).collect::<Vec<_>>()
    }

    // TODO: I should add a test that doesn't start at the first glyph run
    #[test]
    fn test_glyph_iter_english() {
        let glyph_runs = get_text_analyzer_glyph_runs("DejaVu Sans", "Hi");
        assert_eq!(glyph_runs.len(), 1);

        let mut iter = GlyphIter::new();

        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 0);
        assert_eq!(pos.text_index, 0);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 1);
        assert_eq!(pos.text_index, 1);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(&glyph_runs);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_glyph_iter_hebrew() {
        let glyph_runs = get_text_analyzer_glyph_runs("DejaVu Sans", "בא");
        assert_eq!(glyph_runs.len(), 1);

        let mut iter = GlyphIter::new();

        // Hebrew characters are two bytes each in UTF-8.
        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 0);
        assert_eq!(pos.text_index, 0);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 1);
        assert_eq!(pos.text_index, 2);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(&glyph_runs);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_glyph_iter_devanagari() {
        // https://en.wikipedia.org/wiki/Devanagari#Vowel_diacritics
        //
        // I chose kā ki for the test string. It has 4 code points, each 3 bytes. The vowel
        // codepoints come after the consonant codepoints. They result in 4 glyphs, but not in the
        // same order because the second vowel glyph comes before the second consonant.
        let glyph_runs = get_text_analyzer_glyph_runs("Noto Sans Devanagari", "काकि");
        assert_eq!(glyph_runs.len(), 1);

        let mut iter = GlyphIter::new();

        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 0);
        assert_eq!(pos.text_index, 0);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 1);
        assert_eq!(pos.text_index, 0);
        assert_eq!(pos.is_cluster_start, false);

        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 2);
        assert_eq!(pos.text_index, 6);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(&glyph_runs).unwrap();
        assert_eq!(pos.glyph_index, 3);
        assert_eq!(pos.text_index, 6);
        assert_eq!(pos.is_cluster_start, false);

        let pos = iter.next(&glyph_runs);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_glyph_iter_second_run() {
        // Test with two glyph runs using Ukrainian and Hebrew.
        let glyph_runs = get_text_analyzer_glyph_runs("DejaVu Sans", "Ейאבא");
        assert_eq!(glyph_runs.len(), 2);

        let glyph_runs_hebrew = &glyph_runs[1..];
        let mut iter = GlyphIter::new();

        let pos = iter.next(glyph_runs_hebrew).unwrap();
        assert_eq!(pos.glyph_index, 0);
        assert_eq!(pos.text_index, 0);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(glyph_runs_hebrew).unwrap();
        assert_eq!(pos.glyph_index, 1);
        assert_eq!(pos.text_index, 2);
        assert_eq!(pos.is_cluster_start, true);

        let pos = iter.next(glyph_runs_hebrew).unwrap();
        assert_eq!(pos.glyph_index, 2);
        assert_eq!(pos.text_index, 4);
        assert_eq!(pos.is_cluster_start, true);
    }

}

struct LineMeasurement {
    // The index in the TextLayout's text at the end of the line.
    text_end: usize,
    // The total width of all glyphs in each rect in the line.
    rects_glyph_widths: SmallVec<[f32; 1]>,
    // The width of each set of consecutive same-direction runs on the line.
    same_direction_widths: SmallVec<[f32; 1]>,
    // The text index where text should be broken for each rect in the line.
    break_indexes: SmallVec<[usize; 1]>,
    // TODO:  For justification, I probably need to add how many places there are to distribute
    // unused space.
    height: f32,
    baseline: f32,
}

struct LineMeasurementAttempt {
    measurement: LineMeasurement,
    next_height: f32,
    next_baseline: f32,
}

// struct LayoutLineState {
//     height: f32,
//     baseline: f32,
//     line_rect_unused: SmallVec<[f32; 1]>,
// }

struct LayoutLineResults {
    next_height: f32,
}

pub trait TextFramer {
    // Returns all the rectangles within which to lay out text for a single line. There may be
    // multiple rectangles per line because a line could be broken up by a shape or image that text
    // is wrapped around. The height of each returned rectangle will equal the specified
    // `line_height`.
    //
    // The current position is tracked. Each time this method is called with `dry_run` false, the
    // position will be advanced so that the next call will return the rectangles for the subsequent
    // line. When `dry_run` is true, no state is updated by the call. Using `dry_run` true, text
    // layout may generally call this method multiple times with different line heights for a single
    // line to determine which line height to use.
    fn next_line_rects(&mut self, line_height: f32, dry_run: bool) -> SmallVec<[Rect<f32>; 1]>;
}

pub struct TextRectFramer {
    rect: Rect<f32>,
    y: f32,
}

impl TextRectFramer {
    pub fn new(rect: Rect<f32>) -> Self {
        Self {
            rect,
            y: 0.0,
        }
    }
}

impl TextFramer for TextRectFramer {
    fn next_line_rects(&mut self, line_height: f32, dry_run: bool) -> SmallVec<[Rect<f32>; 1]> {
        // For this implementation, I'm not sure if this is better, or ignoring self.rect.height is
        // better.
        if self.y + line_height > self.rect.bottom() {
            return SmallVec::new();
        }
        let line_box = Rect::new(self.rect.x, self.y, self.rect.width, line_height);
        if !dry_run {
            self.y += line_height;
        }
        let mut rects = SmallVec::new();
        rects.push(line_box);
        rects
    }
}

pub struct TextLayout {
    text: FormattedString,
    set_text_count: u32,
    // The paragraph direction, or base direction as HTML calls it, determines the order that runs
    // are laid out in a line. It is separate from the text direction within a run, which is the
    // direction the glyphs are laid out. And both are unrelated to the text alignment.
    // https://www.w3.org/International/articles/inline-bidi-markup/uba-basics#context
    // https://unicode.org/reports/tr9/#BD5
    paragraph_direction: TextDirection,
    paragraph_alignment: TextAlignment,
    analyzer: Option<TextAnalyzer>,
    lines: Vec<TextLayoutLine>,
    runs: Vec<TextLayoutRun>,
}

impl TextLayout {
    pub fn new() -> Self {
        Self {
            text: FormattedString::new(),
            set_text_count: 0,
            paragraph_direction: TextDirection::LeftToRight,
            paragraph_alignment: TextAlignment::Natural,
            analyzer: None,
            lines: Vec::new(),
            runs: Vec::new(),
        }
    }

    pub fn text(&self) -> &FormattedString {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut FormattedString {
        &mut self.text
    }

    pub fn set_text(&mut self, text: FormattedString) {
        self.text = text;
        self.set_text_count = self.set_text_count.saturating_add(1);
        if let Some(analyzer) = &mut self.analyzer {
            analyzer.set_text_from(self.text.text());
        }
    }

    pub fn set_text_from(&mut self, text: &FormattedString) {
        self.text.clone_from(text);
        self.set_text_count = self.set_text_count.saturating_add(1);
        if let Some(analyzer) = &mut self.analyzer {
            analyzer.set_text_from(self.text.text());
        }
    }

    pub fn paragraph_direction(&self) -> TextDirection {
        self.paragraph_direction
    }

    pub fn set_paragraph_direction(&mut self, paragraph_direction: TextDirection) {
        self.paragraph_direction = paragraph_direction;
    }

    // TODO: would just `alignment` be a better name?
    pub fn paragraph_alignment(&self) -> TextAlignment {
        self.paragraph_alignment
    }

    pub fn set_paragraph_alignment(&mut self, paragraph_alignment: TextAlignment) {
        self.paragraph_alignment = paragraph_alignment;
    }

    // When laying out a line, one of the most complicated parts is handling text in a different
    // direction than the alignment. If you have LTR text followed by RTL text, all left-aligned,
    // then when you shrink the available space for the line, text moves from the middle of the line
    // to the next line.


    // ## How to handle leading?
    //
    // In both Word and Chrome, every line has the entire leading. And when you select text, the
    // selection rect includes the leading. Chrome definitely distributes the leading 50% above and
    // 50% below the glyphs in a line. I don't know what the heck Word does.
    //
    // With size 20 Gabriola (metrics are 13.67 ascent, 6.33 descent, 14.0 leading):
    // - Chrome: 7 px leading + 14 px ascent + 6 px descent + 7 px leading = 34 px high lines
    // - Word: 12 px leading + 14 px ascent + 6 px descent + 2 px leading = 34 px high lines (??)
    //
    // I think putting half of the leading above and half below glyphs on a line looks good.


    // Instead of passing the TextFramer, it could take the rects that the TextFramer returned.
    // Either way works.
    fn measure_line(
        glyph_runs: &[TextAnalyzerGlyphRun],
        line_breaks: &BitVec,
        paragraph_direction: TextDirection,
        framer: &mut dyn TextFramer,
        height: f32,
        baseline: f32,
    ) -> LineMeasurementAttempt {
        debug_assert!(!glyph_runs.is_empty());
        let mut glyph_iter = GlyphIter::new();

        let mut rects_glyph_widths = SmallVec::<[f32; 1]>::new();
        let mut same_direction_widths = SmallVec::<[f32; 1]>::new();
        let mut break_indexes = SmallVec::<[usize; 1]>::new();

        let mut next_height = height;
        let mut next_baseline = baseline;
        let mut curr_direction = glyph_runs[0].run.direction;
        let mut same_direction_width = 0.0;

        let mut rects = framer.next_line_rects(height, true);
        if paragraph_direction == TextDirection::RightToLeft {
            rects.reverse();
        }
        for rect in rects {
            let mut found_line_break = false;
            let mut line_break_pos = glyph_iter.pos;
            let mut line_break_total_glyph_width = 0.0;

            // the total width of glyphs in the current line rect
            let mut rect_glyph_width = 0.0;
            let mut height_baseline_increased = false;
            while let Some(pos) = glyph_iter.next(glyph_runs) {
                if pos.glyph_index == 0 {
                    let glyph_run = &glyph_runs[pos.run_index];
                    let run_metrics = glyph_run.font.metrics();
                    let run_height = run_metrics.height();
                    // Add half the leading above and half below like explained above.
                    let run_ascent_plus = run_metrics.ascent + run_metrics.leading * 0.5;
                    if run_height > height || run_ascent_plus > baseline {
                        next_height = next_height.max(run_height);
                        next_baseline = next_baseline.max(run_ascent_plus);
                        height_baseline_increased = true;
                    }

                    if glyph_run.run.direction != curr_direction {
                        same_direction_widths.push(same_direction_width);
                        same_direction_width = 0.0;
                        curr_direction = glyph_run.run.direction;
                    }
                }

                let glyph_width = glyph_runs[pos.run_index].glyph_advances[pos.glyph_index];
                if rect_glyph_width + glyph_width > rect.width || height_baseline_increased {
                    glyph_iter.reset(line_break_pos);
                    rect_glyph_width = line_break_total_glyph_width;
                    break;
                }
                rect_glyph_width += glyph_width;

                let is_line_break = pos.is_cluster_start &&
                    line_breaks[glyph_runs[pos.run_index].text_range.start + pos.text_index];
                found_line_break |= is_line_break;
                // If there isn't enough room to get to the first line break position, just put as
                // many characters as possible in the line rect.
                if !found_line_break || is_line_break {
                    line_break_pos = pos;
                    line_break_total_glyph_width = rect_glyph_width;
                }
            }
            rects_glyph_widths.push(rect_glyph_width);
            break_indexes.push(line_break_pos.text_index);
        }
        same_direction_widths.push(same_direction_width);

        LineMeasurementAttempt {
            measurement: LineMeasurement {
                text_end: glyph_runs[0].text_range.start + glyph_iter.pos.text_index,
                rects_glyph_widths,
                same_direction_widths,
                break_indexes,
                height,
                baseline,
            },
            next_height,
            next_baseline,
        }
    }

    fn measure_line_max(
        glyph_runs: &[TextAnalyzerGlyphRun],
        line_breaks: &BitVec,
        paragraph_direction: TextDirection,
        framer: &mut dyn TextFramer,
    ) -> LineMeasurement {
        let font = &glyph_runs.first()
            .expect("tried to measure line with no glyph runs")
            .font;

        let mut attempt: Option<LineMeasurementAttempt> = None;
        loop {
            let (height, baseline) = attempt.as_ref()
                .map(|m| (m.next_height, m.next_baseline))
                .unwrap_or_else(|| {
                    let metrics = font.metrics();
                    (metrics.height(), metrics.ascent)
                });
            let new_attempt = Self::measure_line(
                &glyph_runs, &line_breaks, paragraph_direction, framer,
                height, baseline
            );
            if let Some(ref mut attempt) = attempt {
                if new_attempt.measurement.text_end > attempt.measurement.text_end {
                    *attempt = new_attempt;
                    if attempt.measurement.height == attempt.next_height &&
                        attempt.measurement.baseline == attempt.next_baseline
                    {
                        // There is no reason to loop and call measure_line() again if the height
                        // and baseline are the same. We'd just get the same result.
                        break;
                    }
                } else {
                    break;
                }
            } else {
                attempt = Some(new_attempt);
            }
        }
        // The unwrap can't happen because the first loop iteration always sets it.
        attempt.unwrap().measurement
    }

    // I could have measure_line and layout_line functions. The measure line is called one or more
    // times to measure how much text fits in each line box and the line as a whole. It just loops
    // through text in logical order and when it runs out of room, jumps back to the previous line
    // break. If it finds a taller run, it returns the taller height and can be called with the
    // taller height to try laying out more. It doesn't do anything special for centered or
    // justified text.
    //
    // The layout_line function takes the height, baseline, and unused space from the measure_line
    // function, and positions and splits the runs.


    fn position_run(&mut self, run: TextAnalyzerGlyphRun, x: f32, width: f32) {
        let (start_x, end_x) = if run.run.direction == TextDirection::LeftToRight {
            (x, x + width)
        } else {
            (x + width, x)
        };
        self.runs.push(TextLayoutRun {
            glyph_run: run,
            start_x,
            end_x,
        });
    }

    fn layout_line(
        &mut self,
        glyph_runs: &mut Vec<TextAnalyzerGlyphRun>,
        first_run: &mut usize,
        line_breaks: &BitVec,
        framer: &mut dyn TextFramer,
        measurement: LineMeasurement,
        alignment: TextAlignment,
    ) {
        // TODO: I could detect that even though this is a dry run, that there's no need to call it
        // again (ending by running out of space, not by getting too tall of a run) and switch to a
        // non-dry run. Just have to tell the caller. I should measure how much it improves
        // performance. Also, center and justified need two passes through the runs, so maybe I'll
        // always need to dry run them anyway.

        fn start_x(alignment: TextAlignment, dir: TextDirection, unused_space: f32) -> f32 {
            match (alignment, dir) {
                // Natural should be converted to an actual alignment first.
                (TextAlignment::Natural, _) => panic!("invalid alignment"),
                (TextAlignment::Left, TextDirection::LeftToRight) => 0.0,
                (TextAlignment::Left, TextDirection::RightToLeft) => unused_space,
                (TextAlignment::Right, TextDirection::LeftToRight) => unused_space,
                (TextAlignment::Right, TextDirection::RightToLeft) => 0.0,
                (TextAlignment::Center, _) => unused_space * 0.5,
                (TextAlignment::Justify, _) => 0.0,
            }
        }

        debug_assert!(!glyph_runs.is_empty());
        let mut glyph_iter = GlyphIter::new();

        let dir_factor = if self.paragraph_direction == TextDirection::LeftToRight {
            1.0
        } else {
            -1.0
        };

        let mut rects = framer.next_line_rects(measurement.height, false);
        if self.paragraph_direction == TextDirection::RightToLeft {
            rects.reverse();
        }
        debug_assert_eq!(measurement.rects_glyph_widths.len(), rects.len());
        debug_assert_eq!(measurement.break_indexes.len(), rects.len());
        for (rect_index, rect) in rects.iter().enumerate() {
            let (rect_glyph_width, just_space) = if alignment == TextAlignment::Justify {
                (rect.width, measurement.rects_glyph_widths[rect_index] - rect.width)
            } else {
                (measurement.rects_glyph_widths[rect_index], 0.0)
            };

            let mut found_line_break = false;
            let mut line_break_pos = glyph_iter.pos;
            let mut line_break_rect_glyph_width = 0.0;

            // para_x moves in the paragraph direction. It isn't really used unless the text has
            // mixed directions.
            let para_x = todo!();
            // run_x moves the direction of consecutive same-direction runs
            let run_x = todo!();

            // - x starts on the left and increases if paragraph direction is LTR.
            // - x starts on the right and decreases if paragraph direction is RTL.
            let mut x = start_x(alignment, self.paragraph_direction, rect.width - rect_glyph_width);
            // the total width of glyphs in the current line rect
            let mut rect_glyph_width = 0.0;
            let mut run_glyph_width = 0.0;
            let mut prev_pos = None;
            while let Some(pos) = glyph_iter.next(&glyph_runs[*first_run..]) {
                // check width, and if too wide, go back to line break and if necessary, split run
                // glyph_iter should be the split-off run
                let is_break = pos.text_index == measurement.break_indexes[rect_index];
                if is_break {

                }

                let glyph_width = glyph_runs[pos.run_index].glyph_advances[pos.glyph_index];
                if rect_glyph_width + glyph_width > rect.width {
                    glyph_iter.reset(line_break_pos);
                    rect_glyph_width = line_break_rect_glyph_width;
                    todo!(); // have to reset run_glyph_width

                    // If not at a run boundary, we have to split the run.
                    let pos = glyph_iter.pos;
                    if pos.text_index != 0 {
                        let new_run = glyph_runs[pos.glyph_index].split_off(pos.text_index);
                        glyph_runs.insert(pos.run_index + 1, new_run);
                        glyph_iter.pos = GlyphPosition {
                            run_index: pos.run_index + 1,
                            ..Default::default()
                        };
                    }

                    self.position_run(glyph_runs[pos.glyph_index].clone(), x, run_glyph_width);

                    break;
                }
                rect_glyph_width += glyph_width;
                run_glyph_width += glyph_width;

                // When finished with a run, we have to position it.
                if pos.glyph_index == glyph_runs[pos.run_index].glyphs.len() - 1 {
                    self.position_run(glyph_runs[pos.glyph_index].clone(), x, run_glyph_width);
                    todo!(); // can't add rect_glyph_width to x
                    x += run_glyph_width * dir_factor;
                    run_glyph_width = 0.0;
                }

                // TODO: write test that fails if this is checked earlier in the loop
                // (and test(s) that fail if other checks are different order)
                let is_line_break = pos.is_cluster_start &&
                    line_breaks[glyph_runs[pos.run_index].text_range.start + pos.text_index];
                found_line_break |= is_line_break;
                // If there isn't enough room to get to the first line break position, just put as
                // many characters as possible in the line rect.
                if !found_line_break || is_line_break {
                    line_break_pos = pos;
                    line_break_rect_glyph_width = rect_glyph_width;
                }

                prev_pos = Some(pos);
            }
        }
        // TODO: it would be nice to position the run at the last glyph in the loop so that I don't have to here

        // TODO: set first_run

        let line_bounds = rects.iter().fold(rects[0], |acc, r| acc.union(*r));
        let line = TextLayoutLine {
            bounds: line_bounds,
            baseline: measurement.baseline,
            first_run: *first_run as u32,
        };
        self.lines.push(line);

        // if !dry_run {
        //     self.lines.push(TextLayoutLine {
        //         rect: Rect::new(line_rect),
        //         baseline,
        //         first_run: (),
        //     });
        // }
    }

    pub fn layout(&mut self, framer: &mut dyn TextFramer) {
        self.ensure_analyzer();
        let analyzer = self.analyzer.as_ref().unwrap(); // set by previous line
        let ana_runs = analyzer.get_runs();
        let alignment = if self.paragraph_alignment == TextAlignment::Natural {
            ana_runs.first().map(|run| match run.direction() {
                TextDirection::LeftToRight => TextAlignment::Left,
                TextDirection::RightToLeft => TextAlignment::Right,
            }).unwrap_or(TextAlignment::Left)
        } else {
            self.paragraph_alignment
        };

        // Should I do something other than panic? I think this is a programming error, so
        // probably leave as is.
        let font = self.text.initial_font().expect("a text layout must have an initial font");

        // Convert the TextAnalyzerRuns to TextAnalyzerGlyphRuns. Since a glyph run consists of one
        // font, we have to split runs whenever the font (family, size, or style) changes.
        let mut glyph_runs = Vec::<TextAnalyzerGlyphRun>::with_capacity(ana_runs.len());
        // use itertools to merge indexes?
        // let split_indexes = run_indexes_iter.merge(fmt_indexes_iter).dedup()
        let fmt_indexes = iter::once(0usize);
        let mut fmt_indexes = fmt_indexes.peekable();
        for run in &ana_runs {
            let mut text_start = run.text_range.start;
            // Split any analyzer runs where font formatting changes.
            loop {
                fmt_indexes.next_if_eq(&run.text_range.start);
                let text_end = fmt_indexes
                    .next_if(|i| *i < run.text_range.end)
                    .unwrap_or(run.text_range.end);
                let glyph_run = analyzer.get_glyphs_and_positions(
                    text_start..text_end, run.clone(), &font
                );
                glyph_runs.push(glyph_run);
                if text_end == run.text_range.end {
                    break;
                }
                text_start = text_end;
            }
        }

        let line_breaks = analyzer.get_line_breaks();

        let mut start_run = 0;
        loop {
            let measurement = Self::measure_line_max(
                    &glyph_runs[start_run..], &line_breaks, self.paragraph_direction, framer);

            self.layout_line(
                &mut glyph_runs,
                &mut start_run,
                &line_breaks,
                framer,
                measurement,
                alignment,
            );

            break;
        }

        loop {
            // TODO: I would rather use let-else here, but it's not stable yet.
            // let line_rect = match framer.next_line_rect(font.size(), false) {
            //     Some(r) => r,
            //     None => break,
            // };
            // let mut layout_runs = Vec::new();
            // for run in &ana_runs {
            //     let glyph_run = analyzer.get_glyphs_and_positions(
            //         run.text_range.clone(), run.clone(), &font
            //     );
            //     layout_runs.push(TextLayoutRun {
            //         glyph_run,
            //         rect: line_rect,
            //     });

            // }
            // self.runs = layout_runs;
            break;
        }
        self.maybe_drop_analyzer();
    }

    pub fn draw(&self, painter: &mut dyn Painter) {
        // for line_index in 0..self.lines.len() {
        //     let line = self.lines[line_index];
        //     let end_run = self.lines.get(line_index + 1)
        //         .map(|line| line.first_run as usize)
        //         .unwrap_or(self.runs.len());
        //     for run in &self.runs[line.first_run as usize..end_run] {
        //         let mut x = run.start_x;
        //         let positions = run.glyph_run.glyph_advances.iter().map(|advance| {
        //             let pos = Point2::new(x, line.baseline);
        //             x += advance;
        //             pos
        //         }).collect::<SmallVec::<[_; 16]>>();
        //         painter.draw_glyphs(
        //             &run.glyph_run.glyphs,
        //             &positions,
        //             line.rect.top_left(),
        //             &run.glyph_run.font,
        //             &crate::Brush::Solid(Color::from_rgba(0, 0, 0, 255)),
        //         );
        //     }
        // }
    }

    // I wish I could return a &TextAnalyzer, but then current borrow checker will borrow the whole
    // object instead of just the one field.
    pub fn ensure_analyzer(&mut self) {
        if self.analyzer.is_none() {
            self.analyzer = Some({
                let mut a = TextAnalyzer::new();
                a.set_text_from(self.text.text());
                a
            });
        }
    }

    pub fn maybe_drop_analyzer(&mut self) {
        if self.set_text_count <= 1 {
            // Until we know this text is not static, free the TextAnalyzer when we aren't using it
            // to reduce memory usage.
            self.analyzer = None;
        }
    }

    // For DirectWrite, I think just use the cluster map. I think macOS and probably Pango have
    // functions to get caret offsets. I need to add a function to TextAnalyzer too.
}

#[cfg(test)]
mod text_layout_tests {
    use approx::assert_abs_diff_eq;

    use crate::text_analyzer::TextDirection;
    use crate::{font, Rect};
    use crate::text::FormattedString;

    use super::{TextLayout, TextRectFramer, TextAlignment};

    #[test]
    fn basic_text_layout_no_panic() {
        let font_family = font::get_family("DejaVu Sans")
            .expect("couldn't find font");
        let font = font_family.get_styles()[0].get_font(20.0);

        let mut text = FormattedString::new();
        text.clear_and_set_text_from(&"First".to_owned());
        text.set_initial_font(font);
        let mut layout = TextLayout::new();
        layout.set_text(text);
        layout.layout(&mut TextRectFramer::new(Rect::new(10.0, 10.0, 50_000.0, 30.0)));
    }

    fn basic_text_layout_two_ltr_runs_with_direction(dir: TextDirection) {
        eprintln!("checking direction {:?}", dir);

        let font_family = font::get_family("DejaVu Sans")
            .expect("couldn't find font");
        let font = font_family.get_styles()[0].get_font(20.0);

        let mut text = FormattedString::new();
        text.clear_and_set_text_from(&"ЕйJohn".to_owned());
        text.set_initial_font(font);
        let mut layout = TextLayout::new();
        layout.set_text(text);
        layout.set_paragraph_alignment(TextAlignment::Left);
        layout.set_paragraph_direction(dir);
        layout.layout(&mut TextRectFramer::new(Rect::new(10.0, 10.0, 50_000.0, 30.0)));

        assert_eq!(layout.lines.len(), 1);
        assert_eq!(layout.runs.len(), 2);

        assert_abs_diff_eq!(layout.runs[0].start_x, 0.0, epsilon = 0.5);
        assert_abs_diff_eq!(layout.runs[0].end_x, 27.0, epsilon = 0.5);

        assert_abs_diff_eq!(layout.runs[1].start_x, 27.0, epsilon = 0.5);
        assert_abs_diff_eq!(layout.runs[1].end_x, 74.0, epsilon = 0.5);
    }

    #[test]
    fn basic_text_layout_two_runs() {
        // The paragraph direction shouldn't change the order of consecutive LTR runs.
        basic_text_layout_two_ltr_runs_with_direction(TextDirection::LeftToRight);
        basic_text_layout_two_ltr_runs_with_direction(TextDirection::RightToLeft);
    }

    // TODO: test:
    // - line breaking with ltr run
    // - line breaking with rtl run
    // - line breaking with ltr para direction
    // - line breaking with rtl para direction
    // - para direction auto detect?
    // - that a taller run moves the baseline down
    // - that a shorter run leaves the baseline the same
    // - that a taller run the same character that get wrapped to the next line doesn't move the
    //   baseline down
    // - center align
    // - right align
    // - TextFramer doesn't return enough line rects to fit all text so layout has to stop
    // - a taller run making TextFramer return a narrower line that doesn't fit as much text, so it
    //   uses the previous measure_line() call
    // - glyphs are too large to fit in rects that TextFramer returns (like you'd easily get with a
    //   huge font size)
    // - line breaking when no line break has been found
    // - line breaking when on the glyph before the space
    // - line breaking when on the space glyph
    // - line breaking when on the glyph after the space
}
