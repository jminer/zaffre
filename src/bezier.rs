
use std::ops::{Sub};
use super::Rect;
use super::nalgebra::{BaseFloat, Cast, cast, Point2, Vector2};

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

impl<N, F> Bezier<N> where F: BaseFloat
                            + Cast<N>
                            + Cast<f32>,
                           N: Copy
                            + Cast<F>
                            + LargerFloat<Float = F>
                            + Sub<Output = N> {
    // fn split_using_matrix(&self, t: f32) -> (Bezier<N>, Bezier<N>) {
    // }

    fn pt_to_float<T: Copy + Cast<N>>(t: Point2<N>) -> Point2<T> {
        cast(t)
    }
    fn vec_to_float<T: Copy + Cast<N>>(t: Vector2<N>) -> Vector2<T> {
        cast(t)
    }
    fn split_using_de_casteljau(&self, t: f32) -> (Bezier<N>, Bezier<N>) {
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

        let lv3p = lv1p0 + (lv1p1 - lv1p0) * t;

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
