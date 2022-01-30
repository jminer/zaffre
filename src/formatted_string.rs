use crate::Color;
use crate::font::{OpenTypeFontWeight, FontStyle, FontWeight};


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
    FontStyle(FontStyle),
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
    font_family: String,
    font_size: f32,
    font_weight: OpenTypeFontWeight,
    font_style: FontStyle,
    underline: LineStyle,
    strikethrough: LineStyle,
    overline: LineStyle,
    small: SmallType,
    fore_color: Color<u8>,
    back_color: Color<u8>,
    spacing: f32,
}

impl Format {
    pub fn new(font_family: String, font_size: f32) -> Self {
        Self {
            font_family,
            font_size,
            font_weight: FontWeight::Normal.into(),
            font_style: FontStyle::Normal,
            underline: LineStyle::None,
            strikethrough: LineStyle::None,
            overline: LineStyle::None,
            small: SmallType::Normal,
            fore_color: Color::from_rgba(0, 0, 0, 255),
            back_color: Color::from_rgba(0, 0, 0, 255),
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
    pub fn new(font_family: String, font_size: f32) -> Self {
        Self {
            text: String::new(),
            formatting: Vec::new(),
            initial_format: Format::new(font_family, font_size),
        }
    }

    pub fn clear_and_set_text(&mut self, text: String) {
        self.formatting.clear();
        self.text = text;
    }
}

