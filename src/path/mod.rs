
use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;
use coordinates::*;
use crate::painter::AsPathIter;

use super::{Point2, QuadBezier};
use nalgebra::{ApproxEq, Cross, origin, Norm, Vector2};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineCap {
    Flat,
    Square,
    Round,
    //Triangular,
}

#[derive(Clone, Copy, Debug)]
pub enum LineJoin {
    //None,
    Round,
    Bevel,
    // (the miter limit)
    Miter(f32),
    //MiterRevert(f32), // default for SVG, PostScript, PDF, Cairo, NVpr
    //MiterTruncate(f32), // default for Flash, XPS, and Qt
}

impl LineJoin {
    pub fn miter_limit(&self) -> Option<f32> {
        if let LineJoin::Miter(limit) = self {
            Some(*limit)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct StrokeStyle {
    pub width: f32,
	pub line_cap: LineCap,
	pub line_join: LineJoin,
    // TODO: add dash style here

	//pub initial_end_cap: LineCap,
	//pub terminal_end_cap: LineCap,
	//pub initial_dash_cap: LineCap,
	//pub terminal_dash_cap: LineCap,
}

impl StrokeStyle {
	pub fn new() -> Self {
		Self {
            width: 1.0,
			line_cap: LineCap::Flat,
			line_join: LineJoin::Miter(4.0),
		}
	}

    pub fn with_width(width: f32) -> Self {
        Self { width, ..Self::new() }
    }

}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::new()
    }
}

// It may make sense to separate out stuff into another type: BakedStrokedPath or BakedStroke. When
// you create it, you specify:
// - the stroke width
// - the dash pattern
// - the end cap
// - the dash cap
// - the join style
// and then they are immutable. That would make the cost of changing them clear. And if you only
// are filling a path, you don't have those settings available. Would make the API more stateless.
// There should also be a BakedFilledPath. I don't think filling has any settings that affect the
// baking. The fill mode should be a parameter to stencil_fill_path().
// Something to keep in mind is I may want to be able to change a path and update the baked
// data without recreating it all from scratch. They might need to be one object in that case?
pub struct PathBuf {
	// stored in separate arrays for memory efficiency
    seg_types: Vec<PathSegmentType>,
    seg_data: Vec<f32>,
}

pub struct Path<'a> {
	// stored in separate arrays for memory efficiency
    seg_types: &'a [PathSegmentType],
    seg_data: &'a [f32],
}

impl<'a> Path<'a> {

    // See https://www.khronos.org/registry/OpenGL/extensions/NV/NV_path_rendering.txt
    // 6.X.4. Path Object Geometric Queries for description of mask.

	// pub fn is_point_in_fill(&self, pt: Point, mask: Option<u32>) -> bool;
	// pub fn is_point_in_stroke(&self, pt: Point) -> bool;

	// pub fn length(segments: Range<usize>) -> f32;
	// pub fn point_at_distance(segments: Range<usize>, distance: f32) -> (Point, f32, f32);

}

impl<'a> AsPathIter for Path<'a> {
    type IterType = PathIter<'a>;

    fn path_iter(&self) -> PathIter<'a> {
        PathIter {
            types: self.seg_types,
            data: self.seg_data,
            types_index: 0,
            data_index: 0,
        }
    }
}

impl PathBuf {
	pub fn new() -> PathBuf {
		PathBuf {
			seg_types: vec![],
			seg_data: vec![],
		}
	}

    pub fn as_path(&self) -> Path {
        Path {
            seg_types: &self.seg_types,
            seg_data: &self.seg_data,
        }
    }

