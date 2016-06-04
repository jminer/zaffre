#![feature(test)]
extern crate test;

#[macro_use]
extern crate nalgebra;
extern crate num;

mod bezier;
mod coordinates;

pub use nalgebra::Point2;
pub use bezier::{Bezier, CurveType};
pub use coordinates::{Size2, Rect, BorderSize2};
