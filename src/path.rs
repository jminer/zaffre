
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use coordinates::*;
use super::Point2;
use nalgebra::Vector2;

pub struct ArcSegment {
	center_pt: Point2<f32>,
	x_radius: f32,
	y_radius: f32,
	angle1: f32,
	angle2: f32,
}

pub enum PathSegment {
	Move(Point2<f32>),
	Line(Point2<f32>),
	QuadCurve(Point2<f32>, Point2<f32>), // cairo doesn't have quad, but D2D, Skia, and NVpr do
	CubicCurve(Point2<f32>, Point2<f32>, Point2<f32>),
	Arc(ArcSegment),
	Close,
}

enum PathSegmentType { // smaller than a PathSegment
    Move,
    Line,
    QuadCurve,
    CubicCurve,
	Arc,
    Close,
}

// should EndCap and JoinStyle be set here? They seem to be in NVpr.
pub enum EndCap {
    Flat,
    Square,
    Round,
    Triangular,
}

pub enum JoinStyle {
    None,
    Round,
    Bevel,
    // (the miter limit)
    MiterRevert(f32), // default for SVG, PostScript, PDF, Cairo, NVpr
    MiterTruncate(f32), // default for Flash, XPS, and Qt
}

struct QuadBezierVertex {
	position: f32,
}

struct Geometry<T> {
	vertices: Vec<T>,
	indices: Vec<u16>,
}

struct BakedStroke {
	solid_geo: Geometry<f32>,
	//quad_bezier_geo: Geometry<StrokeQuadBezierVertex>,
	backend: HashMap<usize, Arc<Any>>,
}

struct BakedFill {
	solid_geo: Geometry<f32>,
	//quad_bezier_geo: Geometry<FillQuadBezierVertex>,
	//cubic_bezier_geo: Geometry<FillCubicBezierVertex>,
	backend: HashMap<usize, Arc<Any>>,
}

pub struct PathBuf {
	// stored in separate arrays for memory efficiency
    seg_types: Vec<PathSegmentType>,
    seg_data: Vec<f32>,
	initial_end_cap: EndCap,
	terminal_end_cap: EndCap,
	// initial_dash_cap: EndCap,
	// terminal_dash_cap: EndCap,
	join_style: JoinStyle,

	baked_stroke: Option<BakedStroke>,
	baked_fill: Option<BakedFill>,
}

impl PathBuf {
	pub fn new() -> PathBuf {
		PathBuf {
			seg_types: vec![],
			seg_data: vec![],
			initial_end_cap: EndCap::Flat,
			terminal_end_cap: EndCap::Flat,
			join_style: JoinStyle::MiterRevert(4.0),
			baked_stroke: None,
			baked_fill: None,
		}
	}

	pub fn segments(&self) -> PathSegments {
		PathSegments {
			types: &self.seg_types,
			data: &self.seg_data,

			types_index: 0,
			data_index: 0,
		}
	}

	pub fn move_to(&mut self, pt: Point2<f32>) {
	}

	pub fn rel_move_to(&mut self, pt: Point2<f32>) {
	}

	pub fn line_to(&mut self, pt: Point2<f32>) {
	}

	pub fn rel_line_to(&mut self, pt: Point2<f32>) {
	}

	pub fn quad_curve_to(&mut self, pt1: Point2<f32>, pt2: Point2<f32>) {
	}

	pub fn rel_quad_curve_to(&mut self, pt1: Point2<f32>, pt2: Point2<f32>) {
	}

	pub fn cubic_curve_to(&mut self, pt1: Point2<f32>, pt2: Point2<f32>, pt3: Point2<f32>) {
	}

	pub fn rel_cubic_curve_to(&mut self, pt1: Point2<f32>, pt2: Point2<f32>, pt3: Point2<f32>) {
	}

	// `angle1` and `angle2` are clockwise from the X axis (of course).
	// see arc and arct in
	// http://www.adobe.com/products/postscript/pdfs/PLRM.pdf
	pub fn arc_to(&mut self, center_pt: Point2<f32>,
	              x_radius: f32, y_radius: f32, // use a Size for these?
	              angle1: f32, angle2: f32) {
	}

