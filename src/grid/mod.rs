use graph::NodeInfo;
use geom::{Coord, haversine_distance};
use towers::Tower;


mod radius;

#[derive(Debug, HeapSizeOf, FromForm, Serialize, Deserialize)]
pub struct BoundingBox {
    pub lat_min: f64,
    pub lat_max: f64,
    pub long_min: f64,
    pub long_max: f64,
}
impl BoundingBox {
    pub fn new() -> BoundingBox {
        use std::f64;
        BoundingBox {
            lat_min: f64::MAX,
            lat_max: f64::MIN,
            long_min: f64::MAX,
            long_max: f64::MIN,
        }
    }
    pub fn contains_point(&self, lat: f64, long: f64) -> bool {
        self.lat_min <= lat && lat <= self.lat_max && self.long_min <= long && long <= self.long_max
    }


    pub fn add_coord<C: Coord>(&mut self, n: &C) {
        if self.lat_min > n.lat() {
            self.lat_min = n.lat()
        }
        if self.long_min > n.lon() {
            self.long_min = n.lon()
        }
        if self.lat_max < n.lat() {
            self.lat_max = n.lat()
        }
        if self.long_max < n.lon() {
            self.long_max = n.lon()
        }
    }
}


#[derive(HeapSizeOf, Debug, Serialize, Deserialize)]
pub struct Grid {
    pub b_box: BoundingBox,
    side_length: usize,
    offset_array: Vec<usize>,
}

impl Grid {
    /// Creates Grid of size `size` with the data in `coords`. The
    /// values in `coords` will be sorted by their cell index inside
    /// the grid.
    pub fn new<C: Coord>(coords: &mut Vec<C>, size: usize) -> Grid {
        let mut b_box = BoundingBox::new();

        //dereference and reborrow needed (ugly...)
        for coord in &*coords {
            b_box.add_coord(coord);
        }
        let mut g = Grid {
            b_box: b_box,
            side_length: size,
            offset_array: vec![0; size * size + 1],
        };

        coords.sort_by_key(|n| g.coord_to_index(n.lat(), n.lon()));
        let mut current = 0;
        for (i, n) in coords.iter().enumerate() {
            let new_index = g.coord_to_index(n.lat(), n.lon()).expect(
                "Coordinate not in grid area. Something is really wrong",
            );
            if new_index != current {
                for offset in &mut g.offset_array[current + 1..new_index + 1] {
                    *offset = i;
                }
                current = new_index;
            }
        }
        for offset in &mut g.offset_array[current + 1..] {
            *offset = coords.len();
        }
        let last_offset = g.offset_array.len() - 1;
        g.offset_array[last_offset] = coords.len();

        g
    }

    /// Converts coordinates to a index inside the grid.
    /// Returns error if the coordinates are not inside the grid
    pub fn coord_to_index(&self, lat: f64, long: f64) -> Result<usize, ()> {
        if !self.b_box.contains_point(lat, long) {
            return Err(());
        }
        let cell_width = (self.b_box.lat_max - self.b_box.lat_min) / self.side_length as f64;
        let cell_height = (self.b_box.long_max - self.b_box.long_min) / self.side_length as f64;
        let lat_dif = lat - self.b_box.lat_min;
        let long_dif = long - self.b_box.long_min;
        let mut x = (lat_dif / cell_width) as usize;
        let mut y = (long_dif / cell_height) as usize;
        if x == self.side_length {
            x -= 1;
        }
        if y == self.side_length {
            y -= 1;
        }
        Ok(y * (self.side_length) + x)

    }


