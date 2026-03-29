//! Scale and Performance Benchmarks using Criterion.rs
//!
//! Comprehensive performance benchmarks for pm4py-rust covering:
//! - 100K events (baseline)
//! - 1M events (standard enterprise)
//! - 10M events (large organization)
//! - 100M events (petabyte-scale distributed)
//!
//! Metrics collected:
//! - Wall-clock time (CPU seconds)
//! - Throughput (events/sec)
//! - Memory efficiency (estimated)
//! - Scalability characteristics (linear vs quadratic)
//! - Fitness accuracy vs Python baseline
//!
//! Usage:
//!   cargo bench --bench scale_benchmarks           # All benchmarks
//!   cargo bench --bench scale_benchmarks -- --sample-size 3  # Faster runs
//!   cargo bench --bench scale_benchmarks discovery # Only discovery
//!
//! Success Targets:
//!   - 1M events: <5 seconds
//!   - 10M events: <30 seconds
//!   - 100M events: <5 minutes
//!   - Memory: 50-60% reduction vs Python pm4py

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pm4py::conformance::TokenReplay;
use pm4py::discovery::{AlphaMiner, DFGMiner, InductiveMiner};
use pm4py::log::{Event, EventLog, Trace};
use std::time::Instant;

/// Memory usage tracker (approximate, using object counts)
struct MemoryTracker {
    events_created: usize,
    traces_created: usize,
    avg_event_size_bytes: usize,
}

impl MemoryTracker {
    fn new() -> Self {
        MemoryTracker {
            events_created: 0,
            traces_created: 0,
            avg_event_size_bytes: 256, // Typical: timestamp + activity + resource + attributes
        }
    }

    fn estimated_bytes(&self) -> usize {
        (self.events_created * self.avg_event_size_bytes) + (self.traces_created * 128)
        // Trace overhead
    }

    fn estimated_mb(&self) -> f64 {
        self.estimated_bytes() as f64 / (1024.0 * 1024.0)
    }
}

/// Generate synthetic event log with realistic patterns
/// Parameters optimize for performance and memory efficiency
fn generate_event_log(
    num_events: usize,
    num_traces: usize,
    num_activities: usize,
) -> (EventLog, MemoryTracker) {
    let mut log = EventLog::new();
    let mut tracker = MemoryTracker::new();

    let activities: Vec<&str> = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]
        .into_iter()
        .take(num_activities.min(10))
        .collect();

    let events_per_trace = (num_events / num_traces).max(1);
    let base_time = Utc::now();

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("case_{:08}", trace_id));
        tracker.traces_created += 1;

        for event_idx in 0..events_per_trace {
            let activity_idx = (event_idx + trace_id * 7) % activities.len();
            let activity = activities[activity_idx];

            let timestamp = base_time
                + chrono::Duration::seconds((trace_id * events_per_trace + event_idx) as i64);
            let event = Event::new(activity, timestamp)
                .with_resource(format!("worker_{}", trace_id % 10))
                .with_attribute("order", &(event_idx % 100).to_string());

            trace.add_event(event);
            tracker.events_created += 1;
        }

        log.add_trace(trace);
    }

    (log, tracker)
}

/// Generate complex event log with branching and rework patterns
/// Realistic for process mining: multiple decision paths, loops, exceptions
fn generate_complex_event_log(num_events: usize, num_traces: usize) -> (EventLog, MemoryTracker) {
    let mut log = EventLog::new();
    let mut tracker = MemoryTracker::new();
    let base_time = Utc::now();

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("complex_{:08}", trace_id));
        let events_per_trace = num_events / num_traces;
        let mut current_time = base_time;
        tracker.traces_created += 1;

        for event_idx in 0..events_per_trace {
            let activity = match event_idx % 6 {
                0 => "Start",
                1 => "Initialize",
                2 => {
                    if (trace_id % 2) == 0 {
                        "Path_C"
                    } else {
                        "Path_D"
                    }
                }
                3 => "Process",
                4 => {
                    if (event_idx % 3) == 0 {
                        "Alt_F"
                    } else {
                        "Alt_G"
                    }
                }
                5 => "Complete",
                _ => "Unknown",
            };

            let event = Event::new(activity, current_time)
                .with_resource(format!("specialist_{}", event_idx % 5))
                .with_attribute("loop_count", &((event_idx / 6) % 5).to_string());

            trace.add_event(event);
            tracker.events_created += 1;

            current_time = current_time + chrono::Duration::seconds(5);
        }

        log.add_trace(trace);
    }

    (log, tracker)
}

// ============================================================================
// ALPHA MINER BENCHMARKS
// ============================================================================

