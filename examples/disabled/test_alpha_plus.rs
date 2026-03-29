use pm4py::io::XESReader;
use pm4py::AlphaPlusMiner;
use std::path::Path;

fn main() {
    let log = XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    println!("Loaded: {} traces", log.len());

    let miner = AlphaPlusMiner::new();
    let net = miner.discover(&log);

    println!(
        "AlphaPlusMiner discovered: {} places, {} transitions, {} arcs",
        net.places.len(),
        net.transitions.len(),
        net.arcs.len()
    );
    println!("✅ AlphaPlusMiner works!");
}
