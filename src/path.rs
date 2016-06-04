
use coordinates::*;
use super::Point2;

pub enum PathSegment<N> {
	Move(Point2<N>),
	Line(Point2<N>),
	QuadCurve(Point2<N>, Point2<N>), // cairo doesn't have quad, but D2D, Skia, and NVpr do
	CubicCurve(Point2<N>, Point2<N>, Point2<N>),
	Close,
}

enum PathSegmentType { // smaller than a PathSegment
    Move,
    Line,
    QuadCurve,
    CubicCurve,
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
	//backend: HashMap<BackendType, BackendBakedStroke>,
}

struct BakedFill {
	solid_geo: Geometry<f32>,
	//quad_bezier_geo: Geometry<FillQuadBezierVertex>,
	//cubic_bezier_geo: Geometry<FillCubicBezierVertex>,
}

pub struct PathBuf {
    seg_types: Vec<PathSegmentType>,
    pts: Vec<Point2<f32>>,
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
			pts: vec![],
			initial_end_cap: EndCap::Flat,
			terminal_end_cap: EndCap::Flat,
			join_style: JoinStyle::MiterRevert(4.0),
			baked_stroke: None,
			baked_fill: None,
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
	    match self.seg_types[self.seg_types.len() - 1] {
	        PathSegmentType::Close => None,
	        _ => Some(self.pts[self.pts.len() - 1])
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

