#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#![allow(unknown_lints)]

extern crate heapsize;
#[macro_use]
extern crate heapsize_derive;
extern crate osmpbfreader;
extern crate rocket;
extern crate ordered_float;
extern crate geojson;
extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate rayon;
extern crate bincode;

mod graph;
mod pbf;
mod grid;
mod geom;
mod towers;
pub mod web;
pub use pbf::load_graph;
pub use graph::load_preprocessed_graph;
pub use towers::load_towers;


#[derive(Deserialize, HeapSizeOf)]
pub struct ApplicationState {
    pub graph: graph::Graph,
    pub towers: Vec<towers::Tower>,
}
