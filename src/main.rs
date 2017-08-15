#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate fapra;

extern crate heapsize;
extern crate rocket;
extern crate clap;
use clap::{App, Arg};

use heapsize::HeapSizeOf;

fn main() {
    let matches = App::new("Flos Funkloch-aware Routenplaner")
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
        fapra::load_preprocessed_graph(path)
    } else {
        let tower_path = matches.value_of("tower-file").expect(
            "for pbf files a tower file is needed",
        );
        let mut towers = fapra::load_towers(tower_path).expect("Could not load towers file");
        let graph = fapra::load_graph(path, &mut towers);
        fapra::ApplicationState { graph, towers }
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
                fapra::web::route,
                fapra::web::next_node_to,
                fapra::web::serve_files,
                fapra::web::towers,
                fapra::web::download,
            ],
        )
        .manage(g.graph)
        .manage(g.towers)
        .launch();
}
