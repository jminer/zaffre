#![feature(test)]
extern crate test;

#[macro_use]
extern crate glium;
#[macro_use]
extern crate nalgebra;
extern crate num;

mod bezier;
mod coordinates;
mod path;

pub use nalgebra::Point2;
pub use bezier::{Bezier, CurveType};
pub use coordinates::{Size2, Rect, BorderSize2};
pub use path::{PathSegment, PathBuf, stencil_stroke_path};
