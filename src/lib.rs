#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate heapsize;
#[macro_use]
extern crate heapsize_derive;
extern crate osmpbfreader;
extern crate rocket;

mod graph;
mod pbf;
mod grid;
pub mod web;

pub use pbf::load_graph;
