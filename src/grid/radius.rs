
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
    cur_radius: isize,
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
            cur_radius: 0,
            next_point,
        }
    }
}

impl Iterator for RadiusIter {
    type Item = isize;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next;
        if self.cur_radius == 0 {
            self.cur_radius += 1;
            next = Point {
                x: self.center.x - self.cur_radius,
                y: self.center.y - self.cur_radius,
                grid_size: self.grid_size,
            }
        } else {
            let cur_point = self.next_point.clone().unwrap();
            next = Point {
                x: cur_point.x + 1,
                ..cur_point
            };
            if (next.x - self.center.x).abs() > self.cur_radius {
                next.x = self.center.x - self.cur_radius;
                next.y += 1;
            }
            while !((next.x - self.center.x).abs() == self.cur_radius ||
                        (next.y - self.center.y).abs() == self.cur_radius)
            {
                next.x += 1;
                if (next.x - self.center.x).abs() > self.cur_radius {
                    next.x = self.center.x - self.cur_radius;
                    next.y += 1;
                }
            }
        }

        let mut next = Some(next);
        ::std::mem::swap(&mut next, &mut self.next_point);
        next.map(|p| p.to_index())
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
    }

}
