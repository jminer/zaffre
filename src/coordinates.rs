
use std::fmt::Debug;
use std::ops::{Add, Sub, Mul, Div, Neg};

// TODO: get rid of ToF64 and FromF64 if the standard library gets any stable replacement
trait ToF64 {
    fn to_f64(self) -> f64;
}

impl ToF64 for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

trait FromF64 {
    fn from_f64(n: f64) -> Self;
}

impl FromF64 for i32 {
    fn from_f64(n: f64) -> i32 {
        n as i32
    }
}

impl FromF64 for f32 {
    fn from_f64(n: f64) -> f32 {
        n as f32
    }
}

impl FromF64 for f64 {
    fn from_f64(n: f64) -> f64 {
        n
    }
}

trait NumZero {
    fn zero() -> Self;
}

impl NumZero for i32 {
    fn zero() -> i32 {
        0
    }
}

impl NumZero for f32 {
    fn zero() -> f32 {
        0.0
    }
}

impl NumZero for f64 {
    fn zero() -> f64 {
        0.0
    }
}


/// A point is an x and y pair.
#[derive(Debug, Copy, PartialEq)]
pub struct Point<T = f64>(pub T, pub T);

impl<T> Point<T> where T: Copy {
    #[inline(always)]
    pub fn x(&self) -> T {
        self.0
    }

    #[inline(always)]
    pub fn y(&self) -> T {
        self.1
    }
}

impl<T: Add<Output = T>> Add for Point<T> {
    type Output = Point<T>;

