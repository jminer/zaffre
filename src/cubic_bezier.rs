
use std::f32::consts::{FRAC_PI_2, PI};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};
use super::{Point2, Rect, LargerFloat, QuadBezier};
use super::nalgebra::{ApproxEq, BaseFloat, Cast, cast, Matrix2, Origin};
use super::smallvec::SmallVec;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CurveType {
    Plain,
    SingleInflection,
    DoubleInflection,
    FormsLoop,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LoopPoints {
    // The first t value on the curve that intersects the second.
    first: f32,
    // The t value of the midpoint of the loop.
    midpoint: f32,
    // The second t value on the curve that intersects the first.
    second: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CurveTypeData {
    Plain,
    SingleInflection(f32),
    DoubleInflection(f32, f32),
    FormsLoop(LoopPoints),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CubicBezier<N> {
    pub p0: Point2<N>,
    pub p1: Point2<N>,
    pub p2: Point2<N>,
    pub p3: Point2<N>,
}

impl<N> CubicBezier<N> {
    pub fn new(p0: Point2<N>, p1: Point2<N>, p2: Point2<N>, p3: Point2<N>) -> Self {
        CubicBezier {
            p0: p0,
            p1: p1,
            p2: p2,
            p3: p3,
        }
    }
}


impl<Nin: Copy, Nout: Copy + Cast<Nin>> Cast<CubicBezier<Nin>> for CubicBezier<Nout> {
    fn from(bezier: CubicBezier<Nin>) -> CubicBezier<Nout> {
        CubicBezier {
            p0: cast(bezier.p0),
            p1: cast(bezier.p1),
            p2: cast(bezier.p2),
            p3: cast(bezier.p3),
        }
    }
}

// impl<'a, M: Copy, N: Copy + ApproxEq<Margin=M>> ApproxEq for CubicBezier<N> {
//     type Margin = M;

//     fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
//         let margin = margin.into();
//         self.p0.x.approx_eq(other.p0.x, margin)
//         self.p0.y.approx_eq(other.p0.y, margin)
//             && self.p1.approx_eq(other.p1, margin)
//             && self.p2.approx_eq(other.p2, margin)
//             && self.p3.approx_eq(other.p3, margin)
//     }
// }

impl<N> ApproxEq<N> for CubicBezier<N> where N: ApproxEq<N> {
    fn approx_epsilon(_: Option<Self>) -> N {
        N::approx_epsilon(None)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: &N) -> bool {
        self.p0.approx_eq_eps(&other.p0, epsilon) &&
        self.p1.approx_eq_eps(&other.p1, epsilon) &&
        self.p2.approx_eq_eps(&other.p2, epsilon) &&
        self.p3.approx_eq_eps(&other.p3, epsilon)
    }
    fn approx_ulps(_: Option<Self>) -> u32 {
        N::approx_ulps(None)
    }
    fn approx_eq_ulps(&self, other: &Self, ulps: u32) -> bool {
        self.p0.approx_eq_ulps(&other.p0, ulps) &&
        self.p1.approx_eq_ulps(&other.p1, ulps) &&
        self.p2.approx_eq_ulps(&other.p2, ulps) &&
        self.p3.approx_eq_ulps(&other.p3, ulps)
    }
}

impl<N, F> CubicBezier<N> where F: BaseFloat
                                 + Cast<N>
                                 + Cast<f32>
                                 + Debug,
                                N: Copy
                                 + Cast<F>
                                 + LargerFloat<Float = F>
                                 + Sub<Output = N>
                                 + Add<Output = N>
                                 + Mul<Output = N>
                                 + Div<Output = N>,
                                f32: Cast<F> {
    #[cfg(test)]
    fn split_using_matrix(&self, t: f32) -> (CubicBezier<N>, CubicBezier<N>) {
        // https://pomax.github.io/bezierinfo/#matrixsplit
        let t: N::Float = cast(t);
        let (p0, p1, p2, p3) = (cast::<Point2<N>, Point2<N::Float>>(self.p0),
                                cast::<Point2<N>, Point2<N::Float>>(self.p1),
                                cast::<Point2<N>, Point2<N::Float>>(self.p2),
                                cast::<Point2<N>, Point2<N::Float>>(self.p3));
        let t_1 = t - cast(1.0);
        let t2 = t * t;
        let t3 = t * t * t;

        // I had to reorder a couple of the terms and add to_point() on the others because
        // you can't add Points.
        let bez0 = CubicBezier {
            p0: p0,
            p1: (p1 * t - p0 * t_1).to_point(),
            p2: p0 * t_1 * t_1
              + (p2 * t2
              - p1 * cast::<_, N::Float>(2.0) * t * t_1),
            p3: (p3 * t3
              - p2 * cast::<_, N::Float>(3.0) * t2 * t_1
              + (p1 * cast::<_, N::Float>(3.0) * t * t_1 * t_1
              - p0 * t_1 * t_1 * t_1)).to_point(),
        };
        let bez1 = CubicBezier {
            p0: (p3 * t3
              - p2 * cast::<_, N::Float>(3.0) * t2 * t_1
              + (p1 * cast::<_, N::Float>(3.0) * t * t_1 * t_1
              - p0 * t_1 * t_1 * t_1)).to_point(),
            p1: p3 * t2
              + (p1 * t_1 * t_1
              - p2 * cast::<_, N::Float>(2.0) * t * t_1),
            p2: (p3 * t - p2 * t_1).to_point(),
            p3: p3,
        };
        (cast(bez0), cast(bez1))
    }

    fn split_using_de_casteljau(&self, t: f32) -> (CubicBezier<N>, CubicBezier<N>) {
        // From benchmarking, it seems this function is a little faster than `split_using_matrix`
        // (21ns vs 25ns)
        // https://pomax.github.io/bezierinfo/#splitting
        let t: N::Float = cast(t);
        // ugh, the casts are so verbose
        let (p0, p1, p2, p3) = (cast::<Point2<N>, Point2<N::Float>>(self.p0),
                                cast::<Point2<N>, Point2<N::Float>>(self.p1),
                                cast::<Point2<N>, Point2<N::Float>>(self.p2),
                                cast::<Point2<N>, Point2<N::Float>>(self.p3));
        let lv1p0 = p0 + (p1 - p0) * t;
        let lv1p1 = p1 + (p2 - p1) * t;
        let lv1p2 = p2 + (p3 - p2) * t;

        let lv2p0 = lv1p0 + (lv1p1 - lv1p0) * t;
        let lv2p1 = lv1p1 + (lv1p2 - lv1p1) * t;

        let lv3p = lv2p0 + (lv2p1 - lv2p0) * t;

        let bez0 = CubicBezier {
            p0: p0,
            p1: lv1p0,
            p2: lv2p0,
            p3: lv3p,
        };
        let bez1 = CubicBezier {
            p0: lv3p,
            p1: lv2p1,
            p2: lv1p2,
            p3: p3,
        };
        (cast(bez0), cast(bez1))
    }

    pub fn split(&self, t: f32) -> (CubicBezier<N>, CubicBezier<N>) {
        self.split_using_de_casteljau(t)
    }

    // pub fn tangent_at(&self, t: f32) -> Vector2<N> {
    // }
    // pub fn project_point(&self, pt: Point2<N>) -> f32 {
    // }
    // pub fn point_at(&self, t: f32) -> Point2<N> {
    // }
    pub fn bounding_box(&self) -> Rect<N> {
        unimplemented!()
    }

    // Translates and rotates the curve so that the first point is at the origin (0, 0) and the
    // last point is on the x axis (x, 0).
    pub fn axis_aligned(&self) -> CubicBezier<N> {
        // https://pomax.github.io/bezierinfo/#aligning
        let (p0, mut p1, mut p2, mut p3) = (cast::<Point2<N>, Point2<N::Float>>(self.p0),
                                            cast::<Point2<N>, Point2<N::Float>>(self.p1),
                                            cast::<Point2<N>, Point2<N::Float>>(self.p2),
                                            cast::<Point2<N>, Point2<N::Float>>(self.p3));

        // Translate all the points to put p0 at the origin.
        let translation = -p0.to_vector();
        p1 += translation;
        p2 += translation;
        p3 += translation;

        // Rotate the points to put the last point on the x axis.
        let angle = -p3.y.atan2(p3.x);
        let (s, c) = angle.sin_cos();
        let rotation = Matrix2::new(c, -s,
                                    s,  c);

        p1 = rotation * p1;
        p2 = rotation * p2;
        p3 = rotation * p3;

        let bez: CubicBezier<N::Float> = CubicBezier {
            p0: Point2::origin(),
            p1: p1,
            p2: p2,
            p3: p3,
        };
        cast(bez)
    }

    pub fn inflection_points(&self) -> (Option<f32>, Option<f32>) {
        // https://pomax.github.io/bezierinfo/#inflections
        let _0_0: N::Float = cast(0.0);
        let _2_0: N::Float = cast(2.0);
        let _3_0: N::Float = cast(3.0);
        let _4_0: N::Float = cast(4.0);
        let _18_0: N::Float = cast(18.0);

        let aligned = self.axis_aligned();
        let (_p0, p1, p2, p3) = (cast::<Point2<N>, Point2<N::Float>>(aligned.p0),
                                            cast::<Point2<N>, Point2<N::Float>>(aligned.p1),
                                            cast::<Point2<N>, Point2<N::Float>>(aligned.p2),
                                            cast::<Point2<N>, Point2<N::Float>>(aligned.p3));

        let a = p2.x * p1.y;
        let b = p3.x * p1.y;
        let c = p1.x * p2.y;
        let d = p3.x * p2.y;

        let x = _18_0 * (-_3_0 * a + _2_0 * b + _3_0 * c - d);
        let y = _18_0 * (_3_0 * a - b - _3_0 * c);
        let z = _18_0 * (c - a);

        // can't divide by zero
        if x == _0_0 {
            return (None, None);
        }

        // Quadratic formula
        let discrim = y * y - _4_0 * x * z;

        if discrim < _0_0 {
            return (None, None);
        }

        let t0: f32 = cast((-y + discrim.sqrt()) / (_2_0 * x));
        let t1: f32 = cast((-y - discrim.sqrt()) / (_2_0 * x));

        match (t0 >= 0.0 && t0 <= 1.0, t1 >= 0.0 && t1 <= 1.0) {
            (true, true) if t1 < t0 => (Some(t1), Some(t0)), // sort
            (true, true) => (Some(t0), Some(t1)),
            (true, false) => (Some(t0), None),
            (false, true) => (Some(t1), None),
            (false, false) => (None, None),
        }
    }

    pub fn loop_points(&self) -> LoopPoints {

        unimplemented!()
    }

    // Returns a quadratic Bezier that approximates this curve. If this curve has an inflection or
    // loop, it must be split before a quadratic Bezier can approximate it.
    pub fn to_quad_bezier(&self) -> Option<QuadBezier<N>> {
        unimplemented!()
    }

    pub fn get_quad_bezier_approximation(&self,
                                         quad_beziers: &mut SmallVec<[QuadBezier<N>; 2]>,
                                         tolerance: f32) {

        unimplemented!()

    }

    pub fn curve_type(&self) -> CurveType {
        // https://pomax.github.io/bezierinfo/#canonical
        let _0_0: N::Float = cast(0.0);
        let _1_0: N::Float = cast(1.0);
        let _2_0: N::Float = cast(2.0);
        let _3_0: N::Float = cast(3.0);
        let _4_0: N::Float = cast(4.0);

        let (p0, mut p1, mut p2, mut p3) = (cast::<Point2<N>, Point2<N::Float>>(self.p0),
                                            cast::<Point2<N>, Point2<N::Float>>(self.p1),
                                            cast::<Point2<N>, Point2<N::Float>>(self.p2),
                                            cast::<Point2<N>, Point2<N::Float>>(self.p3));
        // Transform - to get p0 at (0, 0)
        let tr = p0.to_vector();
        //p0 = origin();
        p1 -= tr;
        p2 -= tr;
        p3 -= tr;

        let f2d1 = p2.y / p1.y;
        let f3d1 = p3.y / p1.y;
        let p3x = (p3.x - p1.x * f3d1) / (p2.x - p1.x * f2d1);
        let p3y = f3d1 + (_1_0 - f2d1) * p3x;
        //println!("x: {:?}, y: {:?}", p3x, p3y);

        if p3y >= _1_0 {
            return CurveType::SingleInflection;
        }
        if p3x > _1_0 {
            return CurveType::Plain;
        }

        let y = if p3x > _0_0 {
            ((_3_0 * (_4_0 * p3x - p3x * p3x)).sqrt() - p3x) / _2_0
        } else {
            (-p3x * p3x + _3_0 * p3x) / _3_0
        };

        if p3y < y {
            CurveType::Plain
        } else {
            let y = (-p3x * p3x + _2_0 * p3x + _3_0) / _4_0;
            if p3y <= y {
                CurveType::FormsLoop
            } else {
                CurveType::DoubleInflection
            }
        }
    }

    pub fn curve_type_data(&self) -> CurveTypeData {

        match self.inflection_points() {
            (Some(t0), Some(t1)) => return CurveTypeData::DoubleInflection(t0, t1),
            (Some(t), None) => return CurveTypeData::SingleInflection(t),
            _ => {},
        }

        // TODO: loop

        CurveTypeData::Plain
    }
}

struct Roots {
    arr: [f32; 3],
    len: u32,
}

impl Roots {
    fn get(&self) -> &[f32] {
        &self.arr[0..self.len as usize]
    }
}

// Takes coefficients of an equation in the form
//   ax^3 + bx^2 + cx + d = 0
// and returns p and q in the equation
//   t^3 + pt + q = 0
fn reduce_to_depressed_cubic(a: f32, b: f32, c: f32, d: f32) -> (f32, f32) {
    // https://en.wikipedia.org/wiki/Cubic_function#Reduction_to_a_depressed_cubic
    // I'm using Wikipedia's formulas because on this page x^3 has no coefficient:
    // http://www.trans4mind.com/personal_development/mathematics/polynomials/cubicAlgebra.htm
    // The only thing it should change compared to the second page is the term added/subtracted at
    // the very end. It needs to be -b/(3*a) instead of -a/3.
    let a_2 = a * a;
    let b_2 = b * b;
    let ac = a * c;
    let p = (3.0 * ac - b_2) / (3.0 * a_2);
    let q = (2.0 * b_2 * b - 9.0 * ac * b + 27.0 * a_2 * d) / (27.0 * a_2 * a);
    (p, q)
}


fn solve_cubic(a: f32, b: f32, c: f32, d: f32) -> Roots {
    let (p, q) = reduce_to_depressed_cubic(a, b, c, d);
    let b_d_3a = b / (3.0 * a);

    // http://www.trans4mind.com/personal_development/mathematics/polynomials/cubicAlgebra.htm
    let p_d_3 = p * (1.0 / 3.0);
    let p_3_d_27 = p_d_3 * p_d_3 * p_d_3;
    let q_d_2 = q * (1.0 / 2.0);

    let delta = q * q * (1.0 / 4.0) + p_3_d_27;
    if delta <= 0.0 {
        let r = (-p_3_d_27).sqrt();
        let phi_d_3 = (-q_d_2 / r).acos() * (1.0 / 3.0);
        let two_r_d_3 = 2.0 * (-p_d_3).sqrt();

        let r1 = two_r_d_3 * phi_d_3.cos() - b_d_3a;
        let r2 = two_r_d_3 * (phi_d_3 + 2.0 * PI / 3.0).cos() - b_d_3a;
        let r3 = two_r_d_3 * (phi_d_3 + 4.0 * PI / 3.0).cos() - b_d_3a;

        let mut roots = Roots {
            arr: [r1, r2, r3],
            len: 3,
        };
        // Deduplicate any very close roots.
        if r3.approx_eq_ulps(&r1, 32) || r3.approx_eq_ulps(&r2, 32) {
            roots.len -= 1;
        }
        if r2.approx_eq_ulps(&r1, 32)  {
            roots.arr = [r1, r3, 0.0];
            roots.len -= 1;
        }

        return roots;
    } else {
        let delta_sqrt = delta.sqrt();
        let u = (-q_d_2 + delta_sqrt).cbrt();
        let v = (q_d_2 + delta_sqrt).cbrt();

        return Roots {
            arr: [u - v - b_d_3a, 0.0, 0.0],
            len: 1,
        };
    }
}

#[cfg(test)]
mod benchmarks {
    use std::hint::black_box;
    use test::bench::Bencher;
    use ::Point2;
    use super::CubicBezier;

    #[bench]
    fn bench_split_using_matrix(b: &mut Bencher) {
        b.iter(|| {
            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.0)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));

            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.1)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));

            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.2)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));

            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.3)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));
        });
    }

    #[bench]
    fn bench_split_using_de_casteljau(b: &mut Bencher) {
        b.iter(|| {
            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.0)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));

            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.1)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));

            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.2)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));

            let bez0 = CubicBezier::new(black_box(Point2::new(5.0, 10.3)),
                                        black_box(Point2::new(10.0, 30.0)),
                                        black_box(Point2::new(50.0, 30.0)),
                                        black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));
        });
    }
}

