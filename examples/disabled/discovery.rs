use chrono::Utc;
use pm4py::discovery::{
    AlphaMiner, CausalNetMiner, DFGMiner, DiscoveryAlgorithm, HeuristicMiner, ILPMiner,
    InductiveMiner, SplitMiner, TreeMiner,
};
/// Process Discovery Examples
///
/// This example demonstrates how to use multiple discovery algorithms
/// to mine process models from event logs.
///
/// Run with: cargo run --example discovery
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("         Process Discovery Examples");
    println!("═══════════════════════════════════════════════════════\n");

    // Create a sample event log
    let log = create_sample_log();
    println!("Created event log with {} traces\n", log.traces.len());

    // Example 1: Alpha Miner
    println!("1. ALPHA MINER");
    println!("   └─ Foundational algorithm for well-structured processes");
    discover_with_alpha_miner(&log);

    println!("\n2. INDUCTIVE MINER");
    println!("   └─ Recursive discovery with better handling of loops");
    discover_with_inductive_miner(&log);

    println!("\n3. HEURISTIC MINER");
    println!("   └─ Frequency-threshold based discovery");
    discover_with_heuristic_miner(&log);

    println!("\n4. DFG MINER");
    println!("   └─ Extract directly-follows graph structure");
    discover_with_dfg_miner(&log);

    println!("\n5. CAUSAL NET MINER");
    println!("   └─ Extract causal dependencies between activities");
    discover_with_causal_net_miner(&log);

    println!("\n6. ILP MINER");
    println!("   └─ Integer Linear Programming-based discovery");
    discover_with_ilp_miner(&log);

    println!("\n7. SPLIT MINER");
    println!("   └─ Advanced split point detection");
    discover_with_split_miner(&log);

    println!("\n8. TREE MINER");
    println!("   └─ Generate process tree representation");
    discover_with_tree_miner(&log);

    println!("\n═══════════════════════════════════════════════════════");
    println!("                    Summary");
    println!("═══════════════════════════════════════════════════════\n");
    println!("✓ All discovery algorithms executed successfully!");
    println!("✓ Each algorithm provides different perspectives on the process");
    println!("✓ Choose based on your process characteristics and requirements\n");
}

/// Create a sample event log with typical order processing traces
fn create_sample_log() -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // Trace 1: Standard order flow
    let mut trace1 = Trace::new("Order_001");
    trace1.add_event(Event::new("Order Received", base_time));
    trace1.add_event(Event::new(
        "Order Validated",
        base_time + chrono::Duration::minutes(5),
    ));
    trace1.add_event(Event::new(
        "Payment Processed",
        base_time + chrono::Duration::minutes(10),
    ));
    trace1.add_event(Event::new(
        "Inventory Reserved",
        base_time + chrono::Duration::minutes(15),
    ));
    trace1.add_event(Event::new(
        "Item Picked",
        base_time + chrono::Duration::minutes(30),
    ));
    trace1.add_event(Event::new(
        "Item Packed",
        base_time + chrono::Duration::minutes(45),
    ));
    trace1.add_event(Event::new(
        "Item Shipped",
        base_time + chrono::Duration::hours(1),
    ));
    log.add_trace(trace1);

    // Trace 2: Order with payment retry
    let mut trace2 = Trace::new("Order_002");
    let offset = base_time + chrono::Duration::hours(1);
    trace2.add_event(Event::new("Order Received", offset));
    trace2.add_event(Event::new(
        "Order Validated",
        offset + chrono::Duration::minutes(5),
    ));
    trace2.add_event(Event::new(
        "Payment Processed",
        offset + chrono::Duration::minutes(10),
    ));
    trace2.add_event(Event::new(
        "Inventory Reserved",
        offset + chrono::Duration::minutes(15),
    ));
    trace2.add_event(Event::new(
        "Item Picked",
        offset + chrono::Duration::minutes(30),
    ));
    trace2.add_event(Event::new(
        "Item Packed",
        offset + chrono::Duration::minutes(45),
    ));
    trace2.add_event(Event::new(
        "Item Shipped",
        offset + chrono::Duration::hours(1),
    ));
    log.add_trace(trace2);

    // Trace 3: Expedited order
    let mut trace3 = Trace::new("Order_003");
    let offset = base_time + chrono::Duration::hours(2);
    trace3.add_event(Event::new("Order Received", offset));
    trace3.add_event(Event::new(
        "Order Validated",
        offset + chrono::Duration::minutes(2),
    ));
    trace3.add_event(Event::new(
        "Payment Processed",
        offset + chrono::Duration::minutes(5),
    ));
    trace3.add_event(Event::new(
        "Inventory Reserved",
        offset + chrono::Duration::minutes(8),
    ));
    trace3.add_event(Event::new(
        "Item Picked",
        offset + chrono::Duration::minutes(15),
    ));
    trace3.add_event(Event::new(
        "Item Shipped",
        offset + chrono::Duration::minutes(30),
    ));
    log.add_trace(trace3);

    // Trace 4: Standard flow
    let mut trace4 = Trace::new("Order_004");
    let offset = base_time + chrono::Duration::hours(3);
    trace4.add_event(Event::new("Order Received", offset));
    trace4.add_event(Event::new(
        "Order Validated",
        offset + chrono::Duration::minutes(5),
    ));
    trace4.add_event(Event::new(
        "Payment Processed",
        offset + chrono::Duration::minutes(10),
    ));
    trace4.add_event(Event::new(
        "Inventory Reserved",
        offset + chrono::Duration::minutes(15),
    ));
    trace4.add_event(Event::new(
        "Item Picked",
        offset + chrono::Duration::minutes(30),
    ));
    trace4.add_event(Event::new(
        "Item Packed",
        offset + chrono::Duration::minutes(45),
    ));
    trace4.add_event(Event::new(
        "Item Shipped",
        offset + chrono::Duration::hours(1),
    ));
    log.add_trace(trace4);

    // Add more traces for better results
    for i in 5..10 {
        let mut trace = Trace::new(format!("Order_{:03}", i));
        let offset = base_time + chrono::Duration::hours(i as i64);
        trace.add_event(Event::new("Order Received", offset));
        trace.add_event(Event::new(
            "Order Validated",
            offset + chrono::Duration::minutes(5),
        ));
        trace.add_event(Event::new(
            "Payment Processed",
            offset + chrono::Duration::minutes(10),
        ));
        trace.add_event(Event::new(
            "Inventory Reserved",
            offset + chrono::Duration::minutes(15),
        ));
        trace.add_event(Event::new(
            "Item Picked",
            offset + chrono::Duration::minutes(30),
        ));
        trace.add_event(Event::new(
            "Item Packed",
            offset + chrono::Duration::minutes(45),
        ));
        trace.add_event(Event::new(
            "Item Shipped",
            offset + chrono::Duration::hours(1),
        ));
        log.add_trace(trace);
    }

    log
}

