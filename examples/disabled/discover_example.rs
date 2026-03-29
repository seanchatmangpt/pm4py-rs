//! Process Discovery Example
//!
//! Demonstrates discovering process models from event logs using different algorithms.
//! This example shows:
//! - Creating event logs
//! - Comparing different discovery algorithms
//! - Exporting discovered models
//!
//! Run with: cargo run --example discover_example

use chrono::{Duration, Utc};
use pm4py::discovery::{
    AlphaMiner, CausalNetMiner, DFGMiner, DiscoveryAlgorithm, HeuristicMiner, ILPMiner,
    InductiveMiner, SplitMiner, TreeMiner,
};
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║         PM4Py Process Discovery Example                ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Create a realistic event log
    let log = create_loan_approval_log();
    println!("Created event log:");
    println!("  Traces: {}", log.traces().len());
    println!("  Events: {}", log.num_events());
    println!("  Activities: {}\n", log.statistics().num_activities);

    // Discover using different algorithms
    println!("Discovering process models...\n");

    discover_with_alpha(&log);
    discover_with_inductive(&log);
    discover_with_heuristic(&log);
    discover_with_dfg(&log);
    discover_with_causal_net(&log);
    discover_with_split_miner(&log);
    discover_with_tree_miner(&log);

    // Comparison summary
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║                  Algorithm Comparison                   ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    println!("Alpha Miner:");
    println!("  ✓ Best for: Well-structured processes");
    println!("  ✓ Speed: Fast");
    println!("  ✗ Limitation: Struggles with loops\n");

    println!("Inductive Miner:");
    println!("  ✓ Best for: Complex, recursive processes");
    println!("  ✓ Speed: Medium");
    println!("  ✓ Feature: Excellent loop handling\n");

    println!("Heuristic Miner:");
    println!("  ✓ Best for: Noisy, real-world data");
    println!("  ✓ Speed: Fast");
    println!("  ✓ Feature: Frequency-based filtering\n");

    println!("DFG Miner:");
    println!("  ✓ Best for: Quick overview");
    println!("  ✓ Speed: Very fast");
    println!("  ✗ Limitation: No formal semantics\n");

    println!("Split Miner:");
    println!("  ✓ Best for: Advanced analysis");
    println!("  ✓ Speed: Slower");
    println!("  ✓ Feature: State-of-the-art accuracy\n");

    println!("Causal Net Miner:");
    println!("  ✓ Best for: Causal dependency analysis");
    println!("  ✓ Feature: Shows cause-effect relationships\n");
}

/// Create a realistic loan approval process log
fn create_loan_approval_log() -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // Trace 1: Standard approval (most common path)
    for i in 1..=30 {
        let mut trace = Trace::new(format!("loan_{:04}", i));
        let offset = base_time + Duration::hours(i as i64);

        trace.add_event(Event::new("apply", offset).with_resource("customer"));
        trace.add_event(
            Event::new("register", offset + Duration::minutes(2)).with_resource("clerk"),
        );
        trace.add_event(
            Event::new("verify_documents", offset + Duration::hours(1)).with_resource("officer"),
        );
        trace.add_event(
            Event::new("credit_check", offset + Duration::hours(2)).with_resource("system"),
        );
        trace
            .add_event(Event::new("approve", offset + Duration::hours(3)).with_resource("manager"));
        trace.add_event(
            Event::new("disburse", offset + Duration::hours(4)).with_resource("accountant"),
        );

        log.add_trace(trace);
    }

    // Trace 2: With rework (less common path)
    for i in 31..=40 {
        let mut trace = Trace::new(format!("loan_{:04}", i));
        let offset = base_time + Duration::hours(i as i64);

        trace.add_event(Event::new("apply", offset).with_resource("customer"));
        trace.add_event(
            Event::new("register", offset + Duration::minutes(2)).with_resource("clerk"),
        );
        trace.add_event(
            Event::new("verify_documents", offset + Duration::hours(1)).with_resource("officer"),
        );
        trace.add_event(
            Event::new("request_documents", offset + Duration::hours(2)).with_resource("officer"),
        );
        trace.add_event(
            Event::new("verify_documents", offset + Duration::hours(4)).with_resource("officer"),
        );
        trace.add_event(
            Event::new("credit_check", offset + Duration::hours(5)).with_resource("system"),
        );
        trace
            .add_event(Event::new("approve", offset + Duration::hours(6)).with_resource("manager"));
        trace.add_event(
            Event::new("disburse", offset + Duration::hours(7)).with_resource("accountant"),
        );

        log.add_trace(trace);
    }

    // Trace 3: With rejection
    for i in 41..=45 {
        let mut trace = Trace::new(format!("loan_{:04}", i));
        let offset = base_time + Duration::hours(i as i64);

        trace.add_event(Event::new("apply", offset).with_resource("customer"));
        trace.add_event(
            Event::new("register", offset + Duration::minutes(2)).with_resource("clerk"),
        );
        trace.add_event(
            Event::new("verify_documents", offset + Duration::hours(1)).with_resource("officer"),
        );
        trace.add_event(
            Event::new("credit_check", offset + Duration::hours(2)).with_resource("system"),
        );
        trace.add_event(Event::new("reject", offset + Duration::hours(3)).with_resource("manager"));
        trace.add_event(
            Event::new("notify_applicant", offset + Duration::hours(4)).with_resource("system"),
        );

        log.add_trace(trace);
    }

    log
}

