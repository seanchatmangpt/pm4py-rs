use pm4py::io::XESReader;
use pm4py::LogSkeletonMiner;
use std::path::Path;

fn main() {
    let log = XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    println!("Loaded: {} traces", log.len());

    let miner = LogSkeletonMiner::new();
    let skeleton = miner.discover(&log);

    println!("Log Skeleton discovered:");
    println!("  Equivalence relations: {}", skeleton.equivalence.len());
    println!("  After relations: {}", skeleton.after.len());
    println!("  Before relations: {}", skeleton.before.len());
    println!(
        "  Never-together relations: {}",
        skeleton.never_together.len()
    );
    println!(
        "  Directly-follows relations: {}",
        skeleton.directly_follows.len()
    );

    println!("\n✅ LogSkeletonMiner works!");
}
