
use std::fmt::Debug;
use std::ops::{Add, Div, Sub};
use super::Rect;
use super::nalgebra::{ApproxEq, BaseFloat, Cast, cast, Point2};

trait LargerFloat: Sized {
    type Float: BaseFloat + Cast<Self> + Cast<f32>;
}

impl LargerFloat for f32 {
    type Float = f32;
}
impl LargerFloat for f64 {
    type Float = f64;
}
impl LargerFloat for i16 {
    type Float = f32;
}
impl LargerFloat for u16 {
    type Float = f32;
}
impl LargerFloat for i32 {
    type Float = f64;
}
impl LargerFloat for u32 {
    type Float = f64;
}
impl LargerFloat for i64 {
    type Float = f64;
}
impl LargerFloat for u64 {
    type Float = f64;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CurveType {
	Plain,
	SingleInflection,
	DoubleInflection,
	FormsLoop,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Bezier<N> {
    pub p0: Point2<N>,
    pub p1: Point2<N>,
    pub p2: Point2<N>,
    pub p3: Point2<N>,
}

impl<N> Bezier<N> {
    pub fn new(p0: Point2<N>, p1: Point2<N>, p2: Point2<N>, p3: Point2<N>) -> Self {
        Bezier {
            p0: p0,
            p1: p1,
            p2: p2,
            p3: p3,
        }
    }
}


impl<Nin: Copy, Nout: Copy + Cast<Nin>> Cast<Bezier<Nin>> for Bezier<Nout> {
    fn from(bezier: Bezier<Nin>) -> Bezier<Nout> {
        Bezier {
            p0: cast(bezier.p0),
            p1: cast(bezier.p1),
            p2: cast(bezier.p2),
            p3: cast(bezier.p3),
        }
    }
}

impl<N> ApproxEq<N> for Bezier<N> where N: ApproxEq<N> {
    fn approx_epsilon(_: Option<Self>) -> N {
        N::approx_epsilon(None)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: &N) -> bool {
        self.p0.approx_eq_eps(&other.p0, epsilon) &&
        self.p1.approx_eq_eps(&other.p1, epsilon) &&
        self.p2.approx_eq_eps(&other.p2, epsilon) &&
        self.p3.approx_eq_eps(&other.p3, epsilon)
    }
    fn approx_ulps(unused_mut: Option<Self>) -> u32 {
        N::approx_ulps(None)
    }
    fn approx_eq_ulps(&self, other: &Self, ulps: u32) -> bool {
        self.p0.approx_eq_ulps(&other.p0, ulps) &&
        self.p1.approx_eq_ulps(&other.p1, ulps) &&
        self.p2.approx_eq_ulps(&other.p2, ulps) &&
        self.p3.approx_eq_ulps(&other.p3, ulps)
    }
}

impl<N, F> Bezier<N> where F: BaseFloat
                            + Cast<N>
                            + Cast<f32>
                            + Debug,
                           N: Copy
                            + Cast<F>
                            + LargerFloat<Float = F>
                            + Sub<Output = N>
                            + Add<Output = N>
                            + Div<Output = N> {
    fn split_using_matrix(&self, t: f32) -> (Bezier<N>, Bezier<N>) {
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
        let bez0 = Bezier {
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
        let bez1 = Bezier {
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

    fn split_using_de_casteljau(&self, t: f32) -> (Bezier<N>, Bezier<N>) {
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

        let bez0 = Bezier {
            p0: p0,
            p1: lv1p0,
            p2: lv2p0,
            p3: lv3p,
        };
        let bez1 = Bezier {
            p0: lv3p,
            p1: lv2p1,
            p2: lv1p2,
            p3: p3,
        };
        (cast(bez0), cast(bez1))
    }

    pub fn split(&self, t: f32) -> (Bezier<N>, Bezier<N>) {
        self.split_using_de_casteljau(t)
    }

    // pub fn tangent_at(&self, t: f32) -> Vector2<N> {
    // }
    // pub fn project_point(&self, pt: Point2<N>) -> f32 {
    // }
    // pub fn point_at(&self, t: f32) -> Point2<N> {
    // }
    // pub fn bounding_box(&self) -> Rect<N> {
    // }

    pub fn curve_type(&self) -> CurveType {
        // https://pomax.github.io/bezierinfo/#canonical
        let _0_0: N::Float = cast(0.0);
        let _1_0: N::Float = cast(1.0);
        let _2_0: N::Float = cast(2.0);
        let _3_0: N::Float = cast(3.0);
        let _4_0: N::Float = cast(4.0);

        let (mut p0, mut p1, mut p2, mut p3) = (cast::<Point2<N>, Point2<N::Float>>(self.p0),
                                                cast::<Point2<N>, Point2<N::Float>>(self.p1),
                                                cast::<Point2<N>, Point2<N::Float>>(self.p2),
                                                cast::<Point2<N>, Point2<N::Float>>(self.p3));
        // Transform - to get p0 at (0, 0)
        let tr = p0.to_vector();
        //p0 = origin();
        p1 -= tr;
        p2 -= tr;
        p3 -= tr;

/*
        // Shear - to get p1 at (0, some value)
        // This might be clearer using matrices, but probably not as fast.
        let s = -p1.x / p1.y;
        // p0.x is already 0
        p1.x = _0_0; // += s * p1.y;
        p2.x += s * p2.y;
        p3.x += s * p3.y;

        // Scale - to get p1 at (0, 1) and p2 at (1, some value)
        let sx = _1_0 / p2.x;
        let sy = _1_0 / p1.y;
        // p0 and p1.x are 0
        p1.y = _1_0; // *= sy;
        p2.x = _1_0; // *= sx;
        p2.y *= sy;
        p3.x *= sx;
        p3.y *= sy;

        // Shear - to get p2 at (1, 1)
        let s = (_1_0 - p2.y) / p2.x;
        // all are already in place except for the following
        p2.y = _1_0; // += s * p2.x;
        p3.y += s * p3.x;
*/
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
}

#[test]
fn test_split() {
    let bez0 = Bezier::new(Point2::new(5.0, 10.0), Point2::new(10.0, 30.0),
                           Point2::new(50.0, 30.0), Point2::new(60.0, 10.0));

    let pair0 = bez0.split_using_de_casteljau(0.5);
    let pair1 = bez0.split_using_matrix(0.5);
    assert_approx_eq!(pair0.0, pair1.0);
    assert_approx_eq!(pair0.1, pair1.1);
}

mod benchmarks {
    use ::test::{black_box, Bencher};
    use ::Point2;
    use super::Bezier;

    #[bench]
    fn bench_split_using_matrix(b: &mut Bencher) {
        b.iter(|| {
            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.0)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));

            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.1)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));

            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.2)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));

            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.3)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_matrix(black_box(0.5)));
        });
    }

    #[bench]
    fn bench_split_using_de_casteljau(b: &mut Bencher) {
        b.iter(|| {
            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.0)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));

            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.1)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));

            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.2)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));

            let bez0 = Bezier::new(black_box(Point2::new(5.0, 10.3)),
                                black_box(Point2::new(10.0, 30.0)),
                                black_box(Point2::new(50.0, 30.0)),
                                black_box(Point2::new(60.0, 10.0)));

            black_box(bez0.split_using_de_casteljau(black_box(0.5)));
        });
    }
}