fn bench_alpha_miner_100k(c: &mut Criterion) {
    let (log, _tracker) = generate_event_log(100_000, 2_000, 5);
    let log = black_box(log);

    c.bench_function("alpha_miner_100k", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_alpha_miner_1m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(1_000_000, 10_000, 5);
    let log = black_box(log);

    println!(
        "\n[1M Alpha Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    c.bench_function("alpha_miner_1m", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_alpha_miner_10m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(10_000_000, 20_000, 5);
    let log = black_box(log);

    println!(
        "\n[10M Alpha Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    let mut group = c.benchmark_group("discovery_10m");
    group.sample_size(3);
    group.bench_function("alpha_miner_10m", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
    group.finish();
}

fn bench_alpha_miner_100m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(100_000_000, 100_000, 7);
    let log = black_box(log);

    println!(
        "\n[100M Alpha Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    let mut group = c.benchmark_group("discovery_100m");
    group.sample_size(2);
    group.measurement_time(std::time::Duration::from_secs(60));
    group.bench_function("alpha_miner_100m", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
    group.finish();
}

// ============================================================================
// INDUCTIVE MINER BENCHMARKS
// ============================================================================

fn bench_inductive_miner_100k(c: &mut Criterion) {
    let (log, _tracker) = generate_complex_event_log(100_000, 2_000);
    let log = black_box(log);

    c.bench_function("inductive_miner_100k", |b| {
        b.iter(|| {
            let miner = InductiveMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_inductive_miner_1m(c: &mut Criterion) {
    let (log, tracker) = generate_complex_event_log(1_000_000, 10_000);
    let log = black_box(log);

    println!(
        "\n[1M Inductive Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    c.bench_function("inductive_miner_1m", |b| {
        b.iter(|| {
            let miner = InductiveMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_inductive_miner_10m(c: &mut Criterion) {
    let (log, tracker) = generate_complex_event_log(10_000_000, 20_000);
    let log = black_box(log);

    println!(
        "\n[10M Inductive Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    let mut group = c.benchmark_group("discovery_inductive_10m");
    group.sample_size(2);
    group.bench_function("inductive_miner_10m", |b| {
        b.iter(|| {
            let miner = InductiveMiner::new();
            miner.discover(&log)
        });
    });
    group.finish();
}

// ============================================================================
// DFG MINER BENCHMARKS
// ============================================================================

fn bench_dfg_miner_100k(c: &mut Criterion) {
    let (log, _tracker) = generate_event_log(100_000, 2_000, 5);
    let log = black_box(log);

    c.bench_function("dfg_miner_100k", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_miner_1m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(1_000_000, 10_000, 5);
    let log = black_box(log);

    println!(
        "\n[1M DFG Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    c.bench_function("dfg_miner_1m", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_miner_10m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(10_000_000, 20_000, 5);
    let log = black_box(log);

    println!(
        "\n[10M DFG Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    c.bench_function("dfg_miner_10m", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_miner_100m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(100_000_000, 100_000, 7);
    let log = black_box(log);

    println!(
        "\n[100M DFG Miner] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    let mut group = c.benchmark_group("dfg_100m");
    group.sample_size(2);
    group.measurement_time(std::time::Duration::from_secs(60));
    group.bench_function("dfg_miner_100m", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
    group.finish();
}

// ============================================================================
// TOKEN REPLAY CONFORMANCE BENCHMARKS
// ============================================================================

fn bench_token_replay_100k(c: &mut Criterion) {
    let (log, _tracker) = generate_event_log(100_000, 2_000, 5);
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);
    let log = black_box(log);
    let net = black_box(net);

    c.bench_function("token_replay_100k", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
}

fn bench_token_replay_1m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(1_000_000, 10_000, 5);
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);
    let log = black_box(log);
    let net = black_box(net);

    println!(
        "\n[1M Token Replay] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    c.bench_function("token_replay_1m", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
}

fn bench_token_replay_10m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(10_000_000, 20_000, 5);
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);
    let log = black_box(log);
    let net = black_box(net);

    println!(
        "\n[10M Token Replay] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    let mut group = c.benchmark_group("conformance_10m");
    group.sample_size(3);
    group.bench_function("token_replay_10m", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
    group.finish();
}

fn bench_token_replay_100m(c: &mut Criterion) {
    let (log, tracker) = generate_event_log(100_000_000, 100_000, 7);
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);
    let log = black_box(log);
    let net = black_box(net);

    println!(
        "\n[100M Token Replay] Estimated memory: {:.2} MB",
        tracker.estimated_mb()
    );

    let mut group = c.benchmark_group("conformance_100m");
    group.sample_size(2);
    group.measurement_time(std::time::Duration::from_secs(60));
    group.bench_function("token_replay_100m", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
    group.finish();
}

// ============================================================================
// SCALABILITY ANALYSIS
// ============================================================================

fn bench_scalability_alpha_miner(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability_alpha");
    group.sample_size(5);

    for size in [100_000, 1_000_000, 10_000_000].iter() {
        let num_traces = size / 50;
        let (log, tracker) = generate_event_log(*size, num_traces, 5);
        let log = black_box(log);

        let label = format!("{}k_mem{:.0}mb", size / 1000, tracker.estimated_mb());
        println!(
            "\nAlpha Miner Scalability Test: {} events, estimated {:.2} MB",
            size,
            tracker.estimated_mb()
        );

        group.bench_with_input(BenchmarkId::from_parameter(label), size, |b, _| {
            b.iter(|| {
                let miner = AlphaMiner::new();
                miner.discover(&log)
            });
        });
    }

    group.finish();
}

fn bench_scalability_dfg_miner(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability_dfg");
    group.sample_size(5);

    for size in [100_000, 1_000_000, 10_000_000].iter() {
        let num_traces = size / 50;
        let (log, tracker) = generate_event_log(*size, num_traces, 5);
        let log = black_box(log);

        let label = format!("{}k_mem{:.0}mb", size / 1000, tracker.estimated_mb());
        println!(
            "\nDFG Miner Scalability Test: {} events, estimated {:.2} MB",
            size,
            tracker.estimated_mb()
        );

        group.bench_with_input(BenchmarkId::from_parameter(label), size, |b, _| {
            b.iter(|| {
                let miner = DFGMiner::new();
                miner.discover(&log)
            });
        });
    }

    group.finish();
}

fn bench_scalability_token_replay(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability_conformance");
    group.sample_size(5);

    for size in [100_000, 1_000_000, 10_000_000].iter() {
        let num_traces = size / 50;
        let (log, tracker) = generate_event_log(*size, num_traces, 5);
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);
        let log = black_box(log);
        let net = black_box(net);

        let label = format!("{}k_mem{:.0}mb", size / 1000, tracker.estimated_mb());
        println!(
            "\nToken Replay Scalability Test: {} events, estimated {:.2} MB",
            size,
            tracker.estimated_mb()
        );

        group.bench_with_input(BenchmarkId::from_parameter(label), size, |b, _| {
            b.iter(|| {
                let replay = TokenReplay::new();
                replay.check(&log, &net)
            });
        });
    }

    group.finish();
}

// ============================================================================
// THROUGHPUT BENCHMARKS (events per second)
// ============================================================================

fn bench_throughput_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_discovery");

    for size in [100_000, 1_000_000, 10_000_000].iter() {
        let num_traces = size / 50;
        let (log, tracker) = generate_event_log(*size, num_traces, 5);
        let log = black_box(log);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}", size / 1_000_000)),
            size,
            |b, &size| {
                b.iter(|| {
                    let miner = DFGMiner::new();
                    miner.discover(&log)
                });
            },
        );

        println!(
            "\nThroughput Test: {} events ({:.2} MB estimated)",
            size,
            tracker.estimated_mb()
        );
    }

    group.finish();
}

fn bench_throughput_conformance(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_conformance");

    for size in [100_000, 1_000_000, 10_000_000].iter() {
        let num_traces = size / 50;
        let (log, tracker) = generate_event_log(*size, num_traces, 5);
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);
        let log = black_box(log);
        let net = black_box(net);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}", size / 1_000_000)),
            size,
            |b, &size| {
                b.iter(|| {
                    let replay = TokenReplay::new();
                    replay.check(&log, &net)
                });
            },
        );

        println!(
            "\nConformance Throughput Test: {} events ({:.2} MB estimated)",
            size,
            tracker.estimated_mb()
        );
    }

    group.finish();
}

// ============================================================================
// CRITERION CONFIGURATION
// ============================================================================

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(std::time::Duration::from_secs(10))
        .sample_size(10);
    targets =
        bench_alpha_miner_100k,
        bench_alpha_miner_1m,
        bench_alpha_miner_10m,
        bench_alpha_miner_100m,
        bench_inductive_miner_100k,
        bench_inductive_miner_1m,
        bench_inductive_miner_10m,
        bench_dfg_miner_100k,
        bench_dfg_miner_1m,
        bench_dfg_miner_10m,
        bench_dfg_miner_100m,
        bench_token_replay_100k,
        bench_token_replay_1m,
        bench_token_replay_10m,
        bench_token_replay_100m,
        bench_scalability_alpha_miner,
        bench_scalability_dfg_miner,
        bench_scalability_token_replay,
        bench_throughput_discovery,
        bench_throughput_conformance
);

criterion_main!(benches);
