use chrono::Utc;
use pm4py::conformance::{ConformanceChecker, TokenReplay};
use pm4py::discovery::AlphaMiner;
/// Conformance Checking Examples
///
/// This example demonstrates how to check if process traces
/// conform to discovered or reference process models.
///
/// Run with: cargo run --example conformance
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("         Conformance Checking Examples");
    println!("═══════════════════════════════════════════════════════\n");

    // Create event logs with different conformance levels
    println!("Creating event logs with different conformance levels...\n");

    println!("1. PERFECTLY CONFORMING LOG");
    println!("   └─ All traces follow a consistent pattern");
    let perfect_log = create_perfect_log();
    check_conformance(&perfect_log);

    println!("\n2. PARTIALLY CONFORMING LOG");
    println!("   └─ Some traces deviate from the pattern");
    let partial_log = create_partial_log();
    check_conformance(&partial_log);

    println!("\n3. DISCOVERED VS ORIGINAL");
    println!("   └─ Compare discovered model with original log");
    let discovery_log = create_discovery_log();
    discover_and_check(&discovery_log);

    println!("\n═══════════════════════════════════════════════════════");
    println!("           Conformance Check Summary");
    println!("═══════════════════════════════════════════════════════\n");
    println!("Key Metrics Explained:");
    println!("  • Fitness: Portion of traces perfectly replayed (0-1)");
    println!("  • Precision: Model describes actual behavior (0-1)");
    println!("  • Generalization: Model captures process diversity\n");
}

fn create_perfect_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..5 {
        let mut trace = Trace::new(format!("perfect_{}", i));
        let offset = base + chrono::Duration::hours(i as i64);

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
            "Item Picked",
            offset + chrono::Duration::minutes(30),
        ));
        trace.add_event(Event::new(
            "Item Shipped",
            offset + chrono::Duration::hours(1),
        ));

        log.add_trace(trace);
    }
    log
}

fn create_partial_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Normal traces
    for i in 0..3 {
        let mut trace = Trace::new(format!("partial_ok_{}", i));
        let offset = base + chrono::Duration::hours(i as i64);

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
            "Item Picked",
            offset + chrono::Duration::minutes(30),
        ));
        trace.add_event(Event::new(
            "Item Shipped",
            offset + chrono::Duration::hours(1),
        ));

        log.add_trace(trace);
    }

    // Trace with deviation (skips Item Picked)
    let mut trace_deviate = Trace::new("partial_deviate_1");
    let offset = base + chrono::Duration::hours(3);
    trace_deviate.add_event(Event::new("Order Received", offset));
    trace_deviate.add_event(Event::new(
        "Order Validated",
        offset + chrono::Duration::minutes(5),
    ));
    trace_deviate.add_event(Event::new(
        "Payment Processed",
        offset + chrono::Duration::minutes(10),
    ));
    trace_deviate.add_event(Event::new(
        "Item Shipped",
        offset + chrono::Duration::hours(1),
    ));
    log.add_trace(trace_deviate);

    // Another deviant trace
    let mut trace_deviate2 = Trace::new("partial_deviate_2");
    let offset = base + chrono::Duration::hours(4);
    trace_deviate2.add_event(Event::new("Order Received", offset));
    trace_deviate2.add_event(Event::new(
        "Order Validated",
        offset + chrono::Duration::minutes(5),
    ));
    trace_deviate2.add_event(Event::new(
        "Payment Processed",
        offset + chrono::Duration::minutes(10),
    ));
    trace_deviate2.add_event(Event::new(
        "Item Picked",
        offset + chrono::Duration::minutes(30),
    ));
    trace_deviate2.add_event(Event::new(
        "Quality Check",
        offset + chrono::Duration::minutes(50),
    ));
    trace_deviate2.add_event(Event::new(
        "Item Shipped",
        offset + chrono::Duration::hours(1),
    ));
    log.add_trace(trace_deviate2);

    log
}

fn create_discovery_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..10 {
        let mut trace = Trace::new(format!("discovery_{}", i));
        let offset = base + chrono::Duration::hours(i as i64);

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
            "Item Picked",
            offset + chrono::Duration::minutes(30),
        ));
        trace.add_event(Event::new(
            "Item Shipped",
            offset + chrono::Duration::hours(1),
        ));

        log.add_trace(trace);
    }
    log
}

fn check_conformance(log: &EventLog) {
    // Discover a model from the log
    let miner = AlphaMiner::new();
    let model = miner.discover(log);

    // Check conformance
    let checker = TokenReplay::new();
    let result = checker.check(log, &model);

    println!("   ┌─ Fitness: {:.2}%", result.fitness * 100.0);
    println!("   ├─ Traces checked: {}", log.traces.len());
    println!("   ├─ Precision: {:.2}%", result.precision * 100.0);
    println!(
        "   └─ Generalization: {:.2}%",
        result.generalization * 100.0
    );

    // Categorize by conformance
    match result.fitness {
        f if f >= 0.95 => println!("   ✓ Status: EXCELLENT conformance"),
        f if f >= 0.75 => println!("   ⚠ Status: GOOD conformance (some deviations)"),
        f if f >= 0.50 => println!("   ⚠ Status: FAIR conformance (multiple deviations)"),
        _ => println!("   ✗ Status: POOR conformance"),
    }
}

fn discover_and_check(log: &EventLog) {
    println!("   Discovering model from log...");
    let miner = AlphaMiner::new();
    let discovered_model = miner.discover(log);
    println!(
        "   ✓ Discovered: {} places, {} transitions",
        discovered_model.places.len(),
        discovered_model.transitions.len()
    );

    println!("   Checking conformance against discovered model...");
    let checker = TokenReplay::new();
    let result = checker.check(log, &discovered_model);

    println!("   ┌─ Fitness: {:.2}%", result.fitness * 100.0);
    println!(
        "   ├─ Model explains behavior: {:.1}%",
        result.fitness * 100.0
    );
    println!("   └─ Precision: {:.2}%", result.precision * 100.0);

    println!("   ✓ Discovered model fits original log well");
}