    pub fn nearest_neighbor<'a>(
        &self,
        lat: f64,
        long: f64,
        nodes: &'a [NodeInfo],
    ) -> Result<NodeInfoWithIndex, ()> {
        use std::{f64, usize};

        let cell_width = haversine_distance(&(self.b_box.lat_max, self.b_box.long_max), &(
            self.b_box
                .lat_min,
            self.b_box
                .long_max,
        )) / self.side_length as f64;
        let cell_height = haversine_distance(&(self.b_box.lat_max, self.b_box.long_max), &(
            self.b_box
                .lat_max,
            self.b_box
                .long_min,
        )) / self.side_length as f64;
        let cell_measure = cell_width.min(cell_height);
        let mut radius = 0;

        let index = self.coord_to_index(lat, long)?;
        let mut min_dist = f64::INFINITY;
        let mut min_index = usize::MAX;
        loop {
            let max_min_dist = (radius as f64 - 1.0) * cell_measure;
            if max_min_dist > min_dist {
                break;
            }
            let radius_iter =
                radius::RadiusIter::new(index as isize, self.side_length as isize, radius);
            radius += 1;
            for index in radius_iter {
                let start = self.offset_array[index];
                let end = self.offset_array[index + 1];

                for (i, n) in nodes[start..end].iter().enumerate() {
                    let dist = haversine_distance(&(lat, long), n);
                    if dist < min_dist {
                        min_dist = dist;
                        min_index = start + i;
                    }
                }

            }
        }
        if min_index < usize::MAX {
            Ok(NodeInfoWithIndex(min_index, nodes[min_index].clone()))
        } else {
            Err(())
        }
    }

    pub fn adjacent_towers<'a, C: Coord>(
        &self,
        coords: &C,
        max_dist: f64,
        towers: &'a [Tower],
    ) -> Result<Vec<::std::slice::Iter<'a, Tower>>, ()> {
        let cell_width = haversine_distance(&(self.b_box.lat_max, self.b_box.long_max), &(
            self.b_box
                .lat_min,
            self.b_box
                .long_max,
        )) / self.side_length as f64;
        let cell_height = haversine_distance(&(self.b_box.lat_max, self.b_box.long_max), &(
            self.b_box
                .lat_max,
            self.b_box
                .long_min,
        )) / self.side_length as f64;
        let cell_measure = cell_width.min(cell_height);
        let mut radius = 0;

        let index = self.coord_to_index(coords.lat(), coords.lon())?;
        let mut result = Vec::new();
        loop {
            let max_min_dist = (radius as f64 - 1.0) * cell_measure;
            if max_min_dist > max_dist {
                break;
            }
            let radius_iter =
                radius::RadiusIter::new(index as isize, self.side_length as isize, radius);
            radius += 1;
            for index in radius_iter {
                let start = self.offset_array[index];
                let end = self.offset_array[index + 1];
                let iter = towers[start..end].iter();
                result.push(iter);
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct NodeInfoWithIndex(pub usize, pub NodeInfo);


#[test]
fn converting_coord_to_index() {
    let mut nodes = vec![
        NodeInfo {
            lat: 3.4,
            long: 5.1,
            ..Default::default()
        },
        NodeInfo {
            lat: 4.4,
            long: 6.1,
            ..Default::default()
        },
    ];
    let g = Grid::new(&mut nodes, 10);

    let index = g.coord_to_index(4.12, 5.73);
    assert_eq!(index.unwrap(), 67)
}

#[test]
fn converting_coord_to_index2() {
    let mut nodes = vec![
        NodeInfo {
            lat: 3.4,
            long: 5.1,
            ..Default::default()
        },
        NodeInfo {
            lat: 4.4,
            long: 5.6,
            ..Default::default()
        },
    ];
    let g = Grid::new(&mut nodes, 10);

    let index = g.coord_to_index(4.12, 5.38);
    assert_eq!(index.unwrap(), 57)
}

#[test]
fn converting_coord_to_index_edge_points() {
    let mut nodes = vec![
        NodeInfo {
            lat: 3.4,
            long: 5.1,
            ..Default::default()
        },
        NodeInfo {
            lat: 4.4,
            long: 5.6,
            ..Default::default()
        },
    ];
    let g = Grid::new(&mut nodes, 10);

    let index = g.coord_to_index(4.4, 5.6);
    assert_eq!(index.unwrap(), 99)
}

#[test]
fn nearest_neighbor_2_points() {
    let mut nodes = vec![
        NodeInfo::new(0, 10.2, 30.4, 0),
        NodeInfo::new(1, 20.5, 40.1, 0),
    ];
    let g = Grid::new(&mut nodes, 10);
    let n = g.nearest_neighbor(10.3, 30.5, &nodes).unwrap();
    assert_eq!(0, n.0);
}

#[test]
fn nearest_neighbor_2_points_other_point() {
    let mut nodes = vec![
        NodeInfo::new(0, 10.2, 30.4, 0),
        NodeInfo::new(1, 20.5, 40.1, 0),
    ];
    let g = Grid::new(&mut nodes, 10);
    let n = g.nearest_neighbor(20.5, 40.1, &nodes).unwrap();
    assert_eq!(1, n.0);
}

#[test]
fn nearest_neighbor_2_points_different_cell() {
    let mut nodes = vec![
        NodeInfo::new(0, 10.2, 30.4, 0),
        NodeInfo::new(1, 20.5, 40.1, 0),
    ];
    let g = Grid::new(&mut nodes, 10);
    let n = g.nearest_neighbor(19.0, 38.0, &nodes).unwrap();
    assert_eq!(1, n.0);
}
