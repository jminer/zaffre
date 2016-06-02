
use std::fmt::Debug;
use std::ops::{Add, Sub, Mul, Div, Neg};
use super::nalgebra::{Cast, cast, Point2, Transpose, zero};
use super::num::Zero;


/// A size is a width and height.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Size2<N> {
    pub width: N,
    pub height: N,
}

impl<N> Size2<N> {
    pub fn new(width: N, height: N) -> Self {
        Size2 {
            width: width,
            height: height,
        }
    }
}

impl<N: Copy> Transpose for Size2<N> {
    /// Swaps the width and height.
    #[inline]
    fn transpose(&self) -> Self {
        Size2::new(self.height, self.width)
    }

    #[inline]
    fn transpose_mut(&mut self) {
        let tmp = self.height;
        self.width = self.height;
        self.height = tmp;
    }
}


impl<N> Size2<N> where N: Copy
                        + Div<Output = N>,
                     f64: Cast<N> {
    /// Returns the aspect ratio of this size (width / height).
    #[inline]
    pub fn ratio(&self) -> f64 {
        cast::<N, f64>(self.width) / cast::<N, f64>(self.height)
    }
}

impl<N> Size2<N> where N: Copy
                        + PartialOrd
                        + Mul<Output = N>
                        + Div<Output = N>
                        + Cast<f64>
                        + Debug,
                     f64: Cast<N> {
    // TODO: add an image or two like Qt has: http://doc.qt.io/qt-5/qt.html#AspectRatioMode-enum
    // and add examples

    /// Scales the size to be as large as possible inside `other`.
    /// The aspect ratio is preserved.
    pub fn scale_keep_ratio(&self, other: &Size2<N>) -> Size2<N> {
        let ratio = self.ratio();
        if cast::<N, f64>(other.width) < cast::<N, f64>(other.height) * ratio {
            Size2::new(other.width, cast(cast::<N, f64>(other.width) / ratio))
        } else {
            Size2::new(cast(cast::<N, f64>(other.height) * ratio), other.height)
        }
    }

    /// Scales the size to be as small as possible while still containing `other`.
    /// The aspect ratio is preserved.
    pub fn scale_outside_keep_ratio(&self, other: &Size2<N>) -> Size2<N> {
        let ratio = self.ratio();
        if cast::<N, f64>(other.width) < cast::<N, f64>(other.height) * ratio {
            Size2::new(cast(cast::<N, f64>(other.height) * ratio), other.height)
        } else {
            Size2::new(other.width, cast(cast::<N, f64>(other.width) / ratio))
        }
    }
}

impl<N: Add<Output = N>> Add for Size2<N> {
    type Output = Size2<N>;

    fn add(self, other: Size2<N>) -> Size2<N> {
        Size2::new(self.width + other.width, self.height + other.height)
    }
}

impl<N: Sub<Output = N>> Sub for Size2<N> {
    type Output = Size2<N>;

    fn sub(self, other: Size2<N>) -> Size2<N> {
        Size2::new(self.width - other.width, self.height - other.height)
    }
}

#[test]
fn test_size() {
    let sz = Size2::new(2i32, 7);
    assert!(sz.width == 2 && sz.height == 7); // width and height

    let sz2 = sz + Size2::new(3, -11); // i32 add
    assert_eq!(&sz2, &Size2::new(5, -4));
    let sz2 = sz - Size2::new(3, -11); // i32 sub
    assert_eq!(&sz2, &Size2::new(-1, 18));

    // f64 add and sub
    assert_eq!(&(Size2::new(5.0f64, 6.0) + Size2::new(1.0, -7.5)), &Size2::new(6.0, -1.5));
    assert_eq!(&(Size2::new(5.0f64, 6.0) - Size2::new(1.0, -7.5)), &Size2::new(4.0, 13.5));

    assert_eq!(&Size2::new(9.0, 6.0).ratio(), &1.5);
    assert_eq!(&Size2::new(9, 6).ratio(), &1.5);

    assert_eq!(&Size2::new(33.0, 24.0).scale_keep_ratio(&Size2::new(16.0, 8.0)), &Size2::new(11.0, 8.0));
    assert_eq!(&Size2::new(24.0, 33.0).scale_keep_ratio(&Size2::new(8.0, 16.0)), &Size2::new(8.0, 11.0));
    assert_eq!(&Size2::new(33.0, 24.0).scale_keep_ratio(&Size2::new(160.0, 80.0)), &Size2::new(110.0, 80.0));
    assert_eq!(&Size2::new(24.0, 33.0).scale_keep_ratio(&Size2::new(80.0, 160.0)), &Size2::new(80.0, 110.0));

    assert_eq!(&Size2::new(32.0, 24.0).scale_outside_keep_ratio(&Size2::new(16.0, 8.0)), &Size2::new(16.0, 12.0));
    assert_eq!(&Size2::new(24.0, 32.0).scale_outside_keep_ratio(&Size2::new(8.0, 16.0)), &Size2::new(12.0, 16.0));
    assert_eq!(&Size2::new(32.0, 24.0).scale_outside_keep_ratio(&Size2::new(160.0, 80.0)),
               &Size2::new(160.0, 120.0));
    assert_eq!(&Size2::new(24.0, 32.0).scale_outside_keep_ratio(&Size2::new(80.0, 160.0)),
               &Size2::new(120.0, 160.0));

    assert_eq!(&Size2::new(33, 24).scale_keep_ratio(&Size2::new(16, 8)), &Size2::new(11, 8));
}