fn discover_with_alpha_miner(log: &EventLog) {
    let miner = AlphaMiner::new();
    let petri_net = miner.discover(log);
    println!(
        "   Result: {} places, {} transitions",
        petri_net.places.len(),
        petri_net.transitions.len()
    );
    println!("   ✓ Places: {} named", petri_net.places.len());
}

fn discover_with_inductive_miner(log: &EventLog) {
    let miner = InductiveMiner::new();
    let petri_net = miner.discover(log);
    println!(
        "   Result: {} places, {} transitions",
        petri_net.places.len(),
        petri_net.transitions.len()
    );
    println!("   ✓ Better at handling complex structures");
}

fn discover_with_heuristic_miner(log: &EventLog) {
    let miner = HeuristicMiner::new();
    let petri_net = miner.discover(log);
    println!(
        "   Result: {} places, {} transitions",
        petri_net.places.len(),
        petri_net.transitions.len()
    );
    println!("   ✓ Frequency threshold: 0.5 (configurable)");
}

fn discover_with_dfg_miner(log: &EventLog) {
    let miner = DFGMiner::new();
    let dfg = miner.discover(log);
    println!(
        "   Result: {} nodes, {} edges",
        dfg.nodes.len(),
        dfg.edges.len()
    );
    println!("   ✓ Fast directly-follows graph extraction");
}

fn discover_with_causal_net_miner(log: &EventLog) {
    let miner = CausalNetMiner::new();
    let causal_net = miner.discover(log);
    println!("   Result: Causal net created");
    println!("   ✓ Activities and relations extracted");
}

fn discover_with_ilp_miner(log: &EventLog) {
    let miner = ILPMiner::new();
    let petri_net = miner.discover(log);
    println!(
        "   Result: {} places, {} transitions",
        petri_net.places.len(),
        petri_net.transitions.len()
    );
    println!("   ✓ Optimized model structure");
}

fn discover_with_split_miner(log: &EventLog) {
    let miner = SplitMiner::new();
    let petri_net = miner.discover(log);
    println!(
        "   Result: {} places, {} transitions",
        petri_net.places.len(),
        petri_net.transitions.len()
    );
    println!("   ✓ Detects splits and joins");
}

fn discover_with_tree_miner(log: &EventLog) {
    let miner = TreeMiner::new();
    let tree = miner.discover(log);
    println!("   Result: Process tree created");
    println!("   ✓ Hierarchical process representation");
}