#[test]
fn test_split() {
    let bez0 = CubicBezier::new(Point2::new(5.0, 10.0), Point2::new(10.0, 30.0),
                                Point2::new(50.0, 30.0), Point2::new(60.0, 10.0));

    let pair0 = bez0.split_using_de_casteljau(0.5);
    let pair1 = bez0.split_using_matrix(0.5);
    assert_approx_eq!(pair0.0, pair1.0);
    assert_approx_eq!(pair0.1, pair1.1);
}

#[test]
fn test_axis_aligned() {
    let bez0 = CubicBezier::new(Point2::new(80.0, 100.0), Point2::new(20.0, 150.0),
                                Point2::new(100.0, 170.0), Point2::new(220.0, 100.0));

    // calculated using https://pomax.github.io/bezierinfo/#aligning
    let aligned = CubicBezier::new(Point2::new(0.0, 0.0), Point2::new(-60.0, 50.0),
                                Point2::new(20.0, 70.0), Point2::new(140.0, 0.0));
    assert_approx_eq!(bez0.axis_aligned(), aligned);
}

#[test]
fn test_inflection_points() {
    let bez = CubicBezier::new(Point2::new(20.0, 70.0),
                               Point2::new(50.0, 30.0),
                               Point2::new(90.0, 90.0),
                               Point2::new(150.0, 40.0));
    let pts = bez.inflection_points();
    assert_eq!(pts.1, None);
    assert_approx_eq!(0.4634282, pts.0.unwrap());

    let bez = CubicBezier::new(Point2::new(40.0, 30.0),
                               Point2::new(120.0, 80.0),
                               Point2::new(65.0, 80.0),
                               Point2::new(150.0, 35.0));
    let pts = bez.inflection_points();
    assert_approx_eq!(0.28623563, pts.0.unwrap());
    assert_approx_eq!(0.7347969, pts.1.unwrap());

    let bez = CubicBezier::new(Point2::new(40.0, 30.0),
                               Point2::new(115.0, 100.0),
                               Point2::new(130.0, 75.0),
                               Point2::new(150.0, 35.0));
    let pts = bez.inflection_points();
    assert_eq!((None, None), pts);
}

