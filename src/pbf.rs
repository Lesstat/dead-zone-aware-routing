use osmpbfreader::{OsmObj, OsmPbfReader, Way};

use std::path::Path;
use std::fs::File;
use std::time::Instant;

use super::graph::*;

pub fn load_graph<P: AsRef<Path>>(p: P) -> Graph {
    let fs = File::open(p).unwrap();
    let mut reader = OsmPbfReader::new(fs);
    let start_loading = Instant::now();
    let obj_map = reader
        .get_objs_and_deps(|obj| obj.tags().contains_key("highway"))
        .unwrap();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (_, obj) in obj_map {
        match obj {
            OsmObj::Node(node) => {
                nodes.push(NodeInfo::new(
                    node.id.0 as usize,
                    (node.decimicro_lat as f64) / 10_000_000.0,
                    (node.decimicro_lon as f64) / 10_000_000.0,
                    0,
                ));
            }
            OsmObj::Way(w) => {
                let speed = determine_speed(&w);
                let is_one_way = is_one_way(&w);
                for (index, node) in w.nodes[0..(w.nodes.len() - 1)].iter().enumerate() {
                    edges.push(EdgeInfo::new(
                        node.0 as NodeId,
                        w.nodes[index + 1].0 as NodeId,
                        1.1, // TODO: calculate Length
                        speed,
                    ));
                    if !is_one_way {

                        edges.push(EdgeInfo::new(
                            w.nodes[index + 1].0 as NodeId,
                            node.0 as NodeId,
                            1.1, // TODO: calculate Length
                            speed,
                        ));
                    }
                }
            }
            _ => (),
        }
    }
    let start_graph = Instant::now();
    let g = Graph::new(nodes, edges);
    let end_graph = Instant::now();

    println!(
        "loading time: {:?}",
        start_graph.duration_since(start_loading)
    );
    println!("graph   time: {:?}", end_graph.duration_since(start_graph));
    g

}

fn determine_speed(way: &Way) -> Speed {
    let speed = way.tags.get("maxspeed").and_then(|s| s.parse().ok());
    if speed.is_some() {
        speed.unwrap()
    } else {
        let street_type = way.tags.get("highway").map(String::as_ref);
        match street_type {
            Some("motorway") => 130,
            Some("residential") => 50,
            _ => 100,
        }
    }
}
fn is_one_way(way: &Way) -> bool {
    let one_way = way.tags.get("oneway").and_then(|s| s.parse().ok());
    match one_way {
        Some(rule) => rule,
        None => {
            match way.tags.get("highway").map(|h| h == "motorway") {
                Some(rule) => rule,
                None => false,
            }
        }
    }
}
