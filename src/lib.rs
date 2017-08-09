#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate heapsize;
#[macro_use]
extern crate heapsize_derive;
extern crate osmpbfreader;
extern crate rocket;
extern crate ordered_float;
extern crate geojson;

mod graph;
mod pbf;
mod grid;
pub mod web;

pub use pbf::load_graph;
