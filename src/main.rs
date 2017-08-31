#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate dzr;

extern crate heapsize;
extern crate rocket;
extern crate clap;
use clap::{App, Arg};

use heapsize::HeapSizeOf;

fn main() {
    let matches = App::new("Dead-Zone-aware Routing")
        .author("Florian Barth <florianbarth@gmx.de>")
        .arg(
            Arg::with_name("graph-file")
                .short("f")
                .value_name("FILE")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("preprocessed")
                .short("p")
                .takes_value(false)
                .help("determines if graph-file is preprocessed"),
        )
        .arg(
            Arg::with_name("tower-file")
                .short("t")
                .value_name("file")
                .takes_value(true)
                .help("Tower file is needed for not preprocessed graphs"),
        )
        .get_matches();

    let path = matches.value_of("graph-file").expect("No Graph-file given");
    let preprocessed = matches.is_present("preprocessed");
    let g = if preprocessed {
        dzr::load_preprocessed_graph(path)
    } else {
        let tower_path = matches.value_of("tower-file").expect(
            "for pbf files a tower file is needed",
        );
        let mut towers = dzr::load_towers(tower_path).expect("Could not load towers file");
        let graph = dzr::load_graph(path, &mut towers);
        dzr::ApplicationState { graph, towers }
    };


    println!(
        "Size of graph: {} MB",
        g.heap_size_of_children() / 1_048_576
    );
    println!(
        "Size of Edges: {} MB",
        g.graph.edges.heap_size_of_children() / 1_048_576
    );

    rocket::ignite()
        .mount(
            "/",
            routes![
                dzr::web::route,
                dzr::web::next_node_to,
                dzr::web::serve_files,
                dzr::web::towers,
                dzr::web::download,
                dzr::web::map_boundary,
                dzr::web::low_coverage,
                dzr::web::redirect_to_index,
            ],
        )
        .manage(g.graph)
        .manage(g.towers)
        .launch();
}
