
use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::rc::Rc;

use glam::Affine2;
use nalgebra::Point2;
use smallvec::SmallVec;
use tiny_skia::{Paint, PathBuilder, Pixmap, Shader, Stroke};

use crate::color::{srgb_to_linear, linear_to_srgb};
use crate::font::{Font, GlyphImageFormat};
use crate::{Color, PathSegment};
use crate::painter::{Brush, Error, Painter};
use crate::path::{ArcSegment, LineCap, LineJoin, StrokeStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinySkiaPainterByteOrder {
    Rgba,
    Bgra,
}

pub struct TinySkiaPainter {
    pixmap: Rc<RefCell<Pixmap>>,
    byte_order: TinySkiaPainterByteOrder,
    err: Vec<Error>,
    transform_stack: Vec<tiny_skia::Transform>,
    transform: tiny_skia::Transform,
}

impl TinySkiaPainter {
    pub fn new(pixmap: Rc<RefCell<Pixmap>>, byte_order: TinySkiaPainterByteOrder) -> Self {
        Self {
            pixmap,
            byte_order,
            err: Vec::new(),
            transform_stack: Vec::new(),
            transform: tiny_skia::Transform::identity(),
        }
    }

    fn color_to_color(color: Color<u8>, byte_order: TinySkiaPainterByteOrder) -> tiny_skia::Color {
        let (r, g, b, a) = color.as_rgba();
        let (r, b) = if byte_order == TinySkiaPainterByteOrder::Rgba { (r, b) } else { (b, r) };
        tiny_skia::Color::from_rgba8(r, g, b, a)
    }

    fn brush_to_shader(brush: &Brush, byte_order: TinySkiaPainterByteOrder) -> Shader<'static> {
        match brush {
            Brush::Solid(color) => {
                Shader::SolidColor(Self::color_to_color(*color, byte_order))
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
            shader: Self::brush_to_shader(brush, self.byte_order),
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
        pixmap.fill(Self::color_to_color(color, self.byte_order))
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

    // The `origin` is the position of the text's baseline
    fn draw_glyphs(
        &mut self,
        glyphs: &[u16],
        positions: &[nalgebra::Point2<f32>],
        baseline_origin: nalgebra::Point2<f32>,
        font: &Font,
        brush: &Brush,
    ) {
        // DirectWrite, Skia, Core Graphics, and Qt all have the origin be the position of the
        // baseline. (Skia and Core Graphics don't say in their docs, but I tested Skia with
        // https://fiddle.skia.org/c/25dc79ab3c8586f7a01c50e610a3d161 and found a Core Graphics code
        // example.)
        let color: Color<u8> = match brush {
            Brush::Solid(c) => *c,
            // TODO: pick one color from gradients instead of panicking?
            // I need a way to set a tiny-skia ClipMask to an A8 image to implement other brushes.
            // Then it could set the clip mask and fill a rect with the brush.
            _ => panic!("only solid color text is implemented for TinySkiaPainter"),
        };
        let color = match self.byte_order {
            TinySkiaPainterByteOrder::Bgra =>
                Color::from_rgba(color.blue, color.green, color.red, color.alpha),
            TinySkiaPainterByteOrder::Rgba => color,
        };
        let color_lin = color.to_linear();
        // TODO: write an object that caches rendered glyphs in a font atlas
        // it only stores them if the transform has no rotation or shear
        let offsets =
            positions.iter().map(|pos| Point2::new(pos.x.fract(), pos.y.fract()))
            .collect::<SmallVec<[_; 32]>>();
        let glyph_images = font.draw_glyphs(glyphs, &offsets, Affine2::IDENTITY);
        let mut pixmap = self.pixmap.borrow_mut();
        let pixmap_width = pixmap.width();
        let pixel_data = pixmap.data_mut();
        for (i, glyph_image) in glyph_images.iter().enumerate() {
            let ts_points = &mut [tiny_skia::Point {
                x: baseline_origin.x + positions[i].x,
                y: baseline_origin.y + positions[i].y
            }];
            self.transform.map_points(ts_points);
            let pos = Point2::new(ts_points[0].x, ts_points[0].y);
            dbg!(pos);
            let ipos = Point2::new(pos.x.floor() as u32, pos.y.floor() as u32);
            const PIXMAP_PIXEL_SIZE: usize = 4;
            match glyph_image.format {
                GlyphImageFormat::Alpha1x1 => {
                    for y in 0..glyph_image.bounding_size.height {
                        let row_ptr = unsafe {
                            glyph_image.data_ptr.add((glyph_image.stride * y) as usize)
                        };
                        for x in 0..glyph_image.bounding_size.width {
                            let glyph_alpha = unsafe {
                                *row_ptr.add(x as usize)
                            };
                            let src_alphaf = glyph_alpha as f32 * (1.0 / 255.0) * color_lin.alpha;
                            let one_minus_src_alphaf = 1.0 - src_alphaf;
                            let pixmap_index =
                                (pixmap_width * (ipos.y + y) + (ipos.x + x)) as usize * PIXMAP_PIXEL_SIZE;
                            let dest_r_lin = srgb_to_linear(pixel_data[pixmap_index+0]);
                            let dest_g_lin = srgb_to_linear(pixel_data[pixmap_index+1]);
                            let dest_b_lin = srgb_to_linear(pixel_data[pixmap_index+2]);
                            let dest_alphaf = pixel_data[pixmap_index+3] as f32 * (1.0 / 255.0);
                            // https://www.teamten.com/lawrence/graphics/premultiplication/
                            pixel_data[pixmap_index+0] = linear_to_srgb(
                                color_lin.red * src_alphaf + dest_r_lin * one_minus_src_alphaf);
                            pixel_data[pixmap_index+1] = linear_to_srgb(
                                color_lin.green * src_alphaf + dest_g_lin * one_minus_src_alphaf);
                            pixel_data[pixmap_index+2] = linear_to_srgb(
                                color_lin.blue * src_alphaf + dest_b_lin * one_minus_src_alphaf);
                            pixel_data[pixmap_index+3] =
                                ((src_alphaf + one_minus_src_alphaf * dest_alphaf) * 255.0) as u8;
                        }
                    }
                },
                GlyphImageFormat::Alpha3x1 => {
                    for x in 0..glyph_image.bounding_size.width {
                        for y in 0..glyph_image.bounding_size.height {
                            unsafe {
                                let glyph_pixel_ptr = glyph_image.data_ptr
                                    .add((glyph_image.stride * y + x) as usize * 3);
                                let glyph_alpha_r = glyph_pixel_ptr.add(0).read();
                                let glyph_alpha_g = glyph_pixel_ptr.add(1).read();
                                let glyph_alpha_b = glyph_pixel_ptr.add(2).read();
                                let pixmap_index =
                                    (pixmap_width * (ipos.y + y) + (ipos.x + x)) as usize * PIXMAP_PIXEL_SIZE;
                                // I know it's slower having the if inside the loop instead of
                                // duplicating the loop, but I don't care about subpixel AA.
                                if self.byte_order == TinySkiaPainterByteOrder::Rgba {
                                    pixel_data[pixmap_index+0] = 255-glyph_alpha_r;
                                    pixel_data[pixmap_index+1] = 255-glyph_alpha_g;
                                    pixel_data[pixmap_index+2] = 255-glyph_alpha_b;
                                } else if self.byte_order == TinySkiaPainterByteOrder::Bgra {
                                    pixel_data[pixmap_index+0] = 255-glyph_alpha_b;
                                    pixel_data[pixmap_index+1] = 255-glyph_alpha_g;
                                    pixel_data[pixmap_index+2] = 255-glyph_alpha_r;
                                }
                                dbg!(pixmap_index);
                            }
                        }
                    }
                },
                GlyphImageFormat::RgbaColor => todo!(),
                GlyphImageFormat::BgraColor => todo!(),
            }
        }

        println!("{:?}", glyph_images[0]);
    }

}
