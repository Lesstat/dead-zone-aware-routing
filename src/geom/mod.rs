use std::f64::consts::PI;

pub fn project(point: (f64, f64), lat0: f64) -> (f64, f64) {
    let degree = 2.0 * PI / 360.0;
    let radius = 6371007.2; // meters
    let point = (point.0 * degree, point.1 * degree);

    (radius * point.0, radius * lat0.cos() * point.1)
}


pub fn intersect(a: (f64, f64), b: (f64, f64), center: (f64, f64), r: f64) -> Vec<f64> {
    let mut result = Vec::new();
    let m = (b.1 - a.1) / (b.0 - a.0);

    let c = a.1 + m * (-a.0);
    let a_quad = m * m + 1.0;
    let b_quad = -2.0 * center.0 + (c - center.1) * 2.0 * m;
    let c_quad = c * c + center.1 * center.1 - 2.0 * c * center.1 - r * r + center.0 * center.0;
    let d_quad = b_quad * b_quad - 4.0 * a_quad * c_quad;
    println!(
        "m: {}, c:{}, A:{}, B:{}, C:{}, D: {}",
        m,
        c,
        a_quad,
        b_quad,
        c_quad,
        d_quad
    );
    if d_quad > 0.0 {
        let x1 = (-b_quad + d_quad.sqrt()) / (2.0 * a_quad);
        let x2 = (-b_quad - d_quad.sqrt()) / (2.0 * a_quad);
        //let y1 = m * x1 + c;
        //let y2 = m * x2 + c;
        let t1 = (x1 - a.0) / (b.0 - a.0);
        let t2 = (x2 - a.0) / (b.0 - a.0);

        println!("x1: {}, x2: {}, t1: {}, t2: {}", x1, x2, t1, t2);
        if t1 >= 0.0 && t1 <= 1.0 {
            result.push(t1);
        }
        if t2 >= 0.0 && t2 <= 2.0 {
            result.push(t2);
        }
    }


    result
}


#[test]
fn empty_circle_segment_intersection() {
    let result = intersect((1.0, 1.0), (2.0, 2.0), (5.0, 5.0), 1.0);
    assert_eq!(0, result.len());
}

#[test]
fn one_circle_segment_intersection() {
    let result = intersect((1.0, 1.0), (2.0, 2.0), (3.0, 2.0), 1.0);
    assert_eq!(vec![1.0], result);
}
