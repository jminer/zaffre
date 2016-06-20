
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use coordinates::*;
use super::{Point2, QuadBezier};
use nalgebra::{ApproxEq, Cross, origin, Norm, Vector2};

mod glium_backend;

pub use self::glium_backend::{stencil_stroke_path};

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

#[derive(Debug, Copy, Clone)]
struct SolidVertex {
	position: (f32, f32),
}

#[derive(Debug, Copy, Clone)]
struct StrokeQuadBezierVertex {
	position: (f32, f32),
    pt0: (f32, f32),
    pt1: (f32, f32),
    pt2: (f32, f32),
}

impl StrokeQuadBezierVertex {
    fn new(position: Point2<f32>, bezier: QuadBezier<f32>) -> Self {
        StrokeQuadBezierVertex {
            position: (position.x, position.y),
            pt0: (bezier.p0.x, bezier.p0.y),
            pt1: (bezier.p1.x, bezier.p1.y),
            pt2: (bezier.p2.x, bezier.p2.y),
        }
    }
}

#[derive(Debug)]
struct Geometry<T> {
	vertices: Vec<T>,
	indices: Vec<u16>,
}

#[derive(Debug)]
struct BakedStroke {
	solid_geo: Geometry<SolidVertex>,
	quad_bezier_geo: Geometry<StrokeQuadBezierVertex>,
	backend: HashMap<usize, Box<Any>>,
}

#[derive(Debug)]
struct BakedFill {
	solid_geo: Geometry<SolidVertex>,
	//quad_bezier_geo: Geometry<FillQuadBezierVertex>,
	//cubic_bezier_geo: Geometry<FillCubicBezierVertex>,
	backend: HashMap<usize, Box<Any>>,
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

    stroke_width: f32,
	initial_end_cap: EndCap,
	terminal_end_cap: EndCap,
	// initial_dash_cap: EndCap,
	// terminal_dash_cap: EndCap,
	join_style: JoinStyle,

    // TODO: I have to clear the baked data, including the backend data, when the path is changed.
	baked_stroke: Option<BakedStroke>,
	baked_fill: Option<BakedFill>,
}

