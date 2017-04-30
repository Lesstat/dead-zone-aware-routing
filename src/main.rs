extern crate fapra;

extern crate heapsize;

use heapsize::HeapSizeOf;


fn main() {
    let g = fapra::load_graph("/home/flo/workspaces/rust/graphdata/stuttgart-regbez-latest.osm.pbf");
    println!("Size of graph: {} MB",
             g.heap_size_of_children() / 1048576);
    let mut d = g.dijkstra();

    println!("{:?}" ,d.distance(5,500) );

}