#[test]
fn test_curve_type() {
    // Interactive bezier curve at https://pomax.github.io/bezierinfo/#canonical
    let bez = CubicBezier::new(Point2::new(30.0, 350.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    let bez = CubicBezier::new(Point2::new(290.0, 370.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    let bez = CubicBezier::new(Point2::new(140.0, 350.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    let bez = CubicBezier::new(Point2::new(140.0, 350.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    // fourth point with 0 < x < 1
    let bez = CubicBezier::new(Point2::new(135.0, 35.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::Plain);

    // fourth point with 0 < x < 1
    let bez = CubicBezier::new(Point2::new(135.0, 35.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(155.0, 150.0));
    assert_eq!(bez.curve_type(), CurveType::FormsLoop);

    // fourth point with 0 < x < 1
    let bez = CubicBezier::new(Point2::new(135.0, 35.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(151.0, 188.0));
    assert_eq!(bez.curve_type(), CurveType::DoubleInflection);

    // fourth point with x < 0
    let bez = CubicBezier::new(Point2::new(200.0, 130.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::Plain);

    // fourth point with x < 0
    let bez = CubicBezier::new(Point2::new(260.0, 80.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::FormsLoop);

    // fourth point with x < 0
    let bez = CubicBezier::new(Point2::new(380.0, 50.0),
                               Point2::new(135.0, 210.0),
                               Point2::new(275.0, 176.0),
                               Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::DoubleInflection);

}

#[test]
fn test_solve_cubic() {
    // I used Wolfram Alpha to solve and graph.

    // x^3 + x^2 + x + 1 = 0
    let roots = solve_cubic(1.0, 1.0, 1.0, 1.0);
    assert_eq!(roots.get().len(), 1);
    assert_approx_eq!(roots.get()[0], -1.0);

    // x^3 - 9x^2 + x + 1 = 0
    let roots = solve_cubic(1.0, -9.0, 1.0, 1.0);
    assert_eq!(roots.get().len(), 3);
    assert_approx_eq!(roots.get()[0], 8.874622);
    assert_approx_eq!(roots.get()[1], -0.278795);
    assert_approx_eq!(roots.get()[2], 0.40417218);

    // test deduping one of the roots
    // x^3 - 3x^2 + x + 2.0886621075 = 0
    let roots = solve_cubic(1.0, -3.0, 1.0, 2.0886621075);
    assert_eq!(roots.get().len(), 2);
    assert_approx_eq!(roots.get()[0], 1.8164966);
    assert_approx_eq!(roots.get()[1], -0.6329931618);
}
