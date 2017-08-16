mod alg;

pub use self::alg::Movement;
use grid::{Grid, NodeInfoWithIndex};
use geom::{Coord, haversine_distance};
use towers::*;

use std::time::Instant;
use std::path::Path;

use rayon::prelude::*;
use bincode;

pub type NodeId = usize;
pub type OsmNodeId = usize;
pub type Latitude = f64;
pub type Longitude = f64;
pub type Length = f64;
pub type Speed = usize;
pub type Height = usize;

#[derive(HeapSizeOf, Default, Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub osm_id: OsmNodeId,
    pub lat: Latitude,
    pub long: Longitude,
    pub height: Height,
}

impl NodeInfo {
    pub fn new(osm_id: OsmNodeId, lat: Latitude, long: Longitude, height: Height) -> NodeInfo {
        NodeInfo {
            osm_id: osm_id,
            lat: lat,
            long: long,
            height: height,
        }
    }
}

impl PartialEq for NodeInfo {
    fn eq(&self, other: &NodeInfo) -> bool {
        self.osm_id == other.osm_id
    }
}
impl Eq for NodeInfo {}

#[derive(PartialEq, Debug, HeapSizeOf)]
pub struct EdgeInfo {
    pub source: NodeId,
    pub dest: NodeId,
    length: Length,
    speed: Speed,
    for_cars: bool,
    for_pedestrians: bool,
}

impl Coord for NodeInfo {
    #[inline]
    fn lat(&self) -> f64 {
        self.lat
    }
    #[inline]
    fn lon(&self) -> f64 {
        self.long
    }
}

impl EdgeInfo {
    pub fn new(source: NodeId, dest: NodeId, length: Length, speed: Speed) -> EdgeInfo {
        EdgeInfo {
            source: source,
            dest: dest,
            length: length,
            speed: speed,
            for_cars: true,
            for_pedestrians: true,
        }
    }

    pub fn not_for_cars(&mut self) {
        self.for_cars = false;
    }
    pub fn not_for_pedestrians(&mut self) {
        self.for_pedestrians = false;
    }
}

#[derive(HeapSizeOf, Debug, PartialEq, Serialize, Deserialize)]
pub struct HalfEdge {
    endpoint: NodeId,
    length: f64,
    time: f64,
    for_cars: bool,
    for_pedestrians: bool,
}

impl HalfEdge {
    #[inline]
    pub fn is_not_for(&self, movement: &Movement) -> bool {
        match *movement {
            Movement::Car => !self.for_cars,
            Movement::Foot => !self.for_pedestrians,
        }
    }

    #[inline]
    pub fn get_cost(&self, goal: &RoutingGoal) -> f64 {
        match *goal {
            RoutingGoal::Length => self.length,
            RoutingGoal::Speed => self.time,
        }
    }

    pub fn get_time(&self, movement: &Movement) -> f64 {
        match *movement {
            Movement::Car => self.time,
            Movement::Foot => self.length / 3.0,
        }
    }
}


#[derive(Clone, PartialEq, Debug, HeapSizeOf, Serialize, Deserialize)]
struct NodeOffset(usize);
impl NodeOffset {
    pub fn new(out_start: usize) -> NodeOffset {
        NodeOffset(out_start)
    }
}

#[derive(HeapSizeOf, Serialize, Deserialize)]
pub struct Graph {
    pub node_info: Vec<NodeInfo>,
    node_offsets: Vec<NodeOffset>,
    pub edges: Vec<HalfEdge>,
    grid: Grid,
    coverage: Coverage,
}

#[derive(Debug)]
pub enum RoutingGoal {
    Length,
    Speed,
}

impl Graph {
    pub fn new(
        mut node_info: Vec<NodeInfo>,
        mut edges: Vec<EdgeInfo>,
        towers: &mut Vec<Tower>,
    ) -> Graph {
        let grid = Grid::new(&mut node_info, 100);
        let coverage = Graph::preprocess_edges(&node_info, &mut edges, towers);
        let node_count = node_info.len();
        let (node_offsets, edges) = Graph::calc_node_offsets(node_count, edges);

        Graph {
            node_info,
            node_offsets,
            edges,
            grid,
            coverage,
        }

    }

    pub fn outgoing_edges_for(&self, id: NodeId) -> EdgeIter {
        EdgeIter {
            start: self.node_offsets[id].0,
            stop: self.node_offsets[id + 1].0,
            position: self.node_offsets[id].0,
            edges: &self.edges,
        }
    }

    fn calc_node_offsets(
        node_count: usize,
        mut edges: Vec<EdgeInfo>,
    ) -> (Vec<NodeOffset>, Vec<HalfEdge>) {
        use std::cmp::Ordering;

        fn calc_offset_inner(edges: &[EdgeInfo], node_offsets: &mut Vec<NodeOffset>) {

            let mut last_id = 0;
            for (index, edge) in edges.iter().enumerate() {
                let cur_id = edge.source;
                for node_offset in &mut node_offsets[last_id + 1..cur_id + 1] {
                    node_offset.0 = index;
                }
                last_id = cur_id;
            }

            for node_offset in &mut node_offsets[last_id + 1..] {
                node_offset.0 = edges.len();
            }
        }

        let mut node_offsets = vec![NodeOffset::new(0); node_count + 1];

        edges.sort_by(|a, b| {
            let ord = a.source.cmp(&b.source);
            match ord {
                Ordering::Equal => a.dest.cmp(&b.dest),
                _ => ord,
            }
        });
        edges.dedup_by_key(|edge| (edge.source, edge.dest));

        calc_offset_inner(&edges, &mut node_offsets);
        let out_edges = Graph::create_half_edges(&edges);

        (node_offsets, out_edges)
    }
    fn create_half_edges(edges: &[EdgeInfo]) -> Vec<HalfEdge> {
        edges
            .iter()
            .map(|e| {
                HalfEdge {
                    endpoint: e.dest,
                    length: e.length,
                    time: e.length / e.speed as f64,
                    for_cars: e.for_cars,
                    for_pedestrians: e.for_pedestrians,
                }
            })
            .collect()
    }

