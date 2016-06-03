
use std::ops::{Add, Sub};
use super::Rect;
use super::nalgebra::{ApproxEq, BaseFloat, Cast, cast, Point2, Vector2};

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
                            + Cast<f32>,
                           N: Copy
                            + Cast<F>
                            + LargerFloat<Float = F>
                            + Sub<Output = N>
                            + Add<Output = N> {
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

    // pub fn split(&self, t: f32) -> (Bezier<N>, Bezier<N>) {
    // }
    // pub fn tangent_at(&self, t: f32) -> Vector2<N> {
    // }
    // pub fn project_point(&self, pt: Point2<N>) -> f32 {
    // }
    // pub fn point_at(&self, t: f32) -> Point2<N> {
    // }
    // pub fn bounding_box(&self) -> Rect<N> {
    // }
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