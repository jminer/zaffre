use std::cell::Cell;
use std::rc::Rc;

use nalgebra::Point2;
use smallvec::SmallVec;

use crate::text::FormattedString;
use crate::{Painter, Rect, Color};
use crate::text_analyzer::{TextAnalyzer, TextAnalyzerGlyphRun};


pub(crate) struct TextLayoutRun {
    glyph_run: TextAnalyzerGlyphRun,
    rect: Rect<f32>,
}

pub trait TextFramer {
    fn next_line_rect(&mut self, line_height: f32, dry_run: bool) -> Option<Rect<f32>>;
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
    fn next_line_rect(&mut self, line_height: f32, dry_run: bool) -> Option<Rect<f32>> {
        // For this implementation, I'm not sure if this is better, or ignoring self.rect.height is
        // better.
        if self.y + line_height > self.rect.bottom() {
            return None;
        }
        let line_box = Rect::new(self.rect.x, self.y, self.rect.width, line_height);
        if !dry_run {
            self.y += line_height;
        }
        Some(line_box)
    }
}

pub struct TextLayout {
    text: FormattedString,
    set_text_count: u32,
    analyzer: Option<TextAnalyzer>,
    runs: Vec<TextLayoutRun>,
}

impl TextLayout {
    pub fn new() -> Self {
        Self {
            text: FormattedString::new(),
            set_text_count: 0,
            analyzer: None,
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

    pub fn layout(&mut self, framer: &mut dyn TextFramer) {
        self.ensure_analyzer();
        let analyzer = self.analyzer.as_ref().unwrap(); // set by previous line
        let ana_runs = analyzer.get_runs();
        loop {
            // Should I do something other than panic? I think this is a programming error, so
            // probably leave as is.
            let font = self.text.initial_font().expect("a text layout must have an initial font");
            // TODO: I would rather use let-else here, but it's not stable yet.
            let line_rect = match framer.next_line_rect(font.size(), false) {
                Some(r) => r,
                None => break,
            };
            let mut layout_runs = Vec::new();
            for run in &ana_runs {
                let glyph_run = analyzer.get_glyphs_and_positions(
                    run.text_range.clone(), run.clone(), &font
                );
                layout_runs.push(TextLayoutRun {
                    glyph_run,
                    rect: line_rect,
                });

            }
            self.runs = layout_runs;
            break;
        }
        self.maybe_drop_analyzer();
    }

    pub fn draw(&self, painter: &mut dyn Painter) {
        for run in &self.runs {
            let mut x = run.rect.x;
            let positions = run.glyph_run.glyph_advances.iter().map(|advance| {
                let pos = Point2::new(x, run.rect.y);
                x += advance;
                pos
            }).collect::<SmallVec::<[_; 16]>>();
            // TODO: the baseline here is really wrong
            painter.draw_glyphs(&run.glyph_run.glyphs, &positions, run.rect.top_left(), &run.glyph_run.font, &crate::Brush::Solid(Color::from_rgba(0, 0, 0, 255)));
        }
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
