use super::{Graph, NodeId, Length, RoutingGoal};

use std::time::Instant;
use std::cmp::Ordering;
use std::f64;
use std::collections::VecDeque;

use ordered_float::OrderedFloat;

impl Graph {
    pub fn count_components(&self) -> usize {
        let start = Instant::now();
        let mut union = UnionFind::new(self.node_info.len());
        for id in 0..self.node_info.len() {
            if id == union.find(id) {
                self.dfs(id, &mut union);
            }
        }
        let count = union.count();
        let end = Instant::now();
        println!("Counting Components took {:?}", end.duration_since(start));
        count
    }

    fn dfs(&self, start: NodeId, union: &mut UnionFind) {
        let mut queue = Vec::<NodeId>::new();
        queue.push(start);
        while let Some(n) = queue.pop() {
            if union.find(n) == n {
                union.union(start, n);
                queue.extend(
                    self.outgoing_edges_for(n, &RoutingGoal::Length)
                        .iter()
                        .map(|e| e.endpoint),
                );
            }

        }
    }

    pub fn dijkstra(&self) -> Dijkstra {
        Dijkstra {
            dist: vec![f64::MAX.into(); self.node_count()],
            touched: Default::default(),
            graph: self,
        }
    }
}


#[derive(Debug)]
struct UnionFind {
    parent: Vec<NodeId>,
}

impl UnionFind {
    pub fn new(size: usize) -> UnionFind {
        let parent = (0..size).collect();
        UnionFind { parent: parent }
    }

    pub fn find(&mut self, id: NodeId) -> NodeId {
        let mut visited_ids = vec![id];
        let mut cur_id = id;
        let mut par_id = self.parent[id];
        while cur_id != par_id {
            cur_id = par_id;
            visited_ids.push(cur_id);
            par_id = self.parent[cur_id];
        }
        for id in visited_ids {
            self.parent[id] = par_id;
        }
        par_id
    }

    pub fn union(&mut self, r: NodeId, s: NodeId) {
        let r_par = self.find(r);
        let s_par = self.find(s);
        if r_par != s_par {
            self.parent[s_par] = r_par;
        }
    }

    fn count(&self) -> usize {
        let mut result = 0;
        for (index, &par) in self.parent.iter().enumerate() {
            if index == par {
                result += 1;
            }
        }
        result
    }
}

#[derive(PartialEq, Eq, Debug)]
struct NodeCost {
    node: NodeId,
    cost: OrderedFloat<f64>,
}

impl Ord for NodeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for NodeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}

#[test]
fn count() {
    use super::{EdgeInfo, NodeInfo};
    let g = Graph::new(
        vec![
            NodeInfo::new(0, 2.2, 3.2, 0),
            NodeInfo::new(1, 2.3, 3.4, 0),
            NodeInfo::new(2, 2.3, 3.4, 0),
            NodeInfo::new(3, 2.3, 3.4, 0),
            NodeInfo::new(4, 2.4, 3.9, 0),
        ],
        vec![
            EdgeInfo::new(0, 1, 3.0, 3),
            EdgeInfo::new(0, 2, 3.0, 3),
            EdgeInfo::new(2, 3, 3.0, 3),
            EdgeInfo::new(4, 0, 3.0, 3),
        ],
    );
    assert_eq!(g.count_components(), 1)
}

pub struct Dijkstra<'a> {
    dist: Vec<OrderedFloat<f64>>,
    touched: Vec<NodeId>,
    graph: &'a Graph,
}

pub struct Route {
    pub distance: Length,
    pub travel_time: f64,
    pub node_seq: NodeSequence,
}

impl<'a> Dijkstra<'a> {
    pub fn distance(
        &mut self,
        source: NodeId,
        dest: NodeId,
        goal: RoutingGoal,
        movement: Movement,
    ) -> Option<Route> {
        use std::collections::BinaryHeap;
        let goal = match movement {
            Movement::Car => goal,
            Movement::Foot => RoutingGoal::Length,
        };


        let mut prev: Vec<usize> = (0..self.graph.node_count()).collect();

        for node in self.touched.drain(..) {
            self.dist[node] = OrderedFloat::from(f64::MAX);
        }
        let mut heap = BinaryHeap::new();
        heap.push(NodeCost {
            node: source,
            cost: 0.0.into(),
        });

        while let Some(NodeCost { node, cost }) = heap.pop() {

            if node == dest {
                let mut path = VecDeque::new();
                let mut cur = node;
                while cur != source {
                    path.push_front(cur);
                    cur = prev[cur];
                }
                path.push_front(cur);
                let b = RouteBuilder::new(path, self.graph);
                let r = b.build(&movement);
                return Some(r);
            }

            if cost > self.dist[node] {
                continue;
            }
            for edge in self.graph.outgoing_edges_for(node, &goal) {
                match movement {
                    Movement::Car => {
                        if !edge.for_cars {
                            continue;
                        }
                    }
                    Movement::Foot => {
                        if !edge.for_pedestrians {
                            continue;
                        }
                    }
                }

                let next = NodeCost {
                    node: edge.endpoint,
                    cost: (cost.into_inner() + edge.weight).into(),
                };
                if next.cost < self.dist[next.node] {
                    prev[next.node] = node;
                    self.dist[next.node] = next.cost;
                    self.touched.push(next.node);
                    heap.push(next);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum Movement {
    Car,
    Foot,
}

type NodeSequence = VecDeque<usize>;
struct RouteBuilder<'a> {
    distance: Option<Length>,
    travel_time: Option<f64>,
    node_seq: NodeSequence,
    graph: &'a Graph,
}

impl<'a> RouteBuilder<'a> {
    pub fn new(node_seq: NodeSequence, graph: &'a Graph) -> RouteBuilder<'a> {
        RouteBuilder {
            distance: None,
            travel_time: None,
            node_seq,
            graph,
        }
    }

    pub fn build(self, movement: &Movement) -> Route {
        let distance = match self.distance {
            Some(d) => d,
            None => self.calc_dist(),
        };
        let travel_time = match self.travel_time {
            Some(t) => t,
            None => self.calc_travel_time(movement),
        };
        Route {
            distance,
            travel_time,
            node_seq: self.node_seq,
        }
    }

    fn calc_dist(&self) -> Length {
        self.calc(&RoutingGoal::Length, &Movement::Car)
    }
    fn calc_travel_time(&self, movement: &Movement) -> f64 {
        self.calc(&RoutingGoal::Speed, movement)

    }
    fn calc(&self, goal: &RoutingGoal, movement: &Movement) -> f64 {
        let length = RoutingGoal::Length;
        let inner_goal = match *movement {
            Movement::Foot => &length,
            Movement::Car => goal,
        };

        let mut result = 0.0;
        let (node_slice, _) = self.node_seq.as_slices();

        for nodes in node_slice.windows(2) {
            let node1 = nodes[0];
            let node2 = nodes[1];
            for edge in self.graph.outgoing_edges_for(node1, inner_goal) {
                if edge.endpoint == node2 {
                    let mut update = false;
                    if let Movement::Foot = *movement {
                        if let RoutingGoal::Speed = *goal {
                            result += edge.weight / 3.0;
                            update = true;
                        }
                    }
                    if !update {
                        result += edge.weight;
                    }

                    break;
                }
            }
        }
        result

    }
}