impl PathBuf {
	pub fn new() -> PathBuf {
		PathBuf {
			seg_types: vec![],
			seg_data: vec![],

            stroke_width: 1.0,
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
	// pub fn iter() -> Iterator {
	// }
	// pub fn flatten(float tolerance) -> Iterator {
	// }

	// pub fn from_svg_path_string(path_string: &str);
	// pub fn from_postscript_path_string(path_string: &str);

	pub fn stroke_width(&self) -> f32 {
        self.stroke_width
    }
	pub fn set_stroke_width(&mut self, width: f32) {
        self.stroke_width = width;
        // TODO: erase baked paths
    }

	// pub fn mask(&self) -> u32;
	// pub fn set_mask(&mut self, mask: u32);

	// pub fn is_point_in_fill(&self, pt: Point, mask: Option<u32>) -> bool;
	// pub fn is_point_in_stroke(&self, pt: Point) -> bool;

	// pub fn length(segments: Range<usize>) -> f32;
	// pub fn point_at_distance(segments: Range<usize>, distance: f32) -> (Point, f32, f32);

	fn bake_stroke(&mut self) {
        let mut solid_vertices = vec![];
        let mut solid_indices = vec![];
        let mut quad_bezier_vertices = vec![];
        let mut quad_bezier_indices = vec![];
        let mut curr_pt = None;
        let mut curr_dir = None;
        for seg in self.segments() {
            match seg {
                PathSegment::Move(pt) => {
                    curr_pt = Some(pt);
                },
                PathSegment::Line(end_pt) => {
                    let start_pt: Point2<f32> = curr_pt.expect("line segment requires current point");
                    let line_dir: Vector2<f32> = end_pt - start_pt;
                    let left_dir = Vector2::new(line_dir.y, -line_dir.x); // rotate 90 CCW
                    let scale_to_width = left_dir.norm().recip() * self.stroke_width / 2.0;
                    let left_dir = left_dir * scale_to_width;
                    let right_dir = -left_dir;

                    let start_index = solid_vertices.len();
                    let vert = start_pt + left_dir; // start left
                    solid_vertices.push(SolidVertex {
                        position: (vert.x, vert.y)
                    });
                    let vert = start_pt + right_dir; // start right
                    solid_vertices.push(SolidVertex {
                        position: (vert.x, vert.y)
                    });
                    let vert = end_pt + left_dir; // end left
                    solid_vertices.push(SolidVertex {
                        position: (vert.x, vert.y)
                    });
                    let vert = end_pt + right_dir; // end right
                    solid_vertices.push(SolidVertex {
                        position: (vert.x, vert.y)
                    });

                    solid_indices.push((start_index + 0) as u16);
                    solid_indices.push((start_index + 1) as u16);
                    solid_indices.push((start_index + 2) as u16);

                    solid_indices.push((start_index + 1) as u16);
                    solid_indices.push((start_index + 3) as u16);
                    solid_indices.push((start_index + 2) as u16);

                    curr_pt = Some(end_pt);
                    curr_dir = Some(line_dir);
                },
                PathSegment::QuadCurve(control_pt, end_pt) => {
                    let start_pt: Point2<f32> = curr_pt.expect("quad curve segment requires current point");
                    let bezier = QuadBezier::new(start_pt, control_pt, end_pt);
                    let half_stroke_width = self.stroke_width / 2.0;

                    let start_dir: Vector2<f32> = (control_pt - start_pt).normalize(); // bezier.tangent_at(0.0);
                    let end_dir: Vector2<f32> = (end_pt - control_pt).normalize(); // bezier.tangent_at(1.0);

                    let start_left_dir = Vector2::new(start_dir.y, -start_dir.x); // rotate 90 CCW
                    let start_left_dir = start_left_dir * half_stroke_width;
                    let start_right_dir = -start_left_dir;

                    let end_left_dir = Vector2::new(end_dir.y, -end_dir.x); // rotate 90 CCW
                    let end_left_dir = end_left_dir * half_stroke_width;
                    let end_right_dir = -end_left_dir;

                    // A line tangent to a quadratic Bézier curve intersects the curve in exactly
                    // that one point. This fact means that multiple tangent lines will bound the
                    // curve.
                    // (I don't know how to prove the tangent only intersect in the one point, but
                    // I know that for the tangent to intersect in another point, the curve would
                    // have to have an inflection point. Quadratic curves never have an inflection
                    // point.)
                    let mid_pt = bezier.point_at(0.5);
                    let mid_dir = bezier.tangent_at(0.5);
                    let mid_left_dir = Vector2::new(mid_dir.y, -mid_dir.x); // rotate 90 CCW
                    let mid_left_dir = mid_left_dir * half_stroke_width;
                    let mid_right_dir = -mid_left_dir;
                    // TODO: This should maybe be configuable to be subdivided an arbitary
                    // number of times. It would reduce the area, but I'm not sure it's
                    // necessary.

                    let curves_right = is_point_right_of_line(start_pt, control_pt, end_pt);

                    // si = start inner, mo = mid outer, ei = end inner
                    let (so_dir, si_dir, mo_dir, mi_dir, eo_dir, ei_dir) = if curves_right {
                        (start_left_dir, start_right_dir,
                         mid_left_dir, mid_right_dir,
                         end_left_dir, end_right_dir)
                    } else {
                        (start_right_dir, start_left_dir,
                         mid_right_dir, mid_left_dir,
                         end_right_dir, end_left_dir)
                    };
                    let start_outer_pt = start_pt + so_dir;
                    let mid_outer_pt = mid_pt + mo_dir;
                    let end_outer_pt = end_pt + eo_dir;

                    let start_index = quad_bezier_vertices.len();
                    let vert = start_outer_pt; // start outer
                    quad_bezier_vertices.push(StrokeQuadBezierVertex::new(vert, bezier));

                    let vert = start_pt + si_dir; // start inner
                    quad_bezier_vertices.push(StrokeQuadBezierVertex::new(vert, bezier));

                    // The unwrap shouldn't ever fail, but it isn't worth a panic.
                    let vert = find_intersection(start_outer_pt, start_dir,
                                                 mid_outer_pt, mid_dir).unwrap_or(mid_pt); // outer before mid
                    quad_bezier_vertices.push(StrokeQuadBezierVertex::new(vert, bezier));

                    let vert = find_intersection(mid_outer_pt, mid_dir,
                                                 end_outer_pt, end_dir).unwrap_or(mid_pt); // outer after mid
                    quad_bezier_vertices.push(StrokeQuadBezierVertex::new(vert, bezier));

                    let vert = mid_pt + mi_dir; // inner mid
                    quad_bezier_vertices.push(StrokeQuadBezierVertex::new(vert, bezier));

                    let vert = end_outer_pt; // end outer
                    quad_bezier_vertices.push(StrokeQuadBezierVertex::new(vert, bezier));

                    let vert = end_pt + ei_dir; // end inner
                    quad_bezier_vertices.push(StrokeQuadBezierVertex::new(vert, bezier));

                    quad_bezier_indices.push((start_index + 0) as u16);
                    quad_bezier_indices.push((start_index + 1) as u16);
                    quad_bezier_indices.push((start_index + 2) as u16);

                    quad_bezier_indices.push((start_index + 1) as u16);
                    quad_bezier_indices.push((start_index + 4) as u16);
                    quad_bezier_indices.push((start_index + 2) as u16);

                    quad_bezier_indices.push((start_index + 2) as u16);
                    quad_bezier_indices.push((start_index + 4) as u16);
                    quad_bezier_indices.push((start_index + 3) as u16);

                    quad_bezier_indices.push((start_index + 4) as u16);
                    quad_bezier_indices.push((start_index + 6) as u16);
                    quad_bezier_indices.push((start_index + 3) as u16);

                    quad_bezier_indices.push((start_index + 3) as u16);
                    quad_bezier_indices.push((start_index + 6) as u16);
                    quad_bezier_indices.push((start_index + 5) as u16);

                    curr_pt = Some(end_pt);
                    curr_dir = Some(end_dir);
                }
                _ => unimplemented!(),
            }
        }
        self.baked_stroke = Some(BakedStroke {
            solid_geo: Geometry {
                vertices: solid_vertices,
                indices: solid_indices,
            },
            quad_bezier_geo: Geometry {
                vertices: quad_bezier_vertices,
                indices: quad_bezier_indices,
            },
            backend: HashMap::new(),
        });

	}

	fn bake_fill(&mut self) {

	}
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
