#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate fapra;

extern crate heapsize;
extern crate rocket;

use heapsize::HeapSizeOf;


fn main() {
    let g = fapra::load_graph("/home/flo/workspaces/rust/graphdata/stuttgart-regbez-latest.osm.pbf");
    println!("Size of graph: {} MB",
             g.heap_size_of_children() / 1048576);


    rocket::ignite().mount("/", routes![fapra::web::route]).manage(g).launch();
}
