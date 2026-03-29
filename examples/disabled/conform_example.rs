//! Conformance Checking Example
//!
//! Demonstrates how to check if event logs conform to process models.
//! This example shows:
//! - Creating logs with different conformance levels
//! - Checking conformance against models
//! - Identifying deviant cases
//! - Interpreting metrics
//!
//! Run with: cargo run --example conform_example

use chrono::{Duration, Utc};
use pm4py::conformance::TokenReplay;
use pm4py::discovery::InductiveMiner;
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║       PM4Py Conformance Checking Example               ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Scenario 1: Perfect conformance
    println!("SCENARIO 1: Perfect Process Conformance");
    println!("─────────────────────────────────────────\n");

    let perfect_log = create_perfect_log();
    let miner = InductiveMiner::new();
    let model = miner.mine(&perfect_log);

    println!(
        "Log: {} traces, {} events",
        perfect_log.traces().len(),
        perfect_log.num_events()
    );
    println!(
        "Model: {} places, {} transitions\n",
        model.places.len(),
        model.transitions.len()
    );

    let checker = TokenReplay::new();
    let result = checker.replay(&perfect_log, &model);

    println!("Results:");
    println!("  Fitness: {:.1}% ✓", result.fitness * 100.0);
    println!("  Precision: {:.1}% ✓", result.precision * 100.0);
    println!("  Generalization: {:.1}% ✓", result.generalization * 100.0);
    println!("  Simplicity: {:.1}%", result.simplicity * 100.0);
    println!("  Deviant traces: {}\n", result.deviant_traces.len());

    println!("Interpretation:");
    println!("  ✓ All traces follow the model perfectly");
    println!("  ✓ Model captures exactly the observed behavior");
    println!("  ✓ No anomalies detected\n");

    // Scenario 2: Partial conformance
    println!("\n════════════════════════════════════════════════════════\n");
    println!("SCENARIO 2: Partial Process Conformance");
    println!("──────────────────────────────────────────\n");

    let partial_log = create_partial_conformance_log();
    let model2 = miner.mine(&partial_log);

    println!(
        "Log: {} traces, {} events",
        partial_log.traces().len(),
        partial_log.num_events()
    );
    println!(
        "Model: {} places, {} transitions\n",
        model2.places.len(),
        model2.transitions.len()
    );

    let result2 = checker.replay(&partial_log, &model2);

    println!("Results:");
    println!("  Fitness: {:.1}%", result2.fitness * 100.0);
    if result2.fitness < 0.95 {
        println!("           ⚠️  Low fitness!");
    } else {
        println!("           ✓");
    }
    println!("  Precision: {:.1}%", result2.precision * 100.0);
    println!("  Generalization: {:.1}%", result2.generalization * 100.0);
    println!("  Simplicity: {:.1}%", result2.simplicity * 100.0);
    println!(
        "  Deviant traces: {} / {}\n",
        result2.deviant_traces.len(),
        partial_log.traces().len()
    );

    // Show deviant traces
    if !result2.deviant_traces.is_empty() {
        println!("Deviant cases (first 3):");
        for (i, trace_id) in result2.deviant_traces.iter().take(3).enumerate() {
            println!("  {}. {}", i + 1, trace_id);

            if let Some(details) = result2.conformance_details.get(trace_id) {
                println!("     - Remaining tokens: {}", details.remaining_tokens);
                if !details.enabled_transitions.is_empty() {
                    println!("     - Could have done: {:?}", details.enabled_transitions);
                }
            }
        }
        if result2.deviant_traces.len() > 3 {
            println!("  ... and {} more", result2.deviant_traces.len() - 3);
        }
    }

    println!("\nInterpretation:");
    println!(
        "  ⚠️ {} some traces deviate from the model",
        result2.deviant_traces.len()
    );
    println!("  → Possible causes:");
    println!("    1. Process definition is incomplete");
    println!("    2. Real-world exceptions in execution");
    println!("    3. Data quality issues\n");

    // Scenario 3: Discovering from mixed data
    println!("\n════════════════════════════════════════════════════════\n");
    println!("SCENARIO 3: Discovery from Mixed Data");
    println!("─────────────────────────────────────────\n");

    let mixed_log = create_mixed_conformance_log();
    println!(
        "Log: {} traces, {} events",
        mixed_log.traces().len(),
        mixed_log.num_events()
    );
    println!("  (Mix of standard and exceptional cases)\n");

    // Try different frequency thresholds
    println!("Testing different discovery approaches:\n");

    test_discovery_approach(&mixed_log, "Strict", 0.5);
    test_discovery_approach(&mixed_log, "Moderate", 0.25);
    test_discovery_approach(&mixed_log, "Permissive", 0.10);

    // Scenario 4: Best practices
    println!("\n════════════════════════════════════════════════════════\n");
    println!("BEST PRACTICES FOR CONFORMANCE CHECKING");
    println!("─────────────────────────────────────────\n");

    println!("1. INTERPRET METRICS");
    println!("   Fitness > 0.95: ✓ Good, model captures most behavior");
    println!("   Fitness 0.80-0.95: ⚠️ Some exceptions exist");
    println!("   Fitness < 0.80: ✗ Major gaps in model\n");

    println!("2. HANDLE DEVIANT CASES");
    println!("   a) Analyze root causes (exceptions or errors?)");
    println!("   b) Filter outliers if data quality issue");
    println!("   c) Update model if process changed\n");

    println!("3. IMPROVE LOW CONFORMANCE");
    println!("   a) Use heuristic miner with frequency filtering");
    println!("   b) Check data quality (missing fields, wrong format)");
    println!("   c) Rediscover with complete dataset\n");

    println!("4. USE PRECISION TO DETECT OVERFITTING");
    println!("   Precision > 0.95: ✓ Model is not overfitted");
    println!("   Precision < 0.80: ⚠️ Model might be too permissive\n");
}

