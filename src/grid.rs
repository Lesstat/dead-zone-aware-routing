use super::graph::{NodeInfo, NodeId};

#[derive(Debug,HeapSizeOf)]
pub struct BoundingBox {
    lat_min: f64,
    lat_max: f64,
    long_min: f64,
    long_max: f64,
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
    pub fn contains(&self, n: &NodeInfo) -> bool {
        self.contains_point(n.lat, n.long)
    }

    pub fn contains_point(&self, lat: f64, long: f64) -> bool {
        self.lat_min <= lat && lat <= self.lat_max && self.long_min <= long && long <= self.long_max
    }


    pub fn add_node(&mut self, n: &NodeInfo) {
        if self.lat_min > n.lat {
            self.lat_min = n.lat
        }
        if self.long_min > n.long {
            self.long_min = n.long
        }
        if self.lat_max < n.lat {
            self.lat_max = n.lat
        }
        if self.long_max < n.long {
            self.long_max = n.long
        }
    }
}


#[derive(HeapSizeOf)]
pub struct Grid {
    b_box: BoundingBox,
    side_length: usize,
    offset_array: Vec<NodeId>,
}

impl Grid {
    pub fn new(nodes: &mut Vec<NodeInfo>, size: usize) -> Grid {
        let mut b_box = BoundingBox::new();

        //dereference and reborrow needed (ugly...)
        for node in &*nodes {
            b_box.add_node(node);
        }
        let mut g = Grid {
            b_box: b_box,
            side_length: size,
            offset_array: Vec::with_capacity(size * size),
        };

        nodes.sort_by_key(|n| g.coord_to_index(n.lat, n.long));
        let mut current = 0;
        g.offset_array.push(0);
        for (i, n) in nodes.iter().enumerate() {
            if g.coord_to_index(n.lat, n.long)
                   .expect("node not in grid area. Something is really wrong") !=
               current {
                current += 1;
                g.offset_array.push(i);
            }
        }
        g
    }

    pub fn coord_to_index(&self, lat: f64, long: f64) -> Result<usize, ()> {
        if !self.b_box.contains_point(lat, long) {
            return Err(());
        }
        let cell_width = (self.b_box.lat_max - self.b_box.lat_min) / self.side_length as f64;
        let cell_height = (self.b_box.long_max - self.b_box.long_min) / self.side_length as f64;
        let lat_dif = lat - self.b_box.lat_min;
        let long_dif = long - self.b_box.long_min;
        let x = (lat_dif / cell_width) as usize;
        let y = (long_dif / cell_height) as usize;

        Ok(y * self.side_length + x)

    }
}

#[test]
fn add_node_to_bounding_box() {
    let mut b = BoundingBox::new();
    let n = NodeInfo::new(1, 1.1, 1.2, 0);
    assert!(!b.contains(&n));
    b.add_node(&n);
    assert!(b.contains(&n));
}

#[test]
fn converting_coord_to_index() {
    let mut nodes = vec![NodeInfo {
                             lat: 3.4,
                             long: 5.1,
                             ..Default::default()
                         },
                         NodeInfo {
                             lat: 4.4,
                             long: 6.1,
                             ..Default::default()
                         }];
    let g = Grid::new(&mut nodes, 10);

    let index = g.coord_to_index(4.12, 5.73);
    assert_eq!(index.unwrap(), 67)
}

#[test]
fn converting_coord_to_index2() {
    let mut nodes = vec![NodeInfo {
                             lat: 3.4,
                             long: 5.1,
                             ..Default::default()
                         },
                         NodeInfo {
                             lat: 4.4,
                             long: 5.6,
                             ..Default::default()
                         }];
    let g = Grid::new(&mut nodes, 10);

    let index = g.coord_to_index(4.12, 5.38);
    assert_eq!(index.unwrap(), 57)
}
