
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

/// Iterator that returns cell indices that form a rectangle with a
/// certain distance to the given center index
pub struct RadiusIter {
    center: Point,
    grid_size: isize,
    radius: isize,
    next_point: Option<Point>,
}

impl RadiusIter {
    pub fn new(center: isize, grid_size: isize, radius: isize) -> RadiusIter {
        let center = Point::from_index(center, grid_size);
        let mut rad = RadiusIter {
            center,
            grid_size,
            radius,
            next_point: None,
        };
        rad.next_point = RadiusIter::calculate_starting_point(&rad).ok();
        rad
    }
}

impl Iterator for RadiusIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next;
        let cur_point = match self.next_point.take() {
            Some(p) => p,
            None => return None,
        };

        if cur_point.x - self.center.x == self.radius &&
            cur_point.y - self.center.y == self.radius
        {
            self.next_point = None;
            return Some(cur_point.to_index() as usize);

        } else {
            next = Point {
                x: cur_point.x + 1,
                ..cur_point
            };

            if self.check_for_line_wrap(&mut next).is_err() {
                self.next_point = None;
                return Some(cur_point.to_index() as usize);
            };

            while !((next.x - self.center.x).abs() == self.radius ||
                        (next.y - self.center.y).abs() == self.radius) ||
                (next.x < 0 || next.y < 0)
            {
                next.x += 1;
                if self.check_for_line_wrap(&mut next).is_err() {
                    self.next_point = None;
                    return Some(cur_point.to_index() as usize);
                };
            }

        }

        self.next_point = Some(next);
        Some(cur_point.to_index() as usize)
    }
}

impl RadiusIter {
    fn check_for_line_wrap(&mut self, next: &mut Point) -> Result<(), ()> {
        if (next.x - self.center.x).abs() > self.radius || next.x >= self.grid_size {

            next.x = self.center.x - self.radius;
            next.y += 1;

            if (next.y - self.center.y).abs() > self.radius || next.y >= self.grid_size {

                return Err(());
            }
        }
        Ok(())
    }

    fn calculate_starting_point(rad_iter: &RadiusIter) -> Result<Point, ()> {
        let mut x = rad_iter.center.x - rad_iter.radius;
        if x < 0 {
            x = rad_iter.center.x + rad_iter.radius;
            if x >= rad_iter.grid_size {
                return Err(());
            }
        }
        let mut y = rad_iter.center.y - rad_iter.radius;
        if y < 0 {
            y = 0;
        }
        let p = Point {
            x,
            y,
            grid_size: rad_iter.grid_size,
        };
        Ok(p)

    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_with_center() {
        let mut r = RadiusIter::new(20, 6, 0);
        assert_eq!(20, r.next().unwrap());
    }

    #[test]
    fn circle_around_center() {

        let mut r = RadiusIter::new(20, 6, 1);

        assert_eq!(Some(13), r.next());
        assert_eq!(Some(14), r.next());
        assert_eq!(Some(15), r.next());
        assert_eq!(Some(19), r.next());
        assert_eq!(Some(21), r.next());
        assert_eq!(Some(25), r.next());
        assert_eq!(Some(26), r.next());
        assert_eq!(Some(27), r.next());
        assert_eq!(None, r.next());
        r = RadiusIter::new(20, 6, 2);
        assert_eq!(Some(6), r.next());
        assert_eq!(Some(7), r.next());
        assert_eq!(Some(8), r.next());
        assert_eq!(Some(9), r.next());
        assert_eq!(Some(10), r.next());
        assert_eq!(Some(12), r.next());
        assert_eq!(Some(16), r.next());
        assert_eq!(Some(18), r.next());
        assert_eq!(Some(22), r.next());
        assert_eq!(Some(24), r.next());
        assert_eq!(Some(28), r.next());
        assert_eq!(Some(30), r.next());
        assert_eq!(Some(31), r.next());
        assert_eq!(Some(32), r.next());
        assert_eq!(Some(33), r.next());
        assert_eq!(Some(34), r.next());
    }

    #[test]
    fn leave_out_indices_for_edge_cells_x_direction() {
        let mut r = RadiusIter::new(17, 6, 1);
        assert_eq!(Some(10), r.next());
        assert_eq!(Some(11), r.next());
        assert_eq!(Some(16), r.next());
        assert_eq!(Some(22), r.next());
        assert_eq!(Some(23), r.next());
    }

    #[test]
    fn leave_out_indices_for_edge_cells_y_direction() {
        let mut r = RadiusIter::new(35, 6, 1);
        assert_eq!(28, r.next().unwrap());
        assert_eq!(29, r.next().unwrap());
        assert_eq!(34, r.next().unwrap());
        assert_eq!(None, r.next());
    }

    #[test]
    fn hop_over_negative_cell_indices() {
        let mut r = RadiusIter::new(0, 3, 1);
        assert_eq!(1, r.next().unwrap());
        assert_eq!(3, r.next().unwrap());
        assert_eq!(4, r.next().unwrap());
        r = RadiusIter::new(0, 3, 2);
        assert_eq!(2, r.next().unwrap());
        assert_eq!(5, r.next().unwrap());
        assert_eq!(6, r.next().unwrap());
        assert_eq!(7, r.next().unwrap());
        assert_eq!(8, r.next().unwrap());
        r = RadiusIter::new(0, 3, 3);
        assert_eq!(None, r.next());
    }


}
