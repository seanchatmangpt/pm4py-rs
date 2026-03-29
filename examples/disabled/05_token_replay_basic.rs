//! Example 5: Basic Token Replay Conformance Checking
//!
//! Demonstrates how to check if an event log conforms to a discovered model.
//! Token replay is the standard conformance checking method.
//!
//! Run with: cargo run --example 05_token_replay_basic

use chrono::{Duration, Utc};
use pm4py::conformance::TokenReplay;
use pm4py::discovery::AlphaMiner;
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║      Example 5: Token Replay Conformance Check        ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Step 1: Create training log (model ground truth)
    let training_log = create_training_log();
    println!("Training Log (used to discover model):");
    println!("  Traces: {}", training_log.traces().len());

    // Step 2: Discover model from training log
    let miner = AlphaMiner::new();
    let model = miner.discover(&training_log);
    println!("  Discovered {} transitions\n", model.transitions.len());

    // Step 3: Create test log (may have deviations)
    let test_log = create_test_log();
    println!("Test Log (checking conformance):");
    println!("  Traces: {}", test_log.traces().len());
    println!("  (Mix of conforming and non-conforming cases)\n");

    // Step 4: Check conformance
    let checker = TokenReplay::new();
    let results = checker.replay(&test_log, &model);

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║             CONFORMANCE RESULTS                        ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let mut total_fitness = 0.0;
    for result in &results {
        let status = if result.fitness >= 1.0 {
            "✓ CONFORMS"
        } else if result.fitness >= 0.8 {
            "≈ MOSTLY OK"
        } else {
            "✗ DEVIATES"
        };

        println!(
            "Case {}: fitness = {:.2} [{}]",
            result.trace_id, result.fitness, status
        );

        total_fitness += result.fitness;
    }

    println!();
    let avg_fitness = total_fitness / results.len() as f64;
    println!("Average fitness: {:.2}", avg_fitness);

    // Interpretation
    println!("\nInterpretation:");
    if avg_fitness >= 0.95 {
        println!("  ✓ Excellent fit - model captures process well");
    } else if avg_fitness >= 0.80 {
        println!("  ≈ Good fit - model mostly captures behavior");
    } else if avg_fitness >= 0.60 {
        println!("  ≈ Fair fit - model misses some behavior");
    } else {
        println!("  ✗ Poor fit - model needs improvement");
    }
}

/// Create a clean training log for model discovery
fn create_training_log() -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // All traces follow: A → B → C → D
    for case_id in 0..5 {
        let mut trace = Trace::new(format!("train_{}", case_id));
        let offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("A".to_string(), offset));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(2)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(3)));

        log.add_trace(trace);
    }

    log
}

/// Create a test log with some deviations
fn create_test_log() -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // Case 1: Conforming - follows model perfectly
    {
        let mut trace = Trace::new("test_1_conform");
        let offset = base_time;

        trace.add_event(Event::new("A".to_string(), offset));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(2)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(3)));

        log.add_trace(trace);
    }

    // Case 2: Mostly conforming - skips one activity
    {
        let mut trace = Trace::new("test_2_skip_c");
        let offset = base_time + Duration::days(1);

        trace.add_event(Event::new("A".to_string(), offset));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(1)));
        // Skips C
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(2)));

        log.add_trace(trace);
    }

    // Case 3: Non-conforming - wrong order
    {
        let mut trace = Trace::new("test_3_wrong_order");
        let offset = base_time + Duration::days(2);

        trace.add_event(Event::new("A".to_string(), offset));
        // B and C out of order
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(2)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(3)));

        log.add_trace(trace);
    }

    // Case 4: Partially conforming - extra activity E
    {
        let mut trace = Trace::new("test_4_extra_activity");
        let offset = base_time + Duration::days(3);

        trace.add_event(Event::new("A".to_string(), offset));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("E".to_string(), offset + Duration::hours(2))); // Extra!
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(3)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(4)));

        log.add_trace(trace);
    }

    log
}
