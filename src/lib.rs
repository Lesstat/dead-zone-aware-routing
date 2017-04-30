extern crate heapsize;
#[macro_use]
extern crate heapsize_derive;
extern crate osmpbfreader;

mod graph;
mod pbf;

pub use pbf::load_graph;
