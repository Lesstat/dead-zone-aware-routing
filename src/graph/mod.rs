mod alg;

pub use self::alg::Movement;
use grid::{Grid, NodeInfoWithIndex};
use geom::{Coord, haversine_distance};
use towers::*;

use std::time::Instant;

use rayon::prelude::*;

pub type NodeId = usize;
pub type OsmNodeId = usize;
pub type Latitude = f64;
pub type Longitude = f64;
pub type Length = f64;
pub type Speed = usize;
pub type Height = usize;

#[derive(HeapSizeOf, Default, Debug, Clone)]
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
    coverage: f64,
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
            coverage: 0.0,
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

#[derive(HeapSizeOf, Debug, PartialEq)]
pub struct HalfEdge {
    endpoint: NodeId,
    length: f64,
    time: f64,
    for_cars: bool,
    for_pedestrians: bool,
}


#[derive(Clone, PartialEq, Debug, HeapSizeOf)]
struct NodeOffset {
    out_start: usize,
}
impl NodeOffset {
    pub fn new(out_start: usize) -> NodeOffset {
        NodeOffset { out_start: out_start }
    }
}

#[derive(HeapSizeOf)]
pub struct Graph {
    pub node_info: Vec<NodeInfo>,
    node_offsets: Vec<NodeOffset>,
    pub edges: Vec<HalfEdge>,
    grid: Grid,
}

#[derive(Debug)]
pub enum RoutingGoal {
    Length,
    Speed,
}

impl Graph {
    pub fn new(mut node_info: Vec<NodeInfo>, mut edges: Vec<EdgeInfo>) -> Graph {
        let grid = Grid::new(&mut node_info, 100);
        Graph::map_edges_to_node_index(&node_info, &mut edges);
        let node_count = node_info.len();
        let (node_offsets, edges) = Graph::calc_node_offsets(node_count, edges);

        Graph {
            node_info,
            node_offsets,
            edges,
            grid,
        }

    }

    pub fn outgoing_edges_for(&self, id: NodeId) -> &[HalfEdge] {
        &self.edges[self.node_offsets[id].out_start..self.node_offsets[id + 1].out_start]
    }

    // pub fn ingoing_edges_for(&self, id: NodeId) -> &[HalfEdge] {
    //     &self.edges.in_edges[self.node_offsets[id].in_start..self.node_offsets[id + 1].in_start]
    // }

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
                    node_offset.out_start = index;
                }
                last_id = cur_id;
            }

            for node_offset in &mut node_offsets[last_id + 1..] {
                node_offset.out_start = edges.len();
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

    fn map_edges_to_node_index(nodes: &[NodeInfo], edges: &mut [EdgeInfo]) {
        use std::collections::hash_map::HashMap;
        let mut towers = load_towers("/home/flo/workspaces/rust/graphdata/o2_towers.csv")
            .expect("tower loading failed");
        let grid = Grid::new(&mut towers, 100);

        let map: HashMap<OsmNodeId, (usize, &NodeInfo)> =
            nodes.iter().enumerate().map(|n| (n.1.osm_id, n)).collect();
        let load_start = Instant::now();
        edges.par_iter_mut().for_each(|e| {
            let (source_id, source) = map[&e.source];
            let (dest_id, dest) = map[&e.dest];
            e.source = source_id;
            e.dest = dest_id;
            e.length = haversine_distance(source, dest);
            e.coverage = edge_coverage(
                source,
                dest,
                grid.adjacent_towers(source, 10.0, &towers)
                    .unwrap_or_default(),
            );
        });

        let load_end = Instant::now();

        println!(
            "preprocessed edges in: {:?}",
            load_end.duration_since(load_start)
        );
    }

    pub fn next_node_to(&self, lat: f64, long: f64) -> Option<NodeInfoWithIndex> {
        self.grid.nearest_neighbor(lat, long, &self.node_info).ok()
    }
}
#[test]
fn graph_creation() {

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
    assert_eq!(
        g.outgoing_edges_for(2),
        &[
            HalfEdge {
                endpoint: 3,
                length: 0.0,
                time: 0.0,
                for_cars: true,
                for_pedestrians: true,
            },
            HalfEdge {
                endpoint: 4,
                length: 15.718725161325155,
                time: 15.718725161325155,
                for_cars: true,
                for_pedestrians: true,
            },
        ]
    );
}
