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
        let coverage = self.graph.coverage.get_all(provider);

        let mut prev: Vec<usize> = (0..self.graph.node_count()).collect();

        self.reset_state();
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
                let scaling_factor = self.calculate_scaling_factor(coverage, n);
                let next = NodeCost {
                    node: edge.endpoint,
                    cost: (cost.into_inner() + edge.get_cost(&goal) * scaling_factor).into(),
                    time: (time.into_inner() + edge.get_time(&movement)).into(),
                    distance: (distance.into_inner() + edge.length).into(),
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

    #[inline]
    fn reset_state(&mut self) {
        for node in self.touched.drain(..) {
            self.dist[node] = OrderedFloat::from(f64::MAX);
        }

    }
    #[inline]
    fn calculate_scaling_factor(&self, coverage: Option<&Vec<f64>>, index: usize) -> f64 {
        match coverage {
            Some(cov) => (1.0 + f64::EPSILON) / (3.0 * cov[index] + f64::EPSILON),
            None => 1.0,
        }
    }
}

#[derive(Debug)]
pub enum Movement {
    Car,
    Foot,
}

type NodeSequence = VecDeque<usize>;
