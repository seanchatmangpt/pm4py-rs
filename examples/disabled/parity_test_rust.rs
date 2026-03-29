// pm4py-rust parity test - compares with Python pm4py results

use chrono::Utc;
use pm4py::{discovery::AlphaMiner, Event, EventLog, Trace};

fn main() {
    println!("pm4py-rust Parity Test");
    println!("======================\n");

    // Create same log as Python test
    let mut log = EventLog::new();

    for i in 0..5 {
        let mut trace = Trace::new(format!("case_{}", i));

        let event_a = Event::new("A", Utc::now());
        trace.add_event(event_a);

        let event_b = Event::new("B", Utc::now());
        trace.add_event(event_b);

        let event_c = Event::new("C", Utc::now());
        trace.add_event(event_c);

        log.add_trace(trace);
    }

    // Run Alpha Miner
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    println!("Rust pm4py Alpha Miner:");
    println!("  Places: {}", net.places.len());
    println!("  Transitions: {}", net.transitions.len());
    println!("  Arcs: {}", net.arcs.len());
    println!();

    // Expected Python results (from pm4py 2.7.19.5):
    // Places: 4, Transitions: 3, Arcs: 6

    println!("Python pm4py 2.7.19.5 (expected):");
    println!("  Places: 4");
    println!("  Transitions: 3");
    println!("  Arcs: 6");
    println!();

    // Check parity
    let places_match = net.places.len() == 4;
    let transitions_match = net.transitions.len() == 3;
    let arcs_match = net.arcs.len() == 6;

    println!("Parity Check:");
    println!(
        "  Places: {}",
        if places_match { "✓ PASS" } else { "✗ FAIL" }
    );
    println!(
        "  Transitions: {}",
        if transitions_match {
            "✓ PASS"
        } else {
            "✗ FAIL"
        }
    );
    println!("  Arcs: {}", if arcs_match { "✓ PASS" } else { "✗ FAIL" });

    if places_match && transitions_match && arcs_match {
        println!("\n✓ Alpha Miner parity CONFIRMED");
        std::process::exit(0);
    } else {
        println!("\n✗ Alpha Miner parity FAILED");
        std::process::exit(1);
    }
}
