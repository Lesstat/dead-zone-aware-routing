mod alg;


pub type NodeId = usize;
pub type OsmNodeId = usize;
pub type Latitude = f64;
pub type Longitude = f64;
pub type Length = usize;
pub type Speed = usize;
pub type Height = usize;

#[derive(HeapSizeOf)]
pub struct NodeInfo {
    pub osm_id: OsmNodeId,
    lat: Latitude,
    long: Longitude,
    height: Height,
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

#[derive(PartialEq,Debug,HeapSizeOf)]
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

#[derive(HeapSizeOf,Debug, Eq, PartialEq)]
pub struct HalfEdge {
    endpoint: NodeId,
    weight: Length,
}



#[derive(Clone,PartialEq,Debug,HeapSizeOf)]
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
    out_edges: Vec<HalfEdge>,
    in_edges: Vec<HalfEdge>,
}

enum OffsetMode {
    In,
    Out,
}
impl Graph {
    pub fn new(node_info: Vec<NodeInfo>, mut edges: Vec<EdgeInfo>) -> Graph {
        use std::cmp::Ordering;
        edges.sort_by(|a, b| {
                          let ord = a.source.cmp(&b.source);
                          match ord {
                              Ordering::Equal => a.dest.cmp(&b.dest),
                              _ => ord,
                          }
                      });

        let node_count = node_info.len();
        let (node_offset, in_edges, out_edges) = Graph::calc_node_offsets(node_count, edges);
        Graph {
            node_info: node_info,
            node_offsets: node_offset,
            out_edges: out_edges,
            in_edges: in_edges,
        }

    }

    pub fn outgoing_edges_for(&self, id: NodeId) -> &[HalfEdge] {
        &self.out_edges[self.node_offsets[id].out_start..self.node_offsets[id + 1].out_start]
    }

    pub fn ingoing_edges_for(&self, id: NodeId) -> &[HalfEdge] {
        &self.out_edges[self.node_offsets[id].in_start..self.node_offsets[id + 1].in_start]
    }

    fn calc_node_offsets(node_count: usize,
                         mut edges: Vec<EdgeInfo>)
                         -> (Vec<NodeOffset>, Vec<HalfEdge>, Vec<HalfEdge>) {
        use std::cmp::Ordering;

        fn calc_offset_inner(edges: &Vec<EdgeInfo>,
                             node_offsets: &mut Vec<NodeOffset>,
                             mode: OffsetMode) {

            let mut last_id = 0;
            for (index, edge) in edges.iter().enumerate() {

                let cur_id = match mode {
                    OffsetMode::In => edge.dest,
                    OffsetMode::Out => edge.source,
                };
                for node_offset in &mut node_offsets[last_id + 1..cur_id + 1] {
                    match mode {
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
                match mode {
                    OffsetMode::In => {
                        node_offset.in_start = edges.len();
                    }
                    OffsetMode::Out => {
                        node_offset.out_start = edges.len();
                    }
                }
            }
        }

        let mut node_offsets = vec![NodeOffset::new(0,0); node_count +1];

        edges.sort_by(|a, b| {
                          let ord = a.source.cmp(&b.source);
                          match ord {
                              Ordering::Equal => a.dest.cmp(&b.dest),
                              _ => ord,
                          }
                      });
        calc_offset_inner(&edges, &mut node_offsets, OffsetMode::Out);
        let out_edges = Graph::create_half_edges(&edges, OffsetMode::Out);

        edges.sort_by(|a, b| {
                          let ord = a.dest.cmp(&b.dest);
                          match ord {
                              Ordering::Equal => a.source.cmp(&b.source),
                              _ => ord,
                          }
                      });
        calc_offset_inner(&edges, &mut node_offsets, OffsetMode::In);
        let in_edges = Graph::create_half_edges(&edges, OffsetMode::In);


        (node_offsets, in_edges, out_edges)
    }
    fn create_half_edges(edges: &Vec<EdgeInfo>, mode: OffsetMode) -> Vec<HalfEdge> {
        match mode {

            OffsetMode::In => {
                edges
                    .iter()
                    .map(|e| {
                             HalfEdge {
                                 endpoint: e.source,
                                 weight: e.length,
                             }
                         })
                    .collect()
            }

            OffsetMode::Out => {
                edges
                    .iter()
                    .map(|e| {
                             HalfEdge {
                                 endpoint: e.dest,
                                 weight: e.length,
                             }
                         })
                    .collect()
            }
        }

    }

    pub fn node_count(&self) -> usize {
        self.node_offsets.len()
    }
}

#[test]
fn graph_creation() {
    let g = Graph::new(vec![NodeInfo::new(23, 3.4, 2.3, 12),
                            NodeInfo::new(27, 4.4, 2.3, 12),
                            NodeInfo::new(53, 6.4, 1.3, 12),
                            NodeInfo::new(36, 3.8, 2.4, 12),
                            NodeInfo::new(78, 9.2, 2.3, 12)],
                       vec![EdgeInfo::new(0, 1, 1, 1),
                            EdgeInfo::new(0, 2, 1, 1),
                            EdgeInfo::new(2, 3, 1, 1),
                            EdgeInfo::new(0, 3, 1, 1),
                            EdgeInfo::new(2, 4, 1, 1)]);
    let exp = vec![NodeOffset::new(0, 0),
                   NodeOffset::new(0, 3),
                   NodeOffset::new(1, 3),
                   NodeOffset::new(2, 5),
                   NodeOffset::new(4, 5),
                   NodeOffset::new(5, 5)];
    assert_eq!(g.node_offsets.len(), exp.len());
    assert_eq!(g.node_offsets, exp);

    assert_eq!(g.outgoing_edges_for(0).len(), 3);
    assert_eq!(g.outgoing_edges_for(2),
               &[HalfEdge {
                     endpoint: 3,
                     weight: 1,
                 },
                 HalfEdge {
                     endpoint: 4,
                     weight: 1,
                 }]);
}