fn create_perfect_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // All traces follow same pattern
    for i in 0..10 {
        let mut trace = Trace::new(format!("perfect_{}", i));
        let offset = base + Duration::hours(i as i64);

        trace.add_event(Event::new("request", offset));
        trace.add_event(Event::new("validate", offset + Duration::minutes(5)));
        trace.add_event(Event::new("process", offset + Duration::minutes(30)));
        trace.add_event(Event::new("approve", offset + Duration::hours(1)));
        trace.add_event(Event::new("complete", offset + Duration::hours(2)));

        log.add_trace(trace);
    }
    log
}

fn create_partial_conformance_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Standard traces (70%)
    for i in 0..7 {
        let mut trace = Trace::new(format!("standard_{}", i));
        let offset = base + Duration::hours(i as i64);

        trace.add_event(Event::new("request", offset));
        trace.add_event(Event::new("validate", offset + Duration::minutes(5)));
        trace.add_event(Event::new("process", offset + Duration::minutes(30)));
        trace.add_event(Event::new("approve", offset + Duration::hours(1)));
        trace.add_event(Event::new("complete", offset + Duration::hours(2)));

        log.add_trace(trace);
    }

    // Deviant traces (30%) - skip validation
    for i in 7..10 {
        let mut trace = Trace::new(format!("deviant_{}", i));
        let offset = base + Duration::hours(i as i64);

        trace.add_event(Event::new("request", offset));
        // Missing: validate
        trace.add_event(Event::new("process", offset + Duration::minutes(10)));
        trace.add_event(Event::new("complete", offset + Duration::hours(1)));

        log.add_trace(trace);
    }

    log
}

fn create_mixed_conformance_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Path 1: Standard (50%)
    for i in 0..5 {
        let mut trace = Trace::new(format!("path1_{}", i));
        let offset = base + Duration::hours(i as i64);

        trace.add_event(Event::new("step_a", offset));
        trace.add_event(Event::new("step_b", offset + Duration::minutes(10)));
        trace.add_event(Event::new("step_d", offset + Duration::minutes(20)));

        log.add_trace(trace);
    }

    // Path 2: With intermediate step (30%)
    for i in 5..8 {
        let mut trace = Trace::new(format!("path2_{}", i));
        let offset = base + Duration::hours(i as i64);

        trace.add_event(Event::new("step_a", offset));
        trace.add_event(Event::new("step_b", offset + Duration::minutes(10)));
        trace.add_event(Event::new("step_c", offset + Duration::minutes(15)));
        trace.add_event(Event::new("step_d", offset + Duration::minutes(25)));

        log.add_trace(trace);
    }

    // Path 3: Alternative (20%)
    for i in 8..10 {
        let mut trace = Trace::new(format!("path3_{}", i));
        let offset = base + Duration::hours(i as i64);

        trace.add_event(Event::new("step_a", offset));
        trace.add_event(Event::new("step_e", offset + Duration::minutes(5)));
        trace.add_event(Event::new("step_d", offset + Duration::minutes(20)));

        log.add_trace(trace);
    }

    log
}

fn test_discovery_approach(log: &EventLog, label: &str, threshold: f64) {
    use pm4py::discovery::HeuristicMiner;

    let miner = HeuristicMiner::new().with_frequency_threshold(threshold);
    let model = miner.mine(log);

    let checker = TokenReplay::new();
    let result = checker.replay(log, &model);

    println!("{} (threshold: {:.0}%)", label, threshold * 100.0);
    println!("  Model size: {} transitions", model.transitions.len());
    println!("  Fitness: {:.1}%", result.fitness * 100.0);
    println!("  Deviant: {} traces", result.deviant_traces.len());

    if result.fitness > 0.95 {
        println!("  ✓ Excellent conformance\n");
    } else if result.fitness > 0.80 {
        println!("  ⚠️  Acceptable with caveats\n");
    } else {
        println!("  ✗ Needs revision\n");
    }
}