    fn add(self, other: Point<T>) -> Point<T> {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

impl<T: Sub<Output = T>> Sub for Point<T> {
    type Output = Point<T>;

    fn sub(self, other: Point<T>) -> Point<T> {
        Point(self.0 - other.0, self.1 - other.1)
    }
}

impl<T: Add<Output = T>> Add<Size<T>> for Point<T> {
    type Output = Rect<T>;

    fn add(self, other: Size<T>) -> Rect<T> {
        Rect(self.0, self.1, other.0, other.1)
    }
}

#[test]
fn test_point() {
    let pt = Point(2i32, 7);
    assert!(pt.x() == 2 && pt.y() == 7); // x() and y()

    let pt2 = pt + Point(3, -11); // i32 add
    assert_eq!(&pt2, &Point(5, -4));
    let pt2 = pt - Point(3, -11); // i32 sub
    assert_eq!(&pt2, &Point(-1, 18));

    // f64 add and sub
    assert_eq!(&(Point(5.0f64, 6.0) + Point(1.0, -7.5)), &Point(6.0, -1.5));
    assert_eq!(&(Point(5.0f64, 6.0) - Point(1.0, -7.5)), &Point(4.0, 13.5));

    assert_eq!(&(Point(2, 4) + Size(6, 8)), &Rect(2, 4, 6, 8));
}


/// A size is a width and height.
#[derive(Debug, Copy, PartialEq)]
pub struct Size<T = f64>(pub T, pub T);

impl<T> Size<T> where T: Copy {
    #[inline(always)]
    pub fn width(&self) -> T {
        self.0
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        self.1
    }

    /// Swaps the width and height.
    #[inline]
    pub fn transpose(&self) -> Size<T> {
        Size(self.1, self.0)
    }
}

impl<T> Size<T> where T: Copy
                       + Div<Output = T>
                       + ToF64 {
    /// Returns the aspect ratio of this size (width / height).
    #[inline]
    pub fn ratio(&self) -> f64 {
        self.width().to_f64() / self.height().to_f64()
    }
}

impl<T> Size<T> where T: Copy
                       + PartialOrd
                       + Mul<Output = T>
                       + Div<Output = T>
                       + ToF64
                       + FromF64 + Debug {
    // TODO: add an image or two like Qt has: http://doc.qt.io/qt-5/qt.html#AspectRatioMode-enum
    // and add examples

    /// Scales the size to be as large as possible inside `other`.
    /// The aspect ratio is preserved.
    pub fn scale_keep_ratio(&self, other: &Size<T>) -> Size<T> {
        let ratio = self.ratio();
        if other.width().to_f64() < other.height().to_f64() * ratio {
            Size(other.width(), FromF64::from_f64(other.width().to_f64() / ratio))
        } else {
            Size(FromF64::from_f64(other.height().to_f64() * ratio), other.height())
        }
    }

    /// Scales the size to be as small as possible while still containing `other`.
    /// The aspect ratio is preserved.
    pub fn scale_outside_keep_ratio(&self, other: &Size<T>) -> Size<T> {
        let ratio = self.ratio();
        if other.width().to_f64() < other.height().to_f64() * ratio {
            Size(FromF64::from_f64(other.height().to_f64() * ratio), other.height())
        } else {
            Size(other.width(), FromF64::from_f64(other.width().to_f64() / ratio))
        }
    }
}

impl<T: Add<Output = T>> Add for Size<T> {
    type Output = Size<T>;

    fn add(self, other: Size<T>) -> Size<T> {
        Size(self.0 + other.0, self.1 + other.1)
    }
}

impl<T: Sub<Output = T>> Sub for Size<T> {
    type Output = Size<T>;

    fn sub(self, other: Size<T>) -> Size<T> {
        Size(self.0 - other.0, self.1 - other.1)
    }
}

#[test]
fn test_size() {
    let sz = Size(2i32, 7);
    assert!(sz.width() == 2 && sz.height() == 7); // width() and height()

    let sz2 = sz + Size(3, -11); // i32 add
    assert_eq!(&sz2, &Size(5, -4));
    let sz2 = sz - Size(3, -11); // i32 sub
    assert_eq!(&sz2, &Size(-1, 18));

    // f64 add and sub
    assert_eq!(&(Size(5.0f64, 6.0) + Size(1.0, -7.5)), &Size(6.0, -1.5));
    assert_eq!(&(Size(5.0f64, 6.0) - Size(1.0, -7.5)), &Size(4.0, 13.5));

    assert_eq!(&Size(9.0, 6.0).ratio(), &1.5);
    assert_eq!(&Size(9, 6).ratio(), &1.5);

    assert_eq!(&Size(33.0, 24.0).scale_keep_ratio(&Size(16.0, 8.0)), &Size(11.0, 8.0));
    assert_eq!(&Size(24.0, 33.0).scale_keep_ratio(&Size(8.0, 16.0)), &Size(8.0, 11.0));
    assert_eq!(&Size(33.0, 24.0).scale_keep_ratio(&Size(160.0, 80.0)), &Size(110.0, 80.0));
    assert_eq!(&Size(24.0, 33.0).scale_keep_ratio(&Size(80.0, 160.0)), &Size(80.0, 110.0));

    assert_eq!(&Size(32.0, 24.0).scale_outside_keep_ratio(&Size(16.0, 8.0)), &Size(16.0, 12.0));
    assert_eq!(&Size(24.0, 32.0).scale_outside_keep_ratio(&Size(8.0, 16.0)), &Size(12.0, 16.0));
    assert_eq!(&Size(32.0, 24.0).scale_outside_keep_ratio(&Size(160.0, 80.0)),
               &Size(160.0, 120.0));
    assert_eq!(&Size(24.0, 32.0).scale_outside_keep_ratio(&Size(80.0, 160.0)),
               &Size(120.0, 160.0));

    assert_eq!(&Size(33, 24).scale_keep_ratio(&Size(16, 8)), &Size(11, 8));
}



/// A rectangle consisting of a point and size (x, y, width, height)
#[derive(Debug, Copy, PartialEq)]
pub struct Rect<T = f64>(pub T, pub T, pub T, pub T);

impl<T> Rect<T> where T: Copy {
    #[inline(always)]
    pub fn x(&self) -> T {
        self.0
    }

    #[inline(always)]
    pub fn y(&self) -> T {
        self.1
    }

    #[inline(always)]
    pub fn width(&self) -> T {
        self.2
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        self.3
    }
}

impl<T> Rect<T> where T: Copy + Add<Output = T> {
    #[inline]
    pub fn right(&self) -> T {
        self.x() + self.width()
    }

    #[inline]
    pub fn bottom(&self) -> T {
        self.y() + self.height()
    }

    #[inline]
    pub fn top_left(&self) -> Point<T> {
        Point(self.x(), self.y())
    }

    #[inline]
    pub fn top_right(&self) -> Point<T> {
        Point(self.right(), self.y())
    }

    #[inline]
    pub fn bottom_right(&self) -> Point<T> {
        Point(self.right(), self.bottom())
    }

    #[inline]
    pub fn bottom_left(&self) -> Point<T> {
        Point(self.x(), self.bottom())
    }

    #[inline]
    pub fn size(&self) -> Size<T> {
        Size(self.width(), self.height())
    }
}

impl<T> Rect<T> where T: Copy + Add<Output = T> + NumZero + PartialOrd {
    /// Returns true if `pt` is inside this rectangle.
    fn contains_pt(&self, pt: Point<T>) -> bool {
        // A point is NOT treated like a zero size rectangle.
        // A point on the right edge is not in the rectangle, but if two rectangles' right
        // edges are at the same location, one rectangle can still contain the other.
        debug_assert!(self.width() >= NumZero::zero() && self.height() >= NumZero::zero());
        pt.x() >= self.x() && pt.x() < self.right() &&
        pt.y() >= self.y() && pt.y() < self.bottom()
    }

    /// Returns true if `rect` is inside this rectangle.
    /// The rectangle's width and height must be positive.
    fn contains_rect(&self, rect: Rect<T>) -> bool {
        debug_assert!(self.width() >= NumZero::zero() && self.height() >= NumZero::zero());
        rect.x() >= self.x() && rect.right() <= self.right() &&
        rect.y() >= self.y() && rect.bottom() <= self.bottom()
    }
}

impl<T> Rect<T> where T: Copy + Add<Output = T> + Neg<Output = T> + NumZero + PartialOrd {
    fn normalize(&self) -> Rect<T> {
        let (x, width) = if self.width() < NumZero::zero() {
            (self.x() + self.width(), -self.width())
        } else {
            (self.x(), self.width())
        };
        let (y, height) = if self.height() < NumZero::zero() {
            (self.y() + self.height(), -self.height())
        } else {
            (self.y(), self.height())
        };
        Rect(x, y, width, height)
    }
}

#[test]
fn test_rect() {
    let rect = Rect(10, 12, 3, 4);
    assert_eq!(&rect.right(), &13);
    assert_eq!(&rect.bottom(), &16);
    assert_eq!(&rect.top_left(), &Point(10, 12));
    assert_eq!(&rect.top_right(), &Point(13, 12));
    assert_eq!(&rect.bottom_right(), &Point(13, 16));
    assert_eq!(&rect.bottom_left(), &Point(10, 16));
    assert_eq!(&Rect(1.5, 2.0, 3.5, 3.5).bottom_right(), &Point(5.0, 5.5));

    assert!(Rect(4, 5, 3, 3).contains_pt(Point(4, 6)));
    assert!(Rect(4, 5, 3, 3).contains_pt(Point(6, 6)));
    assert!(!Rect(4, 5, 3, 3).contains_pt(Point(7, 6)));
    assert!(Rect(4, 5, 3, 3).contains_pt(Point(5, 5)));
    assert!(Rect(4, 5, 3, 3).contains_pt(Point(5, 7)));
    assert!(!Rect(4, 5, 3, 3).contains_pt(Point(5, 8)));
    assert!(!Rect(4, 5, 3, 3).contains_pt(Point(2, 3)));
    assert!(!Rect(4, 5, 3, 3).contains_pt(Point(10, 15)));
    // if I remove the f64 suffix in the next code line:
    // error: internal compiler error: Impl DefId { krate: 0, node: 114 }:coordinates::i32.NumZero was matchable against Obligation(predicate=Binder(TraitPredicate(coordinates::NumZero)),depth=0) but now is not
    assert!(Rect(4.0f64, 5.0, 3.0, 3.0).contains_pt(Point(4.5, 6.1)));

}

impl<T> Add<BorderSize<T>> for Rect<T> where T: Copy
                                              + Add<Output = T>
                                              + Sub<Output = T> {
    type Output = Rect<T>;

    /// Adds the border size to the rectangle. A positive border size will enlarge the rectangle.
    fn add(self, other: BorderSize<T>) -> Rect<T> {
        Rect(self.x() - other.left(),
             self.y() - other.top(),
             self.width() + other.left() + other.right(),
             self.height() + other.top() + other.bottom())
    }
}

impl<T> Sub<BorderSize<T>> for Rect<T> where T: Copy
                                              + Add<Output = T>
                                              + Sub<Output = T> {
    type Output = Rect<T>;

    /// Adds the border size to the rectangle. A positive border size will shrink the rectangle.
    fn sub(self, other: BorderSize<T>) -> Rect<T> {
        Rect(self.x() + other.left(),
             self.y() + other.top(),
             self.width() - other.left() - other.right(),
             self.height() - other.top() - other.bottom())
    }
}

#[test]
fn test_ops() {
    assert_eq!(&(Rect(3, 10, 20, 15) + BorderSize(2, 3, 6, 9)), &Rect(1, 7, 28, 27));
    assert_eq!(&(Rect(3, 10, 20, 15) - BorderSize(2, 3, 6, 9)), &Rect(5, 13, 12, 3));
}


/// A border size stores a size for the left, top, right, and bottom edges of a rectangle
#[derive(Debug, Copy, PartialEq)]
pub struct BorderSize<T = f64>(pub T, pub T, pub T, pub T);

impl<T> BorderSize<T> where T: Copy {
    #[inline(always)]
    pub fn left(&self) -> T {
        self.0
    }

    #[inline(always)]
    pub fn top(&self) -> T {
        self.1
    }

    #[inline(always)]
    pub fn right(&self) -> T {
        self.2
    }

    #[inline(always)]
    pub fn bottom(&self) -> T {
        self.3
    }
}

impl<T> Add for BorderSize<T> where T: Copy + Add<Output = T> {
    type Output = BorderSize<T>;

    fn add(self, other: BorderSize<T>) -> BorderSize<T> {
        BorderSize(self.0 + other.0, self.1 + other.1,
                   self.2 + other.2, self.3 + other.3)
    }
}

impl<T> Sub for BorderSize<T> where T: Copy + Sub<Output = T> {
    type Output = BorderSize<T>;

    fn sub(self, other: BorderSize<T>) -> BorderSize<T> {
        BorderSize(self.0 - other.0, self.1 - other.1,
                   self.2 - other.2, self.3 - other.3)
    }
}

#[test]
fn test_border_size() {
    assert_eq!(&(BorderSize(3, 4, 6, 9) + BorderSize(10, 20, 30, 40)),
               &BorderSize(13, 24, 36, 49));
    assert_eq!(&(BorderSize(3, 4, 6, 9) - BorderSize(10, 20, 30, 40)),
               &BorderSize(-7, -16, -24, -31));
}