fn discover_with_alpha(log: &EventLog) {
    println!("1. ALPHA MINER");
    println!("   └─ Foundational algorithm for well-structured processes");

    let miner = AlphaMiner::new();
    let model = miner.mine(log);

    println!(
        "   Model: {} places, {} transitions",
        model.places.len(),
        model.transitions.len()
    );
    println!("   Use case: Simple, linear processes with few exceptions\n");
}

fn discover_with_inductive(log: &EventLog) {
    println!("2. INDUCTIVE MINER");
    println!("   └─ Recursive discovery with excellent loop handling");

    let miner = InductiveMiner::new();
    let model = miner.mine(log);

    println!(
        "   Model: {} places, {} transitions",
        model.places.len(),
        model.transitions.len()
    );
    println!("   Use case: Complex processes with loops and recursion\n");
}

fn discover_with_heuristic(log: &EventLog) {
    println!("3. HEURISTIC MINER");
    println!("   └─ Frequency-threshold based discovery for noisy data");

    let miner = HeuristicMiner::new().with_frequency_threshold(0.15);
    let model = miner.mine(log);

    println!(
        "   Model: {} places, {} transitions (with 15% threshold)",
        model.places.len(),
        model.transitions.len()
    );
    println!("   Use case: Real-world data with noise and outliers\n");
}

fn discover_with_dfg(log: &EventLog) {
    println!("4. DFG MINER");
    println!("   └─ Extract directly-follows graph structure");

    let miner = DFGMiner::new();
    let dfg = miner.mine(log);

    // DFG structure varies, just show that it ran
    println!("   Graph extracted from activity succession");
    println!("   Use case: Quick overview of process flow\n");
}

fn discover_with_causal_net(log: &EventLog) {
    println!("5. CAUSAL NET MINER");
    println!("   └─ Extract causal dependencies between activities");

    let miner = CausalNetMiner::new();
    let causal_net = miner.mine(log);

    println!("   Causal net: {} activities", causal_net.activities.len());
    println!("   Use case: Understanding cause-effect relationships\n");
}

fn discover_with_split_miner(log: &EventLog) {
    println!("6. SPLIT MINER");
    println!("   └─ Advanced split point detection");

    let miner = SplitMiner::new();
    let model = miner.mine(log);

    println!(
        "   Model: {} places, {} transitions",
        model.places.len(),
        model.transitions.len()
    );
    println!("   Use case: State-of-the-art accuracy with higher complexity\n");
}

fn discover_with_tree_miner(log: &EventLog) {
    println!("7. TREE MINER");
    println!("   └─ Generate process tree representation");

    let miner = TreeMiner::new();
    let tree = miner.mine(log);

    println!("   Process tree generated");
    println!("   Use case: Hierarchical process understanding\n");
}