/// A rectangle consisting of a point and size (x, y, width, height)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Rect<N> {
    // TODO: should this struct be named Box2? Its the only reasonable name from
    // https://en.wikipedia.org/wiki/Hyperrectangle
    pub x: N,
    pub y: N,
    pub width: N,
    pub height: N,
}

impl<N> Rect<N> {
    pub fn new(x: N, y: N, width: N, height: N) -> Self {
        Rect {
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }
}

impl<N> Rect<N> where N: Copy + Add<Output = N> {
    #[inline]
    pub fn right(&self) -> N {
        self.x + self.width
    }

    #[inline]
    pub fn bottom(&self) -> N {
        self.y + self.height
    }

    #[inline]
    pub fn top_left(&self) -> Point2<N> {
        Point2::new(self.x, self.y)
    }

    #[inline]
    pub fn top_right(&self) -> Point2<N> {
        Point2::new(self.right(), self.y)
    }

    #[inline]
    pub fn bottom_right(&self) -> Point2<N> {
        Point2::new(self.right(), self.bottom())
    }

    #[inline]
    pub fn bottom_left(&self) -> Point2<N> {
        Point2::new(self.x, self.bottom())
    }

    #[inline]
    pub fn size(&self) -> Size2<N> {
        Size2::new(self.width, self.height)
    }
}

impl<N> Rect<N> where N: Copy
                       + Add<Output = N>
                       + Zero
                       + PartialOrd {
    /// Returns true if `pt` is inside this rectangle.
    pub fn contains_pt(&self, pt: Point2<N>) -> bool {
        // A point is NOT treated like a zero size rectangle.
        // A point on the right edge is not in the rectangle, but if two rectangles' right
        // edges are at the same location, one rectangle can still contain the other.
        debug_assert!(self.width >= zero() && self.height >= zero());
        pt.x >= self.x && pt.x < self.right() &&
        pt.y >= self.y && pt.y < self.bottom()
    }

    /// Returns true if `rect` is inside this rectangle.
    /// The rectangle's width and height must be positive.
    pub fn contains_rect(&self, rect: Rect<N>) -> bool {
        debug_assert!(self.width >= zero() && self.height >= zero());
        rect.x >= self.x && rect.right() <= self.right() &&
        rect.y >= self.y && rect.bottom() <= self.bottom()
    }
}

impl<N> Rect<N> where N: Copy
                       + Add<Output = N>
                       + Neg<Output = N>
                       + Zero
                       + PartialOrd {
    pub fn normalize(&self) -> Rect<N> {
        let (x, width) = if self.width < zero() {
            (self.x + self.width, -self.width)
        } else {
            (self.x, self.width)
        };
        let (y, height) = if self.height < zero() {
            (self.y + self.height, -self.height)
        } else {
            (self.y, self.height)
        };
        Rect::new(x, y, width, height)
    }
}

#[test]
fn test_rect() {
    let rect = Rect::new(10, 12, 3, 4);
    assert_eq!(&rect.right(), &13);
    assert_eq!(&rect.bottom(), &16);
    assert_eq!(&rect.top_left(), &Point2::new(10, 12));
    assert_eq!(&rect.top_right(), &Point2::new(13, 12));
    assert_eq!(&rect.bottom_right(), &Point2::new(13, 16));
    assert_eq!(&rect.bottom_left(), &Point2::new(10, 16));
    assert_eq!(&Rect::new(1.5, 2.0, 3.5, 3.5).bottom_right(), &Point2::new(5.0, 5.5));

    assert!(Rect::new(4, 5, 3, 3).contains_pt(Point2::new(4, 6)));
    assert!(Rect::new(4, 5, 3, 3).contains_pt(Point2::new(6, 6)));
    assert!(!Rect::new(4, 5, 3, 3).contains_pt(Point2::new(7, 6)));
    assert!(Rect::new(4, 5, 3, 3).contains_pt(Point2::new(5, 5)));
    assert!(Rect::new(4, 5, 3, 3).contains_pt(Point2::new(5, 7)));
    assert!(!Rect::new(4, 5, 3, 3).contains_pt(Point2::new(5, 8)));
    assert!(!Rect::new(4, 5, 3, 3).contains_pt(Point2::new(2, 3)));
    assert!(!Rect::new(4, 5, 3, 3).contains_pt(Point2::new(10, 15)));
    // if I remove the f64 suffix in the next code line:
    // error: internal compiler error: Impl DefId { krate: 0, node: 114 }:coordinates::i32.NumZero was matchable against Obligation(predicate=Binder(TraitPredicate(coordinates::NumZero)),depth=0) but now is not
    assert!(Rect::new(4.0f64, 5.0, 3.0, 3.0).contains_pt(Point2::new(4.5, 6.1)));

    assert!(Rect::new(4, 5, 3, 3).contains_rect(Rect::new(5, 6, 1, 1)));
    assert!(Rect::new(4, 5, 3, 3).contains_rect(Rect::new(4, 5, 3, 3)));
    assert!(!Rect::new(4, 5, 3, 3).contains_rect(Rect::new(5, 6, 3, 3)));
    assert!(!Rect::new(4, 5, 3, 3).contains_rect(Rect::new(5, 6, 4, 1)));
    assert!(!Rect::new(4, 5, 3, 3).contains_rect(Rect::new(5, 6, 1, 4)));
    assert!(!Rect::new(4, 5, 3, 3).contains_rect(Rect::new(5, 4, 1, 4)));
    assert!(!Rect::new(4, 5, 3, 3).contains_rect(Rect::new(3, 6, 4, 1)));
    assert!(!Rect::new(4, 5, 3, 3).contains_rect(Rect::new(20, 30, 5, 5)));
    assert!(!Rect::new(4, 5, 3, 3).contains_rect(Rect::new(-20, -30, 5, 5)));
    assert!(Rect::new(4.0f64, 5.0, 3.0, 3.0).contains_rect(Rect::new(5.6, 6.0, 1.3, 0.5)));
    assert!(!Rect::new(4.0f64, 5.0, 3.0, 3.0).contains_rect(Rect::new(5.6, 6.0, 1.5, 0.5)));

    assert_eq!(&Rect::new(10, 15, 2, 3).normalize(), &Rect::new(10, 15, 2, 3));
    assert_eq!(&Rect::new(10, 15, -2, 3).normalize(), &Rect::new(8, 15, 2, 3));
    assert_eq!(&Rect::new(10, 15, 2, -3).normalize(), &Rect::new(10, 12, 2, 3));
    assert_eq!(&Rect::new(10, 15, -2, -3).normalize(), &Rect::new(8, 12, 2, 3));
    assert_eq!(&Rect::new(10.0f64, 15.0, -2.0, 3.0).normalize(), &Rect::new(8.0, 15.0, 2.0, 3.0));
}

impl<N> Add<BorderSize2<N>> for Rect<N> where N: Copy
                                               + Add<Output = N>
                                               + Sub<Output = N> {
    type Output = Rect<N>;

    /// Adds the border size to the rectangle. A positive border size will enlarge the rectangle.
    fn add(self, other: BorderSize2<N>) -> Rect<N> {
        Rect::new(self.x - other.left,
                  self.y - other.top,
                  self.width + other.left + other.right,
                  self.height + other.top + other.bottom)
    }
}

impl<N> Sub<BorderSize2<N>> for Rect<N> where N: Copy
                                               + Add<Output = N>
                                               + Sub<Output = N> {
    type Output = Rect<N>;

    /// Adds the border size to the rectangle. A positive border size will shrink the rectangle.
    fn sub(self, other: BorderSize2<N>) -> Rect<N> {
        Rect::new(self.x + other.left,
                  self.y + other.top,
                  self.width - other.left - other.right,
                  self.height - other.top - other.bottom)
    }
}

#[test]
fn test_ops() {
    assert_eq!(&(Rect::new(3, 10, 20, 15) + BorderSize2::new(2, 3, 6, 9)), &Rect::new(1, 7, 28, 27));
    assert_eq!(&(Rect::new(3, 10, 20, 15) - BorderSize2::new(2, 3, 6, 9)), &Rect::new(5, 13, 12, 3));
}


/// A border size stores a size for the left, top, right, and bottom edges of a rectangle
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BorderSize2<N> {
    pub left: N,
    pub top: N,
    pub right: N,
    pub bottom: N,
}

impl<N> BorderSize2<N> {
    pub fn new(left: N, top: N, right: N, bottom: N) -> Self {
        BorderSize2 {
            left: left,
            top: top,
            right: right,
            bottom: bottom,
        }
    }
}

impl<N> Add for BorderSize2<N> where N: Copy + Add<Output = N> {
    type Output = BorderSize2<N>;

    fn add(self, other: BorderSize2<N>) -> BorderSize2<N> {
        BorderSize2::new(self.left + other.left, self.top + other.top,
                         self.right + other.right, self.bottom + other.bottom)
    }
}

impl<N> Sub for BorderSize2<N> where N: Copy + Sub<Output = N> {
    type Output = BorderSize2<N>;

    fn sub(self, other: BorderSize2<N>) -> BorderSize2<N> {
        BorderSize2::new(self.left - other.left, self.top - other.top,
                         self.right - other.right, self.bottom - other.bottom)
    }
}

#[test]
fn test_border_size() {
    assert_eq!(&(BorderSize2::new(3, 4, 6, 9) + BorderSize2::new(10, 20, 30, 40)),
               &BorderSize2::new(13, 24, 36, 49));
    assert_eq!(&(BorderSize2::new(3, 4, 6, 9) - BorderSize2::new(10, 20, 30, 40)),
               &BorderSize2::new(-7, -16, -24, -31));
}
