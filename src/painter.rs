use std::any::Any;
use std::backtrace::Backtrace;

use crate::{Color, PathSegment};
use crate::path::StrokeStyle;

pub enum Error {
    InvalidPath(Backtrace),
}

pub enum Brush {
    Solid(Color<u8>),
    LinearGradient,
    RadialGradient,
    Prepared(Box<dyn Any>),
}

pub trait AsPathIter {
    type IterType: Iterator<Item = PathSegment>;
    fn path_iter(&self) -> Self::IterType;
}

// Unlike piet::RenderContext, this trait is object safe, which is necessary to use it as a trait
// object.
pub trait Painter {


    fn solid_brush(&mut self, color: Color<u8>) -> Brush;

    fn gradient_brush(&mut self, gradient: ()) -> Brush;

    fn stroke_path(
        &mut self,
        shape: &mut dyn Iterator<Item=PathSegment>,
        brush: &Brush,
        style: &StrokeStyle,
    );

    fn clear(&mut self, color: Color<u8>);

}

trait ToPath {

}

pub trait PainterExt<'a> {
    fn stroke<PI: AsPathIter>(
        &'a mut self,
        shape: &PI,
        brush: &Brush,
        style: &StrokeStyle,
    );
}

impl<'a> PainterExt<'a> for dyn Painter {
    fn stroke<PI: AsPathIter>(
        &'a mut self,
        shape: &PI,
        brush: &Brush,
        style: &StrokeStyle,
    ) {
        self.stroke_path(&mut shape.path_iter(), brush, style)
    }
}