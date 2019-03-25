#![feature(const_cstr_unchecked)]
#![feature(non_exhaustive)]
#![feature(re_rebalance_coherence)]
#![feature(test)]
extern crate test;

#[macro_use]
extern crate ash;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nalgebra;
extern crate num;
extern crate smallvec;
extern crate winapi;

mod color;
mod coordinates;
mod cubic_bezier;
mod path;
mod quad_bezier;
mod retained;
mod vk_util;

pub use nalgebra::{Point2, Vector2};
pub use color::Color;
pub use coordinates::{Size2, Rect, BorderSize2};
pub use cubic_bezier::{CubicBezier, CurveType};
pub use path::{PathSegment, PathBuf, stencil_stroke_path};
pub use quad_bezier::QuadBezier;
pub use retained::{DrawCommand, Image, Scene, Surface, ScalingMode};

use nalgebra::{BaseFloat, Cast};

trait LargerFloat: Sized {
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
