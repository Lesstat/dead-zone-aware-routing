mod alg;

pub use self::alg::Route;
use super::grid::{Grid, NodeInfoWithIndex};


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
}

impl EdgeInfo {
    pub fn new(source: NodeId, dest: NodeId, length: Length, speed: Speed) -> EdgeInfo {
        EdgeInfo {
            source: source,
            dest: dest,
            length: length,
            speed: speed,
        }
    }
}

#[derive(HeapSizeOf, Debug, PartialEq)]
pub struct HalfEdge {
    endpoint: NodeId,
    weight: f64,
}

#[derive(HeapSizeOf)]
pub struct HalfEdges {
    out_edges: Vec<HalfEdge>,
    in_edges: Vec<HalfEdge>,
}


#[derive(Clone, PartialEq, Debug, HeapSizeOf)]
struct NodeOffset {
    in_start: usize,
    out_start: usize,
}
impl NodeOffset {
    pub fn new(in_start: usize, out_start: usize) -> NodeOffset {
        NodeOffset {
            in_start: in_start,
            out_start: out_start,
        }
    }
}

#[derive(HeapSizeOf)]
pub struct Graph {
    node_info: Vec<NodeInfo>,
    node_offsets: Vec<NodeOffset>,
    length_edges: HalfEdges,
    speed_edges: HalfEdges,
    grid: Grid,
}

enum OffsetMode {
    In,
    Out,
}

pub enum RoutingGoal {
    Length,
    Speed,
}

impl Graph {
    pub fn new(mut node_info: Vec<NodeInfo>, mut edges: Vec<EdgeInfo>) -> Graph {
        let grid = Grid::new(&mut node_info, 100);
        Graph::map_edges_to_node_index(&node_info, &mut edges);
        let node_count = node_info.len();
        let (node_offsets, length_edges, speed_edges) = Graph::calc_node_offsets(node_count, edges);

        Graph {
            node_info,
            node_offsets,
            length_edges,
            speed_edges,
            grid,
        }

    }

    pub fn outgoing_edges_for(&self, id: NodeId, goal: &RoutingGoal) -> &[HalfEdge] {
        let out_edges = match *goal {
            RoutingGoal::Length => &self.length_edges.out_edges,
            RoutingGoal::Speed => &self.speed_edges.out_edges,
        };
        &out_edges[self.node_offsets[id].out_start..self.node_offsets[id + 1].out_start]
    }

    pub fn ingoing_edges_for(&self, id: NodeId, goal: &RoutingGoal) -> &[HalfEdge] {
        let in_edges = match *goal {
            RoutingGoal::Length => &self.length_edges.in_edges,
            RoutingGoal::Speed => &self.speed_edges.in_edges,
        };
        &in_edges[self.node_offsets[id].in_start..self.node_offsets[id + 1].in_start]
    }

