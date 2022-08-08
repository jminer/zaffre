use std::any::Any;
use std::backtrace::Backtrace;

use nalgebra::Point2;

use crate::font::Font;
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

    fn save(&mut self);

    fn restore(&mut self);

    fn translate(&mut self, x: f64, y: f64);

    fn scale(&mut self, x: f64, y: f64);

    fn draw_glyphs(
        &mut self,
        glyphs: &[u16],
        positions: &[nalgebra::Point2<f32>],
        origin: nalgebra::Point2<f32>,
        font: &Font,
        brush: &Brush,
    );

    // if adding a PDF backend, it would need to have this, like cairo and Skia:
    //fn draw_glyphs_with_text(
    //    &mut self,
    //    glyphs: &[u16],
    //    positions: &[Point2<f32>],
    //    origin: Point2<f32>,
    //    clusters: &[usize],
    //    text: &str,
    //    font: &Font,
    //    brush: &Brush,
    //);
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