    pub fn path_iter(&self) -> PathIter<'_> {
        PathIter {
            types: &self.seg_types,
            data: &self.seg_data,
            types_index: 0,
            data_index: 0,
        }
    }

	pub fn move_to(&mut self, pt: Point2<f32>) {
        self.seg_types.push(PathSegmentType::Move);
        self.seg_data.push(pt.x);
        self.seg_data.push(pt.y);
	}

	pub fn rel_move_to(&mut self, pt: Point2<f32>) {
	}

	pub fn line_to(&mut self, pt: Point2<f32>) {
        self.current_point().expect("line_to requires a current point");
        self.seg_types.push(PathSegmentType::Line);
        self.seg_data.push(pt.x);
        self.seg_data.push(pt.y);
	}

	pub fn rel_line_to(&mut self, pt: Point2<f32>) {
	}

	pub fn quad_curve_to(&mut self, pt1: Point2<f32>, pt2: Point2<f32>) {
        self.current_point().expect("quad_curve_to requires a current point");
        self.seg_types.push(PathSegmentType::QuadCurve);
        self.seg_data.push(pt1.x);
        self.seg_data.push(pt1.y);
        self.seg_data.push(pt2.x);
        self.seg_data.push(pt2.y);
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
	// pub fn flatten(float tolerance) -> Iterator {
	// }

	// pub fn from_svg_path_string(path_string: &str);
	// pub fn from_postscript_path_string(path_string: &str);

	// pub fn mask(&self) -> u32;
	// pub fn set_mask(&mut self, mask: u32);

}

// Returns the point at which the two specified lines intersect. The first line passes
// through `pt0` with the slope of `vec0`, and the second line passes through `pt1` with the
// slope of `vec1`.
fn find_intersection(pt0: Point2<f32>, vec0: Vector2<f32>, pt1: Point2<f32>, vec1: Vector2<f32>)
                        -> Option<Point2<f32>> {
    // http://stackoverflow.com/a/565282/69671
    fn cross_vec2(v0: Vector2<f32>, v1: Vector2<f32>) -> f32 {
        v0.x * v1.y - v0.y * v1.x
    }

    let denom = cross_vec2(vec0, vec1);
    if denom == 0.0 {
        return None;
    }
    let t = cross_vec2(pt1 - pt0, vec1) / denom;
    // println!("pt0: {}, vec0: {}, pt1: {}, vec1: {}", pt0, vec0, pt1, vec1);
    Some(pt0 + vec0 * t)
}

#[test]
fn test_find_intersection() {
    let pt = find_intersection(Point2::new(5.0f32, 8.0), Vector2::new(1.0, 0.0),
                               Point2::new(10.0, 2.0), Vector2::new(0.0, 1.0)).unwrap();
    assert_approx_eq!(pt, Point2::new(10.0, 8.0));

    let pt = find_intersection(Point2::new(50.0f32, 50.0), Vector2::new(1.0, -1.0),
                               Point2::new(10.0, 10.0), Vector2::new(1.0, 0.0)).unwrap();
    assert_approx_eq!(pt, Point2::new(90.0, 10.0));

    let pt = find_intersection(Point2::new(50.0f32, 50.0), Vector2::new(1.0, 1.0).normalize(),
                               Point2::new(60.0, 50.0), Vector2::new(-1.0, 1.0).normalize()).unwrap();
    assert_approx_eq!(pt, Point2::new(55.0, 55.0));
}

// Returns true if the point `pt` is to the right of the line passing through `line_pt0`
// and `line_pt1`, as if the observer is at `line_pt0`.
fn is_point_right_of_line(line_pt0: Point2<f32>, line_pt1: Point2<f32>, pt: Point2<f32>) -> bool {
    // http://stackoverflow.com/a/3461533/69671
    (line_pt1 - line_pt0).cross(&(pt - line_pt0)).x >= 0.0
}

#[test]
fn test_is_point_right_of_line() {
    assert!(is_point_right_of_line(Point2::new(5.0f32, 20.0), Point2::new(5.0, 10.0),
                                   Point2::new(8.0, 15.0)));
    assert!(is_point_right_of_line(Point2::new(5.0f32, 20.0), Point2::new(5.0, 10.0),
                                   Point2::new(50.0, 15.0)));
    assert!(!is_point_right_of_line(Point2::new(20.0f32, 20.0), Point2::new(30.0, 10.0),
                                    Point2::new(25.0, 10.0)));
}

pub struct PathIter<'a> {
    types: &'a [PathSegmentType],
    data: &'a [f32],

	types_index: usize,
	data_index: usize,
}

impl<'a> Iterator for PathIter<'a> {
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