    fn calc_node_offsets(
        node_count: usize,
        mut edges: Vec<EdgeInfo>,
    ) -> (Vec<NodeOffset>, HalfEdges, HalfEdges) {
        use std::cmp::Ordering;

        fn calc_offset_inner(
            edges: &[EdgeInfo],
            node_offsets: &mut Vec<NodeOffset>,
            mode: &OffsetMode,
        ) {

            let mut last_id = 0;
            for (index, edge) in edges.iter().enumerate() {

                let cur_id = match *mode {
                    OffsetMode::In => edge.dest,
                    OffsetMode::Out => edge.source,
                };
                for node_offset in &mut node_offsets[last_id + 1..cur_id + 1] {
                    match *mode {
                        OffsetMode::In => {
                            node_offset.in_start = index;
                        }
                        OffsetMode::Out => {
                            node_offset.out_start = index;
                        }
                    }

                }
                last_id = cur_id;
            }

            for node_offset in &mut node_offsets[last_id + 1..] {
                match *mode {
                    OffsetMode::In => {
                        node_offset.in_start = edges.len();
                    }
                    OffsetMode::Out => {
                        node_offset.out_start = edges.len();
                    }
                }
            }
        }

        let mut node_offsets = vec![NodeOffset::new(0, 0); node_count + 1];

        edges.sort_by(|a, b| {
            let ord = a.source.cmp(&b.source);
            match ord {
                Ordering::Equal => a.dest.cmp(&b.dest),
                _ => ord,
            }
        });
        calc_offset_inner(&edges, &mut node_offsets, &OffsetMode::Out);
        let l_out_edges = Graph::create_half_edges(&edges, OffsetMode::Out, RoutingGoal::Length);
        let s_out_edges = Graph::create_half_edges(&edges, OffsetMode::Out, RoutingGoal::Speed);

        edges.sort_by(|a, b| {
            let ord = a.dest.cmp(&b.dest);
            match ord {
                Ordering::Equal => a.source.cmp(&b.source),
                _ => ord,
            }
        });
        calc_offset_inner(&edges, &mut node_offsets, &OffsetMode::In);
        let l_in_edges = Graph::create_half_edges(&edges, OffsetMode::In, RoutingGoal::Length);
        let s_in_edges = Graph::create_half_edges(&edges, OffsetMode::In, RoutingGoal::Speed);

        let length_edges = HalfEdges {
            in_edges: l_in_edges,
            out_edges: l_out_edges,
        };
        let speed_edges = HalfEdges {
            in_edges: s_in_edges,
            out_edges: s_out_edges,
        };

        (node_offsets, length_edges, speed_edges)
    }
    fn create_half_edges(edges: &[EdgeInfo], mode: OffsetMode, goal: RoutingGoal) -> Vec<HalfEdge> {
        match mode {

            OffsetMode::In => {
                edges
                    .iter()
                    .map(|e| {

                        let weight = match goal {
                            RoutingGoal::Length => e.length,
                            RoutingGoal::Speed => e.length / e.speed as f64,
                        };
                        HalfEdge {
                            endpoint: e.source,
                            weight,
                        }
                    })
                    .collect()
            }

            OffsetMode::Out => {
                edges
                    .iter()
                    .map(|e| {
                        let weight = match goal {
                            RoutingGoal::Length => e.length,
                            RoutingGoal::Speed => e.length / e.speed as f64,
                        };
                        HalfEdge {
                            endpoint: e.dest,
                            weight,
                        }

                    })
                    .collect()
            }
        }

    }

    pub fn node_count(&self) -> usize {
        self.node_offsets.len()
    }

    fn map_edges_to_node_index(nodes: &[NodeInfo], edges: &mut [EdgeInfo]) {
        use std::collections::hash_map::HashMap;
        let map: HashMap<OsmNodeId, (usize, &NodeInfo)> =
            nodes.iter().enumerate().map(|n| (n.1.osm_id, n)).collect();
        for e in edges {
            let (source_id, source) = map[&e.source];
            let (dest_id, dest) = map[&e.dest];
            e.source = source_id;
            e.dest = dest_id;
            e.length = ((source.lat - dest.lat).powi(2) + (source.long - dest.long).powi(2)).sqrt();
        }
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
        NodeOffset::new(0, 0),
        NodeOffset::new(0, 3),
        NodeOffset::new(1, 3),
        NodeOffset::new(2, 5),
        NodeOffset::new(4, 5),
        NodeOffset::new(5, 5),
    ];
    assert_eq!(g.node_offsets.len(), exp.len());
    assert_eq!(g.node_offsets, exp);

    assert_eq!(g.outgoing_edges_for(0, &RoutingGoal::Length).len(), 3);
    assert_eq!(
        g.outgoing_edges_for(2, &RoutingGoal::Length),
        &[
            HalfEdge {
                endpoint: 3,
                weight: 0.0,
            },
            HalfEdge {
                endpoint: 4,
                weight: (((2.3 - 2.4) as f64).powi(2) + ((3.3 - 3.4) as f64).powi(2)).sqrt(),
            },
        ]
    );
}