#[test]
fn test_curve_type() {
    // Interactive bezier curve at https://pomax.github.io/bezierinfo/#canonical
    let bez = Bezier::new(Point2::new(30.0, 350.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    let bez = Bezier::new(Point2::new(290.0, 370.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    let bez = Bezier::new(Point2::new(140.0, 350.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    let bez = Bezier::new(Point2::new(140.0, 350.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::SingleInflection);

    // fourth point with 0 < x < 1
    let bez = Bezier::new(Point2::new(135.0, 35.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::Plain);

    // fourth point with 0 < x < 1
    let bez = Bezier::new(Point2::new(135.0, 35.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(155.0, 150.0));
    assert_eq!(bez.curve_type(), CurveType::FormsLoop);

    // fourth point with 0 < x < 1
    let bez = Bezier::new(Point2::new(135.0, 35.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(151.0, 188.0));
    assert_eq!(bez.curve_type(), CurveType::DoubleInflection);

    // fourth point with x < 0
    let bez = Bezier::new(Point2::new(200.0, 130.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::Plain);

    // fourth point with x < 0
    let bez = Bezier::new(Point2::new(260.0, 80.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::FormsLoop);

    // fourth point with x < 0
    let bez = Bezier::new(Point2::new(380.0, 50.0),
                          Point2::new(135.0, 210.0),
                          Point2::new(275.0, 176.0),
                          Point2::new(220.0, 40.0));
    assert_eq!(bez.curve_type(), CurveType::DoubleInflection);

}