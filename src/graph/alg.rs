use super::{Graph, NodeId, Length};

use std::time::Instant;
use std::cmp::Ordering;
use std::usize;

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
                queue.extend(self.outgoing_edges_for(n).iter().map(|e| e.endpoint));
            }

        }
    }

    pub fn dijkstra(&self) -> Dijkstra {
        Dijkstra {
            dist: vec![usize::MAX; self.node_count()],
            touched: Default::default(),
            graph: &self,
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

#[derive(PartialEq, Eq, Debug )]
struct NodeCost {
    node: NodeId,
    cost: usize,
}

impl Ord for NodeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for NodeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn count() {
    use super::{EdgeInfo, NodeInfo};
    let g = Graph::new(vec![NodeInfo::new(1, 2.3, 3.4, 0),
                            NodeInfo::new(2, 2.3, 3.4, 0),
                            NodeInfo::new(3, 2.3, 3.4, 0),
                            NodeInfo::new(4, 2.3, 3.4, 0),
                            NodeInfo::new(5, 2.3, 3.4, 0)],
                       vec![EdgeInfo::new(0, 1, 3, 3),
                            EdgeInfo::new(0, 2, 3, 3),
                            EdgeInfo::new(2, 3, 3, 3),
                            EdgeInfo::new(4, 0, 3, 3)]);
    assert_eq!(g.count_components(), 1)
}

pub struct Dijkstra<'a> {
    dist: Vec<Length>,
    touched: Vec<NodeId>,
    graph: &'a Graph,
}

impl<'a> Dijkstra<'a> {
    pub fn distance(&mut self, source: NodeId, dest: NodeId) -> Option<Length> {
        use std::collections::BinaryHeap;

        for node in self.touched.drain(..) {
            self.dist[node] = usize::MAX;
        }
        let mut heap = BinaryHeap::new();
        heap.push(NodeCost {
                      node: source,
                      cost: 0,
                  });

        while let Some(NodeCost { node, cost }) = heap.pop() {

            if node == dest {
                return Some(cost);
            }

            if cost > self.dist[node] {
                continue;
            }
            for edge in self.graph.outgoing_edges_for(node) {
                let next = NodeCost {
                    node: edge.endpoint,
                    cost: cost + edge.weight,
                };
                if next.cost < self.dist[next.node] {
                    self.dist[next.node] = next.cost;
                    self.touched.push(next.node);
                    heap.push(next);

                }
            }
        }
        None
    }
}
