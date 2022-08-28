use std::ops::RangeBounds;
use std::rc::Rc;

use crate::Color;
use crate::font::{OpenTypeFontWeight, FontSlant, FontWeight, Font};


#[derive(Debug, Clone, Copy)]
pub enum LineStyle {
    None,
    Single,
    Double,
    Dotted,
    //Dashed,
    //Wavy,
}

#[derive(Debug, Clone, Copy)]
pub enum SmallType {
	/// Specifies normal text.
	Normal,
	/// Specifies text smaller than normal and raised above the normal baseline.
	Superscript,
	/// Specifies text smaller than normal and lowered below the normal baseline.
	Subscript,
}

#[derive(Debug, Clone)]
enum FormatChange {
    FontFamily(String),
    FontSize(f32),
    FontWeight(OpenTypeFontWeight),
    FontSlant(FontSlant),
    Underline(LineStyle),
    Strikethrough(LineStyle),
    Overline(LineStyle),
    Small(SmallType),
    ForeColor(Color<u8>),
    BackColor(Color<u8>),
    Spacing(f32),
}

#[derive(Debug, Clone)]
struct FormatChangeStart {
    index: usize,
    ty: FormatChange,
}

#[derive(Debug, Clone)]
pub struct Format {
    // The four formatting types that I don't think have obvious defaults are `Option`s so that they
    // can inherit from a context. I think it makes sense to be able to have a `FormattedString`
    // that doesn't have a font family or a color built-in.
    font: Option<Font>,
    // font_family: Option<Rc<String>>,
    // font_size: Option<f32>,
    // font_weight: OpenTypeFontWeight,
    // font_slant: FontSlant,
    underline: LineStyle,
    strikethrough: LineStyle,
    overline: LineStyle,
    small: SmallType,
    fore_color: Option<Color<u8>>,
    back_color: Option<Color<u8>>,
    spacing: f32,
}

impl Format {
    pub fn new() -> Self {
        Self {
            font: None,
            // font_family: None,
            // font_size: None,
            // font_weight: FontWeight::Normal.into(),
            // font_slant: FontSlant::Normal,
            underline: LineStyle::None,
            strikethrough: LineStyle::None,
            overline: LineStyle::None,
            small: SmallType::Normal,
            fore_color: None,
            back_color: None,
            // It may be easier to implement if this is fixed amount to add/sub between characters
            spacing: 1.0,
        }
    }
}

pub struct FormattedString {
    text: String,
    formatting: Vec<FormatChangeStart>,
    initial_format: Format,
}

impl FormattedString {
    pub fn new() -> Self {
        // this function used to take `font_family: String, font_size: f32` params
        Self {
            text: String::new(),
            formatting: Vec::new(),
            initial_format: Format::new(),
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }


    pub fn insert_str(&mut self, index: usize, text: &str) {
        self.text.insert_str(index, text);
        // TODO: adjust formatting vec
    }

    pub fn push_str(&mut self, text: &str) {
        self.insert_str(self.text.len(), text);
    }

    pub fn remove_range<R>(&mut self, range: R) where R: RangeBounds<usize> {
        self.text.drain(range);
        // TODO: adjust formatting vec
    }

    pub fn clear_and_set_text_from(&mut self, text: &String) {
        self.formatting.clear();
        self.text.clone_from(&text);
    }

    pub fn initial_font(&self) -> Option<&Font> {
        self.initial_format.font.as_ref()
    }

    pub fn set_initial_font(&mut self, font: Font) {
        self.initial_format.font = Some(font);
    }
}

impl Clone for FormattedString {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            formatting: self.formatting.clone(),
            initial_format: self.initial_format.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.text.clone_from(&source.text);
        self.formatting.clone_from(&source.formatting);
        self.initial_format.clone_from(&source.initial_format);
    }
}

