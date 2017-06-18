
#[derive(Clone, Debug)]
struct Point {
    x: isize,
    y: isize,
    grid_size: isize,
}

impl Point {
    fn from_index(index: isize, grid_size: isize) -> Point {
        let y = index / grid_size;
        let x = index - y * grid_size;
        Point { x, y, grid_size }
    }

    fn to_index(&self) -> isize {

        let index = self.y * self.grid_size as isize + self.x;
        if index < 0 {
            panic!("i need to handle this")
        }
        index as isize

    }
}

pub struct RadiusIter {
    center: Point,
    grid_size: isize,
    radius: isize,
    next_point: Option<Point>,
}

impl RadiusIter {
    fn new(center: isize, grid_size: isize) -> RadiusIter {
        let center = Point::from_index(center, grid_size);
        let next_point = Some(center.clone());
        println!("center: {:?}", center);

        RadiusIter {
            center,
            grid_size,
            radius: 0,
            next_point,
        }
    }
}

impl Iterator for RadiusIter {
    type Item = isize;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next;
        let cur_point = self.next_point.take().unwrap();
        if cur_point.x - self.center.x == self.radius &&
            cur_point.y - self.center.y == self.radius
        {
            self.radius += 1;
            next = Point {
                x: self.center.x - self.radius,
                y: self.center.y - self.radius,
                grid_size: self.grid_size,
            }
        } else {
            next = Point {
                x: cur_point.x + 1,
                ..cur_point
            };
            self.check_for_line_wrap(&mut next);
            while !((next.x - self.center.x).abs() == self.radius ||
                        (next.y - self.center.y).abs() == self.radius)
            {
                next.x += 1;
                self.check_for_line_wrap(&mut next);
            }
        }

        self.next_point = Some(next);
        Some(cur_point.to_index())
    }
}

impl RadiusIter {
    fn check_for_line_wrap(&self, next: &mut Point) {
        if (next.x - self.center.x).abs() > self.radius {
            next.x = self.center.x - self.radius;
            next.y += 1;
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_with_center() {
        let mut r = RadiusIter::new(20, 6);
        assert_eq!(20, r.next().unwrap());
    }

    #[test]
    fn circle_around_center() {

        let mut r = RadiusIter::new(20, 6);
        r.next();

        assert_eq!(13, r.next().unwrap());
        assert_eq!(14, r.next().unwrap());
        assert_eq!(15, r.next().unwrap());
        assert_eq!(19, r.next().unwrap());
        assert_eq!(21, r.next().unwrap());
        assert_eq!(25, r.next().unwrap());
        assert_eq!(26, r.next().unwrap());
        assert_eq!(27, r.next().unwrap());
        assert_eq!(6, r.next().unwrap());
        assert_eq!(7, r.next().unwrap());
        assert_eq!(8, r.next().unwrap());
        assert_eq!(9, r.next().unwrap());
        assert_eq!(10, r.next().unwrap());
        assert_eq!(12, r.next().unwrap());
        assert_eq!(16, r.next().unwrap());
        assert_eq!(18, r.next().unwrap());
        assert_eq!(22, r.next().unwrap());
        assert_eq!(24, r.next().unwrap());
        assert_eq!(28, r.next().unwrap());
        assert_eq!(30, r.next().unwrap());
        assert_eq!(31, r.next().unwrap());
        assert_eq!(32, r.next().unwrap());
        assert_eq!(33, r.next().unwrap());
        assert_eq!(34, r.next().unwrap());
    }

}
