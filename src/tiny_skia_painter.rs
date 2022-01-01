
use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::rc::Rc;

use tiny_skia::{Paint, PathBuilder, Pixmap, Shader, Stroke};

use crate::{Color, PathSegment};
use crate::painter::{Brush, Error, Painter};
use crate::path::{ArcSegment, LineCap, LineJoin, StrokeStyle};

pub struct TinySkiaPainter {
    pixmap: Rc<RefCell<Pixmap>>,
    err: Vec<Error>,
    transform_stack: Vec<tiny_skia::Transform>,
    transform: tiny_skia::Transform,
}

impl TinySkiaPainter {
    pub fn new(pixmap: Rc<RefCell<Pixmap>>) -> Self {
        Self {
            pixmap,
            err: Vec::new(),
            transform_stack: Vec::new(),
            transform: tiny_skia::Transform::identity(),
        }
    }

    fn color_to_color(color: Color<u8>) -> tiny_skia::Color {
        let (r, g, b, a) = color.as_rgba();
        tiny_skia::Color::from_rgba8(r, g, b, a)
    }

    fn brush_to_shader(brush: &Brush) -> Shader<'static> {
        match brush {
            Brush::Solid(color) => {
                Shader::SolidColor(Self::color_to_color(*color))
            },
            Brush::LinearGradient => todo!(),
            Brush::RadialGradient => todo!(),
            //Brush::Prepared(any) => *any.downcast_ref().expect("invalid prepared brush"),
            Brush::Prepared(any) => panic!("this backend does not use prepared brushes"),
        }
    }

    fn path_to_path(path: &mut dyn Iterator<Item=PathSegment>) -> Option<tiny_skia::Path> {
        let mut builder = PathBuilder::new();
        for seq in path {
            match seq {
                PathSegment::Move(p) => {
                    builder.move_to(p.x, p.y);
                }
                PathSegment::Line(p) =>  {
                    builder.line_to(p.x as f32, p.y as f32);
                }
                PathSegment::QuadCurve(p1, p2) => {
                    builder.quad_to(p1.x as f32, p1.y as f32, p2.x as f32, p2.y as f32);
                }
                PathSegment::CubicCurve(p1, p2, p3) => {
                    builder.cubic_to(
                        p1.x as f32,
                        p1.y as f32,
                        p2.x as f32,
                        p2.y as f32,
                        p3.x as f32,
                        p3.y as f32,
                    );
                }
                PathSegment::Arc(arc_seg) => {
                    // I can either call builder.conic_to() or I could remove Arc from the enum
                    todo!()
                },
                PathSegment::Close => builder.close(),

            }
        }
        builder.finish()
    }

    // fn transform_to_affine(affine: Affine) -> tiny_skia::Transform {
    //     let coeffs = affine.as_coeffs();
    //     tiny_skia::Transform::from_row(
    //         coeffs[0] as f32,
    //         coeffs[1] as f32,
    //         coeffs[2] as f32,
    //         coeffs[3] as f32,
    //         coeffs[4] as f32,
    //         coeffs[5] as f32,
    //     )
    // }

    fn line_cap_to_line_cap(end_cap: LineCap) -> tiny_skia::LineCap {
        match end_cap {
            LineCap::Flat => tiny_skia::LineCap::Butt,
            LineCap::Square => tiny_skia::LineCap::Square,
            LineCap::Round => tiny_skia::LineCap::Round,
        }
    }

    fn line_join_to_line_join(join_style: LineJoin) -> tiny_skia::LineJoin {
        match join_style {
            LineJoin::Round => tiny_skia::LineJoin::Round,
            LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
            LineJoin::Miter(_) => tiny_skia::LineJoin::Miter,
        }
    }

    // fn vec2_to_point(vec: Point2<f32>) -> tiny_skia::Point {
    //     tiny_skia::Point {
    //         x: vec.x,
    //         y: vec.y,
    //     }
    // }

}

impl<'a> Painter for TinySkiaPainter {
    fn solid_brush(&mut self, color: Color<u8>) -> Brush {
        Brush::Solid(color)
    }

    fn gradient_brush(&mut self, gradient: ()) -> Brush {
        todo!()
    }

    fn stroke_path(
        &mut self,
        path: &mut dyn Iterator<Item=PathSegment>,
        brush: &Brush,
        style: &StrokeStyle,
    ) {
        let path = match Self::path_to_path(path) {
            Some(path) => path,
            None => {
                self.err.push(Error::InvalidPath(Backtrace::capture()));
                return;
            }
        };
        let paint = Paint {
            shader: Self::brush_to_shader(brush),
            blend_mode: tiny_skia::BlendMode::SourceOver,
            anti_alias: true,
            force_hq_pipeline: false,
        };
        // let dash = if style.dash_pattern.is_empty() {
        //     None
        // } else {
        //     tiny_skia::StrokeDash::new(
        //         style.dash_pattern.iter().map(|&f| f as f32).collect(),
        //         style.dash_offset as f32,
        //     )
        // };
        let dash = None;
        let stroke = Stroke {
            width: style.width,
            miter_limit: style.line_join.miter_limit().unwrap_or(4.0) as f32,
            line_cap: Self::line_cap_to_line_cap(style.line_cap),
            line_join: Self::line_join_to_line_join(style.line_join),
            dash,
        };
        let mut pixmap = self.pixmap.borrow_mut();
        pixmap.stroke_path(&path, &paint, &stroke, self.transform, None);
    }

    fn clear(&mut self, color: Color<u8>) {
        let mut pixmap = self.pixmap.borrow_mut();
        pixmap.fill(Self::color_to_color(color))
    }

    fn save(&mut self) {
        self.transform_stack.push(self.transform);
    }

    fn restore(&mut self) {
        self.transform = self.transform_stack.pop()
            .expect("`restore` called more times than `save`");
    }

    fn translate(&mut self, x: f64, y: f64) {
        self.transform = self.transform.post_translate(x as f32, y as f32);
    }

    fn scale(&mut self, x: f64, y: f64) {
        self.transform = self.transform.post_scale(x as f32, y as f32);
    }

}