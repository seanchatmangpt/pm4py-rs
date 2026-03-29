use chrono::Utc;
use pm4py::discovery::{CausalNetMiner, ILPMiner, SplitMiner};
/// Test new discovery algorithms
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    // Create a simple log
    let mut log = EventLog::new();

    for i in 0..5 {
        let mut trace = Trace::new(format!("case{}", i));
        trace.add_event(Event::new("A", Utc::now()));
        trace.add_event(Event::new("B", Utc::now()));
        trace.add_event(Event::new("C", Utc::now()));
        log.add_trace(trace);
    }

    println!("Testing ILP Miner...");
    let ilp_miner = ILPMiner::new();
    let ilp_net = ilp_miner.discover(&log);
    println!(
        "ILP discovered {} places and {} transitions",
        ilp_net.places.len(),
        ilp_net.transitions.len()
    );

    println!("\nTesting Split Miner...");
    let split_miner = SplitMiner::new();
    let split_net = split_miner.discover(&log);
    println!(
        "Split discovered {} places and {} transitions",
        split_net.places.len(),
        split_net.transitions.len()
    );

    println!("\nTesting Causal Net Miner...");
    let causal_miner = CausalNetMiner::new();
    let causal_net = causal_miner.discover(&log);
    println!(
        "Causal Net discovered {} activities and {} relations",
        causal_net.num_activities(),
        causal_net.num_relations()
    );

    // Test trace acceptance
    let trace = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    println!(
        "Causal Net accepts [A->B->C]: {}",
        causal_net.accepts_trace(&trace)
    );

    println!("\nAll tests completed successfully!");
}
