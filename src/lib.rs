#![feature(backtrace)]
#![feature(bench_black_box)]
#![deny(unreachable_pub)]
#![feature(test)]
extern crate test;

extern crate ahash;
#[macro_use]
extern crate ash;
#[macro_use]
extern crate nalgebra;
extern crate num;
extern crate once_cell;
extern crate tiny_skia;
extern crate smallvec;
extern crate windows;

mod color;
mod coordinates;
mod cubic_bezier;
mod image_group;
mod path;
mod quad_bezier;
mod retained;
mod vk_allocator;
mod vk_descriptor_set_allocator;
mod vk_util;
mod painter;
mod tiny_skia_painter;
mod formatted_string;
mod text_analyzer;
mod text_layout;

mod generic_backend;

#[cfg(windows)]
#[path = "windows_backend/mod.rs"]
pub mod backend;
#[cfg(unix)]
#[path = "gtk_backend/mod.rs"]
pub mod backend;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;

pub mod font;

pub use nalgebra::{Point2, Vector2};
pub use color::Color;
pub use coordinates::{Size2, Rect, BorderSize2};
pub use cubic_bezier::{CubicBezier, CurveType};
pub use path::{PathSegment, PathBuf, StrokeStyle};
pub use quad_bezier::QuadBezier;
pub use retained::{DrawCommand, ImageBuf, LinearGradient, ScalingMode, RenderingBackend, SwapchainSurface};
pub use painter::{AsPathIter, Brush, Error, Painter, PainterExt};
pub use tiny_skia_painter::TinySkiaPainter;
pub use vk_util::VulkanGlobals;

pub mod text {
    pub use crate::formatted_string::{Format, LineStyle, FormattedString, SmallType};
    pub use crate::text_analyzer::TextAnalyzer;
    pub use crate::text_layout::TextLayout;
}

use nalgebra::{BaseFloat, Cast};

pub(crate) type AHashMap<K, V> = HashMap<K, V, BuildHasherDefault<ahash::AHasher>>;

pub trait LargerFloat: Sized {
    type Float: BaseFloat + Cast<Self> + Cast<f32>;
}

impl LargerFloat for f32 {
    type Float = f32;
}
impl LargerFloat for f64 {
    type Float = f64;
}
impl LargerFloat for i16 {
    type Float = f32;
}
impl LargerFloat for u16 {
    type Float = f32;
}
impl LargerFloat for i32 {
    type Float = f64;
}
impl LargerFloat for u32 {
    type Float = f64;
}
impl LargerFloat for i64 {
    type Float = f64;
}
impl LargerFloat for u64 {
    type Float = f64;
}
