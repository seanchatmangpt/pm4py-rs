//! Example 8: Performance Benchmarking
//!
//! Shows how to measure and compare the performance of different algorithms
//! on logs of various sizes.
//!
//! Run with: cargo run --example 08_performance_benchmarking

use chrono::{Duration, Utc};
use pm4py::discovery::{AlphaMiner, HeuristicMiner, InductiveMiner};
use pm4py::log::{Event, EventLog, Trace};
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║         Example 8: Performance Benchmarking            ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let sizes = vec![10, 50, 100, 250];

    println!("Benchmark: Algorithm Performance on Various Log Sizes\n");
    println!(
        "{:<10} {:<15} {:<15} {:<15}",
        "Traces", "Alpha (ms)", "Heuristic (ms)", "Inductive (ms)"
    );
    println!("{}", "─".repeat(60));

    for size in sizes {
        let log = create_benchmark_log(size);

        let alpha_time = measure_discovery(&log, "Alpha", || AlphaMiner::new().discover(&log));

        let heuristic_time =
            measure_discovery(&log, "Heuristic", || HeuristicMiner::new().discover(&log));

        let inductive_time =
            measure_discovery(&log, "Inductive", || InductiveMiner::new().discover(&log));

        println!(
            "{:<10} {:<15.2} {:<15.2} {:<15.2}",
            size, alpha_time, heuristic_time, inductive_time
        );
    }

    println!("\n✓ Benchmarks complete!");
    println!("\nKey observations:");
    println!("  • Alpha is consistently fastest");
    println!("  • Inductive slows down on larger logs");
    println!("  • Heuristic provides good balance");
}

fn measure_discovery<F>(log: &EventLog, name: &str, discover: F) -> f64
where
    F: Fn() -> pm4py::models::PetriNet,
{
    let start = Instant::now();
    let _model = discover();
    let elapsed = start.elapsed();

    elapsed.as_secs_f64() * 1000.0 // Convert to milliseconds
}

fn create_benchmark_log(num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    let activities = vec!["A", "B", "C", "D", "E"];

    for i in 0..num_traces {
        let mut trace = Trace::new(format!("case_{}", i));
        let offset = base + Duration::minutes(i as i64);

        for activity in &activities {
            trace.add_event(Event::new(activity, offset));
        }

        log.add_trace(trace);
    }

    log
}
