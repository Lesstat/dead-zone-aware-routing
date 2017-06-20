
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
    pub fn new(center: isize, grid_size: isize) -> RadiusIter {
        let center = Point::from_index(center, grid_size);
        let next_point = Some(center.clone());
        RadiusIter {
            center,
            grid_size,
            radius: 0,
            next_point,
        }
    }
}

impl Iterator for RadiusIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next;
        let cur_point = self.next_point.take().unwrap();
        if cur_point.x - self.center.x == self.radius &&
            cur_point.y - self.center.y == self.radius
        {
            next = match self.increase_radius_calculate_starting_point() {
                Ok(p) => p,
                Err(()) => return None,
            }
        } else {
            next = Point {
                x: cur_point.x + 1,
                ..cur_point
            };
            if self.check_for_line_wrap(&mut next).is_err() {
                return None;
            };

            while !((next.x - self.center.x).abs() == self.radius ||
                        (next.y - self.center.y).abs() == self.radius) ||
                (next.x < 0 || next.y < 0)
            {
                next.x += 1;
                if self.check_for_line_wrap(&mut next).is_err() {
                    return None;
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
                *next = self.increase_radius_calculate_starting_point()?;
            }
        }
        Ok(())
    }

    fn increase_radius_calculate_starting_point(&mut self) -> Result<Point, ()> {
        self.radius += 1;
        let mut x = self.center.x - self.radius;
        if x < 0 {
            x = self.center.x + self.radius;
            if x > self.grid_size {
                return Err(());
            }
        }
        let mut y = self.center.y - self.radius;
        if y < 0 {
            y = 0;
        }
        let p = Point {
            x,
            y,
            grid_size: self.grid_size,
        };
        println!("p {:?}", p);
        Ok(p)

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

    #[test]
    fn leave_out_indices_for_edge_cells_x_direction() {
        let mut r = RadiusIter::new(17, 6);
        assert_eq!(17, r.next().unwrap());
        assert_eq!(10, r.next().unwrap());
        assert_eq!(11, r.next().unwrap());
        assert_eq!(16, r.next().unwrap());
        assert_eq!(22, r.next().unwrap());
        assert_eq!(23, r.next().unwrap());
        assert_eq!(3, r.next().unwrap());
    }

    #[test]
    fn leave_out_indices_for_edge_cells_y_direction() {
        let mut r = RadiusIter::new(35, 6);
        assert_eq!(35, r.next().unwrap());
        assert_eq!(28, r.next().unwrap());
        assert_eq!(29, r.next().unwrap());
        assert_eq!(34, r.next().unwrap());
        assert_eq!(21, r.next().unwrap());
        assert_eq!(22, r.next().unwrap());
    }

    #[test]
    fn stop_iteration_after_radius_gets_to_big() {
        let mut r = RadiusIter::new(4, 3);
        assert_eq!(4, r.next().unwrap());
        assert_eq!(0, r.next().unwrap());
        assert_eq!(1, r.next().unwrap());
        assert_eq!(2, r.next().unwrap());
        assert_eq!(3, r.next().unwrap());
        assert_eq!(5, r.next().unwrap());
        assert_eq!(6, r.next().unwrap());
        assert_eq!(7, r.next().unwrap());
        assert_eq!(8, r.next().unwrap());
        assert_eq!(None, r.next());
    }

    #[test]
    fn hop_over_negative_cell_indices() {
        let mut r = RadiusIter::new(0, 3);
        assert_eq!(0, r.next().unwrap());
        assert_eq!(1, r.next().unwrap());
        assert_eq!(3, r.next().unwrap());
        assert_eq!(4, r.next().unwrap());
        assert_eq!(2, r.next().unwrap());
        assert_eq!(5, r.next().unwrap());
        assert_eq!(6, r.next().unwrap());
        assert_eq!(7, r.next().unwrap());
        assert_eq!(8, r.next().unwrap());
        assert_eq!(None, r.next());
    }


}