	pub fn tangent_arc_to(&mut self, pt1: Point2<f32>, pt2: Point2<f32>,
	                      x_radius: f32, y_radius: f32) {
	}

	pub fn current_point(&self) -> Option<Point2<f32>> {
	    if self.seg_types.is_empty() {
	        return None;
	    }
		let dlen = self.seg_data.len();
	    match self.seg_types[self.seg_types.len() - 1] {
	        PathSegmentType::Close => None,
	        PathSegmentType::Arc => {
				let mut center = Point2::new(self.seg_data[dlen - 6], self.seg_data[dlen - 5]);
				// https://en.wikipedia.org/wiki/Ellipse#Equations
				let angle2 = self.seg_data[dlen - 1];
				center += Vector2::new(self.seg_data[dlen - 4] * angle2.cos(),
				                       self.seg_data[dlen - 3] * angle2.sin());
				Some(center)
			},
	        _ => Some(Point2::new(self.seg_data[dlen - 2], self.seg_data[dlen - 1]))
	    }
	}
	// pub fn iter() -> Iterator {
	// }
	// pub fn flatten(float tolerance) -> Iterator {
	// }

	// pub fn from_svg_path_string(path_string: &str);
	// pub fn from_postscript_path_string(path_string: &str);

	// pub fn stroke_width(&self) -> f32;
	// pub fn set_stroke_width(&mut self, width: f32);

	// pub fn mask(&self) -> u32;
	// pub fn set_mask(&mut self, mask: u32);

	// pub fn is_point_in_fill(&self, pt: Point, mask: Option<u32>) -> bool;
	// pub fn is_point_in_stroke(&self, pt: Point) -> bool;

	// pub fn length(segments: Range<usize>) -> f32;
	// pub fn point_at_distance(segments: Range<usize>, distance: f32) -> (Point, f32, f32);

	fn bake_stroke(&mut self) {

	}

	fn bake_fill(&mut self) {

	}
}

pub struct PathSegments<'a> {
    types: &'a Vec<PathSegmentType>,
    data: &'a Vec<f32>,

	types_index: usize,
	data_index: usize,
}

impl<'a> Iterator for PathSegments<'a> {
	type Item = PathSegment;

	fn next(&mut self) -> Option<PathSegment> {
		if self.types_index == self.types.len() {
			return None;
		}

		let di = self.data_index;
		let types_index = self.types_index;
		self.types_index += 1;
		Some(match self.types[types_index] {
			PathSegmentType::Move => {
				let s = PathSegment::Move(Point2::new(self.data[di], self.data[di + 1]));
				self.data_index += 2;
				s
			},
			PathSegmentType::Line => {
				let s = PathSegment::Line(Point2::new(self.data[di], self.data[di + 1]));
				self.data_index += 2;
				s
			},
			PathSegmentType::QuadCurve => {
				let s = PathSegment::QuadCurve(Point2::new(self.data[di    ], self.data[di + 1]),
				                               Point2::new(self.data[di + 2], self.data[di + 3]));
				self.data_index += 4;
				s
			},
			PathSegmentType::CubicCurve => {
				let s = PathSegment::CubicCurve(Point2::new(self.data[di    ], self.data[di + 1]),
				                                Point2::new(self.data[di + 2], self.data[di + 3]),
				                                Point2::new(self.data[di + 4], self.data[di + 5]));
				self.data_index += 6;
				s
			},
			PathSegmentType::Arc => {
				let s = PathSegment::Arc(ArcSegment {
					center_pt: Point2::new(self.data[di], self.data[di + 1]),
					x_radius: self.data[di + 2],
					y_radius: self.data[di + 3],
					angle1: self.data[di + 4],
					angle2: self.data[di + 5],
				});
				self.data_index += 6;
				s
			},
			PathSegmentType::Close => {
				let s = PathSegment::Close;
				s
			},
		})
	}
}
