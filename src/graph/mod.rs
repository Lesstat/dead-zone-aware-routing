mod alg;

pub use self::alg::{RoutingGoal, Movement};
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

    /// Prevent routes for cars from using this edge
    pub fn not_for_cars(&mut self) {
        self.for_cars = false;
    }
    /// Prevent routes for pedestrians from using this edge
    pub fn not_for_pedestrians(&mut self) {
        self.for_pedestrians = false;
    }
}

/// HalfEdge structs do not need both endpoints as one of them can be
/// concluded from its position in the offset array
#[derive(HeapSizeOf, Debug, PartialEq, Serialize, Deserialize)]
pub struct HalfEdge {
    pub endpoint: NodeId,
    length: f64,
    time: f64,
    for_cars: bool,
    for_pedestrians: bool,
}

impl HalfEdge {
    /// Check if this edges is available for the chosen Movement type
    #[inline]
    pub fn is_not_for(&self, movement: &Movement) -> bool {
        match *movement {
            Movement::Car => !self.for_cars,
            Movement::Foot => !self.for_pedestrians,
        }
    }

    /// Extract cost according to given routing goal
    #[inline]
    pub fn get_cost(&self, goal: &RoutingGoal) -> f64 {
        match *goal {
            RoutingGoal::Length => self.length,
            RoutingGoal::Speed => self.time,
        }
    }

    /// calculate needed time according to given routing goal
    #[inline]
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
    pub grid: Grid,
    pub coverage: Coverage,
}


impl Graph {
    pub fn new(
        mut node_info: Vec<NodeInfo>,
        mut edge_infos: Vec<EdgeInfo>,
        towers: &mut Vec<Tower>,
    ) -> Graph {
        let grid = Grid::new(&mut node_info, 100);
        Graph::rename_node_ids_and_calculate_distance(&node_info, &mut edge_infos);
        let node_count = node_info.len();
        let (node_offsets, edges) = Graph::calc_node_offsets(node_count, &mut edge_infos);
        let coverage = Graph::calculate_coverage(&node_info, &mut edge_infos, towers);

        Graph {
            node_info,
            node_offsets,
            edges,
            grid,
            coverage,
        }

    }

    /// Returns an iterator over HalfEdges going out of node with ID id.
    /// The iterator yields tuples in the form (EdgeId, HalfEdge)
    pub fn outgoing_edges_for(&self, id: NodeId) -> EdgeIter {
        EdgeIter {
            start: self.node_offsets[id].0,
            stop: self.node_offsets[id + 1].0,
            position: self.node_offsets[id].0,
            edges: &self.edges,
        }
    }

    /// Creates the offset array of HalfEdges by sorting the edges by
    /// source and target node and then iterating over all edges and
    /// updating the Offsets
    fn calc_node_offsets(
        node_count: usize,
        edges: &mut Vec<EdgeInfo>,
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

        calc_offset_inner(edges, &mut node_offsets);
        let out_edges = Graph::create_half_edges(edges);

        (node_offsets, out_edges)
    }

    fn create_half_edges(edges: &[EdgeInfo]) -> Vec<HalfEdge> {
        edges
            .par_iter()
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

    fn rename_node_ids_and_calculate_distance(nodes: &[NodeInfo], edges: &mut [EdgeInfo]) {
        use std::collections::hash_map::HashMap;

        let map: HashMap<OsmNodeId, (usize, &NodeInfo)> = nodes
            .par_iter()
            .enumerate()
            .map(|n| (n.1.osm_id, n))
            .collect();
        edges.par_iter_mut().for_each(|e| {
            let (source_id, source) = map[&e.source];
            let (dest_id, dest) = map[&e.dest];
            e.source = source_id;
            e.dest = dest_id;
            e.length = haversine_distance(source, dest);
        });

    }

    pub fn calculate_coverage(
        nodes: &[NodeInfo],
        edges: &mut Vec<EdgeInfo>,
        towers: &mut Vec<Tower>,
    ) -> Coverage {

        let grid = Grid::new(towers, 100);
        let coverage = Coverage::new(edges.len());

        edges.par_iter_mut().enumerate().for_each(|(n, e)| {
            let source = &nodes[e.source];
            let dest = &nodes[e.dest];
            let (tele, voda, o2) = edge_coverage(
                source,
                dest,
                grid.adjacent_towers(source, 15000.0, towers)
                    .unwrap_or_default(),
            );

            coverage.set(&Provider::Telekom, n, tele);
            coverage.set(&Provider::Vodafone, n, voda);
            coverage.set(&Provider::O2, n, o2);
        });

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
                length: 15718.742925384355,
                time: 15718.742925384355,
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
