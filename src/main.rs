#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate fapra;

extern crate heapsize;
extern crate rocket;

use heapsize::HeapSizeOf;

fn main() {
    let path = "/home/flo/workspaces/rust/graphdata/baden-wuerttemberg-latest.osm.pbf";
    let g = fapra::load_graph(path);
    println!("Size of graph: {} MB", g.heap_size_of_children() / 1048576);

    rocket::ignite()
        .mount(
            "/",
            routes![
                fapra::web::route,
                fapra::web::next_node_to,
                fapra::web::serve_files,
                fapra::web::towers,
            ],
        )
        .manage(g)
        .launch();
}
