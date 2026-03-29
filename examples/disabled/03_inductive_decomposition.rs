//! Example 3: Inductive Miner with Decomposition
//!
//! Demonstrates the Inductive Miner which discovers process trees
//! through recursive decomposition. Best for complex processes with loops.
//!
//! Run with: cargo run --example 03_inductive_decomposition

use chrono::{Duration, Utc};
use pm4py::discovery::InductiveMiner;
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║      Example 3: Inductive Miner Decomposition          ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Create a log with choice and loops
    let log = create_complex_event_log();

    println!("Log Statistics:");
    let stats = log.statistics();
    println!("  Traces: {}", stats.num_traces);
    println!("  Events: {}", stats.num_events);
    println!("  Activities: {}", stats.num_activities);
    println!("  Variants: {}\n", stats.num_variants);

    // Discover with Inductive Miner
    let miner = InductiveMiner::new();
    let tree = miner.discover(&log);

    println!("Discovered Process Tree:");
    println!("  Root: {:?}", tree.root.operator);
    println!("  Depth: {}", tree.depth());
    println!("  Leaf count: {}", tree.leaf_count());

    println!("\nProcess Tree Structure:");
    print_tree(&tree.root, 0);

    println!("\n✓ Tree discovery complete!");
}

/// Create a log with choice (either approval OR rejection) and loops
fn create_complex_event_log() -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // Main path: Request → Review → {Approve | Reject}
    for case_id in 0..5 {
        let mut trace = Trace::new(format!("case_app_{}", case_id));
        let offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("Request".to_string(), offset));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Approve".to_string(),
            offset + Duration::hours(2),
        ));

        log.add_trace(trace);
    }

    // Alternative: Request → Review → Reject
    for case_id in 5..8 {
        let mut trace = Trace::new(format!("case_rej_{}", case_id));
        let offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("Request".to_string(), offset));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Reject".to_string(),
            offset + Duration::hours(2),
        ));

        log.add_trace(trace);
    }

    // With rework (loop): Request → Review → [Rework → Review] → Approve
    for case_id in 8..10 {
        let mut trace = Trace::new(format!("case_rework_{}", case_id));
        let mut offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("Request".to_string(), offset));
        offset = offset + Duration::hours(1);
        trace.add_event(Event::new("Review".to_string(), offset));

        // Loop iteration
        offset = offset + Duration::hours(1);
        trace.add_event(Event::new("Rework".to_string(), offset));
        offset = offset + Duration::hours(1);
        trace.add_event(Event::new("Review".to_string(), offset));

        offset = offset + Duration::hours(1);
        trace.add_event(Event::new("Approve".to_string(), offset));

        log.add_trace(trace);
    }

    log
}

/// Print tree structure with indentation
fn print_tree(node: &pm4py::models::ProcessTreeNode, depth: usize) {
    let indent = "  ".repeat(depth);

    match &node.operator {
        pm4py::models::TreeOperator::Activity(name) => {
            println!("{}├─ {}", indent, name);
        }
        pm4py::models::TreeOperator::Sequence => {
            println!("{}├─ SEQUENCE (→)", indent);
            for child in &node.children {
                print_tree(child, depth + 1);
            }
        }
        pm4py::models::TreeOperator::Parallel => {
            println!("{}├─ PARALLEL (∥)", indent);
            for child in &node.children {
                print_tree(child, depth + 1);
            }
        }
        pm4py::models::TreeOperator::Choice => {
            println!("{}├─ CHOICE (⊕)", indent);
            for child in &node.children {
                print_tree(child, depth + 1);
            }
        }
        pm4py::models::TreeOperator::Loop => {
            println!("{}├─ LOOP (↻)", indent);
            for child in &node.children {
                print_tree(child, depth + 1);
            }
        }
    }
}
