use std::f64::consts::PI;

use ordered_float::OrderedFloat;

use graph::Length;

const EARTH_RADIUS: f64 = 6_371_007.2;

/// Allow uniform access to structs with spherical coordinates
pub trait Coord {
    fn lat(&self) -> f64;
    fn lon(&self) -> f64;
}

/// Allow uniform access to structs with coordinates on a plain as
/// well as multiplying and subtraction operations that work like a -
/// b = (a.x - b.x, a.y - b.y)
pub trait Point {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn sub(&self, rhs: &Point) -> TuplePoint {
        (self.x() - rhs.x(), self.y() - rhs.y())
    }
    fn mul(&self, rhs: &Point) -> TuplePoint {
        (self.x() * rhs.x(), self.y() * rhs.y())
    }
    fn sum(&self) -> f64 {
        self.x() + self.y()
    }
}


pub type TuplePoint = (f64, f64);

impl Point for TuplePoint {
    #[inline]
    fn x(&self) -> f64 {
        self.0
    }
    #[inline]
    fn y(&self) -> f64 {
        self.1
    }
}



impl Coord for TuplePoint {
    #[inline]
    fn lat(&self) -> f64 {
        self.0
    }
    #[inline]
    fn lon(&self) -> f64 {
        self.1
    }
}

/// Projects a coordinate to a point relative to latitude
pub fn project<C: Coord>(point: &C, lat0: f64) -> TuplePoint {
    let degree = 2.0 * PI / 360.0;
    let point = (point.lat() * degree, point.lon() * degree);
    let x = EARTH_RADIUS * point.0;
    let y = EARTH_RADIUS * lat0.cos() * point.1;
    assert!(!x.is_nan(), "x is NaN");
    assert!(!y.is_nan(), "y is NaN");
    (x, y)
}


/// Intersect a line segment defined by points `a` and `b` with a circle
/// with center `center` and radius `r`.
///inspiration from https://gis.stackexchange.com/questions/36841/line-intersection-with-circle-on-a-sphere-globe-or-earth/36979#36979
pub fn intersect<P: Point>(a: &P, b: &P, center: &P, r: f64) -> SegmentSection {
    let v = a.sub(center);
    let u = b.sub(a);
    let alpha = u.mul(&u).sum();
    let beta = u.mul(&v).sum();
    let gamma = v.mul(&v).sum() - r * r;

    let tmp = beta.powi(2) - alpha * gamma;
    if tmp <= 0f64 {
        return SegmentSection::empty();
    }
    let t1 = (-beta + tmp.sqrt()) / alpha;
    let t2 = (-beta - tmp.sqrt()) / alpha;
    assert!(!t1.is_nan(), "t1 is NaN");
    assert!(!t2.is_nan(), "t2 is NaN");

    SegmentSection::new(t1, t2)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SegmentSection {
    start: OrderedFloat<f64>,
    end: OrderedFloat<f64>,
}

impl SegmentSection {
    fn empty() -> SegmentSection {
        SegmentSection {
            start: 0.0.into(),
            end: 0.0.into(),
        }
    }

    fn new(first: f64, second: f64) -> SegmentSection {
        let start = SegmentSection::normalize(first.min(second)).into();
        let end = SegmentSection::normalize(first.max(second)).into();
        SegmentSection { start, end }
    }
    fn normalize(value: f64) -> f64 {
        if value < 0.0 {
            0.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        }
    }
    pub fn is_empty(&self) -> bool {
        self.length() <= 0.0
    }

    pub fn is_full(&self) -> bool {
        self.length() >= 1.0
    }

    pub fn is_overlapping(&self, other: &Self) -> bool {
        if self.start < other.start {
            self.end >= other.start
        } else {
            other.end >= self.start
        }

    }
    pub fn merge(&self, other: &Self) -> SegmentSection {
        let start = if self.start < other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end > other.end {
            self.end
        } else {
            other.end
        };
        SegmentSection { start, end }
    }
    pub fn length(&self) -> f64 {
        self.end.into_inner() - self.start.into_inner()
    }
}

/// Calculate the haversine distance. Adapted from https://github.com/georust/rust-geo
pub fn haversine_distance<C1: Coord, C2: Coord>(a: &C1, b: &C2) -> Length {
    let theta1 = a.lat().to_radians();
    let theta2 = b.lat().to_radians();
    let delta_theta = (b.lat() - a.lat()).to_radians();
    let delta_lambda = (b.lon() - a.lon()).to_radians();
    let a = (delta_theta / 2.0).sin().powi(2) +
        theta1.cos() * theta2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS * c
}

#[test]
fn empty_circle_segment_intersection() {
    let result = intersect(&(1.0, 1.0), &(2.0, 2.0), &(5.0, 5.0), 1.0);
    assert_eq!(true, result.is_empty());
}

#[test]
fn circle_touches_segment_intersection() {
    let result = intersect(&(1.0, 1.0), &(2.0, 2.0), &(3.0, 2.0), 1.0);
    assert_eq!(true, result.is_empty());
}
#[test]
fn circle_intersects_in_middle_of_segment() {
    let result = intersect(&(1.0, 1.0), &(5.0, 1.0), &(3.0, 1.0), 1.0);
    assert_eq!(SegmentSection::new(0.25, 0.75), result);
    assert_eq!(false, result.is_empty());
}

#[test]
fn circle_includes_segment() {
    let result = intersect(&(1.0, 1.0), &(2.0, 2.0), &(3.0, 2.0), 10.0);
    assert_eq!(SegmentSection::new(0.0, 1.0), result);
    assert_eq!(false, result.is_empty());
}
#[test]
fn circle_includes_one_endpoint() {
    let result = intersect(&(1.0, 1.0), &(2.0, 1.0), &(1.0, 1.0), 0.5);
    assert_eq!(SegmentSection::new(0.0, 0.5), result);
    assert_eq!(false, result.is_empty());
}

#[test]
fn segment_goes_upward() {
    let result = intersect(&(1.0, 1.0), &(1.0, 2.0), &(1.0, 1.0), 0.5);
    assert_eq!(SegmentSection::new(0.0, 0.5), result);
    assert_eq!(false, result.is_empty());
}

#[test]
fn merge_segments() {
    let sec1 = SegmentSection::new(0.1, 0.4);
    let sec2 = SegmentSection::new(0.3, 0.6);
    assert_eq!(SegmentSection::new(0.1, 0.6), sec1.merge(&sec2));
    assert_eq!(sec2.merge(&sec1), sec1.merge(&sec2));
}
