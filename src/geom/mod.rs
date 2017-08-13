use std::f64::consts::PI;
use std::f64::EPSILON;

use graph::Length;

pub trait Coord {
    fn lat(&self) -> f64;
    fn lon(&self) -> f64;
}

pub trait Point {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
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

pub fn project<C: Coord>(point: &C, lat0: f64) -> TuplePoint {
    let degree = 2.0 * PI / 360.0;
    let radius = 6371007.2; // meters
    let point = (point.lat() * degree, point.lon() * degree);

    (radius * point.0, radius * lat0.cos() * point.1)
}


pub fn intersect<P: Point>(a: &P, b: &P, center: &P, r: f64) -> SegmentSection {
    assert!((b.x() - a.x()).abs() >= EPSILON);
    let m = (b.y() - a.y()) / (b.x() - a.x());
    let c = a.y() + m * (-a.x());
    let a_quad = m * m + 1.0;
    let b_quad = -2.0 * center.x() + (c - center.y()) * 2.0 * m;
    let c_quad = c * c + center.y() * center.y() - 2.0 * c * center.y() - r * r +
        center.x() * center.x();
    let d_quad = b_quad * b_quad - 4.0 * a_quad * c_quad;

    if d_quad > 0.0 {
        let x1 = (-b_quad + d_quad.sqrt()) / (2.0 * a_quad);
        let x2 = (-b_quad - d_quad.sqrt()) / (2.0 * a_quad);
        let t1 = (x1 - a.x()) / (b.x() - a.x());
        let t2 = (x2 - a.x()) / (b.x() - a.x());
        return SegmentSection::new(t1, t2);
    }
    SegmentSection::empty()

}

#[derive(Debug, PartialEq)]
pub struct SegmentSection {
    start: f64,
    end: f64,
}

impl SegmentSection {
    fn empty() -> SegmentSection {
        SegmentSection {
            start: 0.0,
            end: 0.0,
        }
    }

    fn new(first: f64, second: f64) -> SegmentSection {
        let start = SegmentSection::normalize(first.min(second));
        let end = SegmentSection::normalize(first.max(second));
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
    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.end - self.start <= 0.0
    }
}

// Adapted from https://github.com/georust/rust-geo
pub fn haversine_distance<C: Coord>(a: &C, b: &C) -> Length {
    let theta1 = a.lat().to_radians();
    let theta2 = b.lat().to_radians();
    let delta_theta = (b.lat() - a.lat()).to_radians();
    let delta_lambda = (b.lon() - a.lon()).to_radians();
    let a = (delta_theta / 2.0).sin().powi(2) +
        theta1.cos() * theta2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    // WGS84 equatorial radius is 6378137.0
    6371.0 * c
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
