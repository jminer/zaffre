
use std::fmt::Debug;
use std::ops::{Add, Div, Sub};
use super::{LargerFloat, Point2, Rect, Vector2};
use super::nalgebra::{ApproxEq, BaseFloat, Cast, cast, Norm};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct QuadBezier<N> {
    pub p0: Point2<N>,
    pub p1: Point2<N>,
    pub p2: Point2<N>,
}

impl<N> QuadBezier<N> {
    pub fn new(p0: Point2<N>, p1: Point2<N>, p2: Point2<N>) -> Self {
        QuadBezier {
            p0: p0,
            p1: p1,
            p2: p2,
        }
    }
}


impl<Nin: Copy, Nout: Copy + Cast<Nin>> Cast<QuadBezier<Nin>> for QuadBezier<Nout> {
    fn from(bezier: QuadBezier<Nin>) -> QuadBezier<Nout> {
        QuadBezier {
            p0: cast(bezier.p0),
            p1: cast(bezier.p1),
            p2: cast(bezier.p2),
        }
    }
}

impl<N> ApproxEq<N> for QuadBezier<N> where N: ApproxEq<N> {
    fn approx_epsilon(_: Option<Self>) -> N {
        N::approx_epsilon(None)
    }

    fn approx_eq_eps(&self, other: &Self, epsilon: &N) -> bool {
        self.p0.approx_eq_eps(&other.p0, epsilon) &&
        self.p1.approx_eq_eps(&other.p1, epsilon) &&
        self.p2.approx_eq_eps(&other.p2, epsilon)
    }
    fn approx_ulps(_: Option<Self>) -> u32 {
        N::approx_ulps(None)
    }
    fn approx_eq_ulps(&self, other: &Self, ulps: u32) -> bool {
        self.p0.approx_eq_ulps(&other.p0, ulps) &&
        self.p1.approx_eq_ulps(&other.p1, ulps) &&
        self.p2.approx_eq_ulps(&other.p2, ulps)
    }
}

impl<N, F> QuadBezier<N> where F: BaseFloat
                                + Cast<N>
                                + Cast<f32>
                                + Debug,
                               N: Copy
                                + Cast<F>
                                + LargerFloat<Float = F>
                                + Sub<Output = N>
                                + Add<Output = N>
                                + Div<Output = N> {

    pub fn tangent_at(&self, t: f32) -> Vector2<N> {
        let _1_0: N::Float = cast(1.0);
        let t: N::Float = cast(t);
        let (p0, p1, p2) = (cast::<Point2<N>, Point2<N::Float>>(self.p0),
                            cast::<Point2<N>, Point2<N::Float>>(self.p1),
                            cast::<Point2<N>, Point2<N::Float>>(self.p2));

        cast(((p1 - p0) * (_1_0 - t) + (p2 - p1) * t).normalize())
    }

    pub fn point_at(&self, t: f32) -> Point2<N> {
        let _1_0: N::Float = cast(1.0);
        let _2_0: N::Float = cast(2.0);
        let t: N::Float = cast(t);
        let (p0, p1, p2) = (cast::<Point2<N>, Point2<N::Float>>(self.p0),
                            cast::<Point2<N>, Point2<N::Float>>(self.p1),
                            cast::<Point2<N>, Point2<N::Float>>(self.p2));

        let one_m_t = _1_0 - t;

        let p1 = p1.to_vector();
        let p2 = p2.to_vector();
        cast(p0 * one_m_t * one_m_t + p1 * _2_0 * one_m_t * t + p2 * t * t)
    }
}

#[test]
fn test_tangent_at() {
    let bez = QuadBezier::new(Point2::new(220.0, 40.0),
                              Point2::new(50.0, 180.0),
                              Point2::new(135.0, 210.0));

    assert_approx_eq!(bez.tangent_at(0.0).norm(), 1.0);
    assert_approx_eq!(bez.tangent_at(0.3).norm(), 1.0);
    assert_approx_eq!(bez.tangent_at(1.0).norm(), 1.0);

    assert_approx_eq!(bez.tangent_at(0.0), Vector2::new(-170.0, 140.0).normalize());
    assert_approx_eq!(bez.tangent_at(1.0), Vector2::new(85.0, 30.0).normalize());
    assert_approx_eq!(bez.tangent_at(0.3), Vector2::new(-93.5, 107.0).normalize());
}

#[test]
fn test_point_at() {
    let bez = QuadBezier::new(Point2::new(220.0, 40.0),
                              Point2::new(50.0, 180.0),
                              Point2::new(135.0, 210.0));

    assert_approx_eq!(bez.point_at(0.0), Point2::new(220.0, 40.0));
    assert_approx_eq!(bez.point_at(1.0), Point2::new(135.0, 210.0));

    assert_approx_eq!(bez.point_at(0.5), Point2::new(113.75, 152.5));
    assert_approx_eq_eps!(bez.point_at(0.3), Point2::new(140.95, 114.1), 0.00001);
}