    pub fn node_count(&self) -> usize {
        self.node_offsets.len()
    }

    fn preprocess_edges(
        nodes: &[NodeInfo],
        edges: &mut [EdgeInfo],
        towers: &mut Vec<Tower>,
    ) -> Coverage {
        use std::collections::hash_map::HashMap;
        let grid = Grid::new(towers, 100);
        let coverage = Coverage::new(edges.len());

        let map: HashMap<OsmNodeId, (usize, &NodeInfo)> =
            nodes.iter().enumerate().map(|n| (n.1.osm_id, n)).collect();
        println!("processing coverage");
        let load_start = Instant::now();
        edges.par_iter_mut().enumerate().for_each(|(n, e)| {
            let (source_id, source) = map[&e.source];
            let (dest_id, dest) = map[&e.dest];
            e.source = source_id;
            e.dest = dest_id;
            e.length = haversine_distance(source, dest);
            let (tele, voda, o2) = edge_coverage(
                source,
                dest,
                grid.adjacent_towers(source, 10.0 + e.length, towers)
                    .unwrap_or_default(),
            );
            coverage.set(&Provider::Telekom, n, tele);
            coverage.set(&Provider::Vodafone, n, voda);
            coverage.set(&Provider::O2, n, o2);
        });
        let load_end = Instant::now();

        println!(
            "preprocessed edges in: {:?}",
            load_end.duration_since(load_start)
        );
        coverage
    }

    pub fn next_node_to(&self, lat: f64, long: f64) -> Option<NodeInfoWithIndex> {
        self.grid.nearest_neighbor(lat, long, &self.node_info).ok()
    }
}
#[test]
fn graph_creation() {

    let mut towers = Vec::new();
    let g = Graph::new(
        vec![
            NodeInfo::new(23, 2.3, 3.3, 12),
            NodeInfo::new(27, 2.3, 3.3, 12),
            NodeInfo::new(53, 2.3, 3.3, 12),
            NodeInfo::new(36, 2.3, 3.3, 12),
            NodeInfo::new(78, 2.4, 3.4, 12),
        ],
        vec![
            EdgeInfo::new(23, 27, 1.0, 1),
            EdgeInfo::new(23, 53, 1.0, 1),
            EdgeInfo::new(53, 36, 1.0, 1),
            EdgeInfo::new(23, 36, 1.0, 1),
            EdgeInfo::new(53, 78, 1.0, 1),
        ],
        &mut towers,
    );
    let exp = vec![
        NodeOffset::new(0),
        NodeOffset::new(3),
        NodeOffset::new(3),
        NodeOffset::new(5),
        NodeOffset::new(5),
        NodeOffset::new(5),
    ];
    assert_eq!(g.node_offsets.len(), exp.len());
    assert_eq!(g.node_offsets, exp);

    assert_eq!(g.outgoing_edges_for(0).len(), 3);
    let mut iter = g.outgoing_edges_for(2);
    assert_eq!(
        Some((
            3,
            &HalfEdge {
                endpoint: 3,
                length: 0.0,
                time: 0.0,
                for_cars: true,
                for_pedestrians: true,
            },
        )),
        iter.next()
    );
    assert_eq!(
        Some((
            4,
            &HalfEdge {
                endpoint: 4,
                length: 15.718725161325155,
                time: 15.718725161325155,
                for_cars: true,
                for_pedestrians: true,
            },
        )),
        iter.next()
    );


}

#[derive(Debug)]
pub struct EdgeIter<'a> {
    start: usize,
    stop: usize,
    position: usize,
    edges: &'a Vec<HalfEdge>,
}

impl<'a> Iterator for EdgeIter<'a> {
    type Item = (usize, &'a HalfEdge);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.stop {
            None
        } else {
            let item = (self.position, &self.edges[self.position]);
            self.position += 1;
            Some(item)
        }
    }
}
impl<'a> EdgeIter<'a> {
    pub fn len(&self) -> usize {
        self.stop - self.start
    }
}

pub fn load_preprocessed_graph<P: AsRef<Path>>(path: P) -> super::ApplicationState {
    use std::fs::File;
    use std::io::BufReader;

    let start = Instant::now();
    let mut reader = BufReader::new(File::open(path).expect(
        "Preprocessed Graph file could not be opened",
    ));
    let g = bincode::deserialize_from(&mut reader, bincode::Infinite)
        .expect("Could not deserialize preprocessed graph");
    let end = Instant::now();
    println!(
        "loaded preprocessed graph in {:?}",
        end.duration_since(start)
    );
    g
}
