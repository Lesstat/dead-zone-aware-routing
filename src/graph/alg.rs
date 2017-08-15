use super::{Graph, NodeId, Length, RoutingGoal};
use towers::Provider;

use std::cmp::Ordering;
use std::f64;
use std::collections::VecDeque;

use ordered_float::OrderedFloat;

impl Graph {
    pub fn dijkstra(&self) -> Dijkstra {
        Dijkstra {
            dist: vec![f64::MAX.into(); self.node_count()],
            touched: Default::default(),
            graph: self,
        }
    }
}


#[derive(PartialEq, Eq, Debug)]
struct NodeCost {
    node: NodeId,
    cost: OrderedFloat<f64>,
    time: OrderedFloat<f64>,
    distance: OrderedFloat<f64>,
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
        provider: Option<Provider>,
    ) -> Option<Route> {
        use std::collections::BinaryHeap;
        let goal = match movement {
            Movement::Car => goal,
            Movement::Foot => RoutingGoal::Length,
        };
        let coverage = match provider {
            Some(provider) => Some(self.graph.coverage.get_all(&provider)),
            None => None,
        };

        let mut prev: Vec<usize> = (0..self.graph.node_count()).collect();

        for node in self.touched.drain(..) {
            self.dist[node] = OrderedFloat::from(f64::MAX);
        }
        let mut heap = BinaryHeap::new();
        heap.push(NodeCost {
            node: source,
            cost: 0.0.into(),
            time: 0.0.into(),
            distance: 0.0.into(),
        });

        while let Some(NodeCost {
                           node,
                           cost,
                           time,
                           distance,
                       }) = heap.pop()
        {

            if node == dest {
                let mut path = VecDeque::new();
                let mut cur = node;
                while cur != source {
                    path.push_front(cur);
                    cur = prev[cur];
                }
                path.push_front(cur);
                return Some(Route {
                    node_seq: path,
                    distance: distance.into_inner(),
                    travel_time: time.into_inner(),
                });
            }

            if cost > self.dist[node] {
                continue;
            }
            for (n, edge) in self.graph.outgoing_edges_for(node) {
                if edge.is_not_for(&movement) {
                    continue;
                }
                let scaling_factor = match coverage {
                    Some(cov) => (1.0 + f64::EPSILON) / (3.0 * cov[n] + f64::EPSILON),
                    None => 1.0,
                };
                let next = match goal {
                    RoutingGoal::Length => {
                        let time_calc = match movement {
                            Movement::Car => edge.time,
                            Movement::Foot => edge.length / 3.0,
                        };
                        NodeCost {
                            node: edge.endpoint,
                            cost: (cost.into_inner() + edge.length * scaling_factor).into(),
                            time: (time.into_inner() + time_calc).into(),
                            distance: (distance.into_inner() + edge.length).into(),
                        }
                    }
                    RoutingGoal::Speed => NodeCost {
                        node: edge.endpoint,
                        cost: (cost.into_inner() + edge.time * scaling_factor).into(),
                        time: (time.into_inner() + edge.time).into(),
                        distance: (distance.into_inner() + edge.length).into(),
                    },
                };
                //assert!(next.cost >= next.distance);
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
