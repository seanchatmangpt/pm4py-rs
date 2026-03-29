//! Process Discovery Algorithm Performance Benchmarks
//!
//! Comprehensive benchmarks for:
//! - Alpha Miner (all scales)
//! - Inductive Miner (all scales)
//! - Heuristic Miner (all scales)
//! - DFG Miner (all scales)
//!
//! Runs at: 100K, 1M, 10M events
//! Measures: execution time, throughput, memory efficiency

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pm4py::discovery::{AlphaMiner, DFGMiner, InductiveMiner};
use pm4py::log::{Event, EventLog, Trace};

/// Generate simple linear event log (good for Alpha Miner)
fn generate_linear_log(num_events: usize, num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let activities = vec!["A", "B", "C", "D", "E"];
    let events_per_trace = (num_events / num_traces).max(1);
    let base_time = Utc::now();

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("trace_{:08}", trace_id));

        for event_idx in 0..events_per_trace {
            let activity_idx = event_idx % activities.len();
            let timestamp = base_time
                + chrono::Duration::seconds((trace_id * events_per_trace + event_idx) as i64);
            let event = Event::new(activities[activity_idx], timestamp)
                .with_resource(format!("res_{}", trace_id % 5));
            trace.add_event(event);
        }

        log.add_trace(trace);
    }

    log
}

/// Generate parallel event log (good for all miners)
fn generate_parallel_log(num_events: usize, num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("parallel_{:08}", trace_id));
        let events_per_trace = (num_events / num_traces).max(1);

        for event_idx in 0..events_per_trace {
            let activity = match event_idx % 7 {
                0 => "Start",
                1 => "ParallelA",
                2 => "ParallelB",
                3 => "ParallelC",
                4 => "Join",
                5 => "Process",
                6 => "End",
                _ => "Unknown",
            };

            let timestamp = base_time
                + chrono::Duration::seconds((trace_id * events_per_trace + event_idx) as i64);
            let event = Event::new(activity, timestamp)
                .with_resource(format!("specialist_{}", event_idx % 4));
            trace.add_event(event);
        }

        log.add_trace(trace);
    }

    log
}

/// Generate complex log with loops and rework
fn generate_loop_log(num_events: usize, num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("loop_{:08}", trace_id));
        let events_per_trace = (num_events / num_traces).max(1);
        let mut current_time = base_time;

        for event_idx in 0..events_per_trace {
            let activity = if (event_idx % 5) == 4 && event_idx > 0 {
                "Rework" // Create loops
            } else {
                match (event_idx / 5) % 4 {
                    0 => "Init",
                    1 => "Check",
                    2 => "Process",
                    _ => "Complete",
                }
            };

            let event =
                Event::new(activity, current_time).with_resource(format!("agent_{}", trace_id % 8));
            trace.add_event(event);
            current_time = current_time + chrono::Duration::seconds(3);
        }

        log.add_trace(trace);
    }

    log
}

// ============================================================================
// ALPHA MINER
// ============================================================================

fn bench_alpha_100k_linear(c: &mut Criterion) {
    let log = black_box(generate_linear_log(100_000, 2_000));

    c.bench_function("discovery_alpha_100k_linear", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_alpha_100k_parallel(c: &mut Criterion) {
    let log = black_box(generate_parallel_log(100_000, 2_000));

    c.bench_function("discovery_alpha_100k_parallel", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_alpha_1m_linear(c: &mut Criterion) {
    let log = black_box(generate_linear_log(1_000_000, 10_000));

    c.bench_function("discovery_alpha_1m_linear", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_alpha_1m_parallel(c: &mut Criterion) {
    let log = black_box(generate_parallel_log(1_000_000, 10_000));

    c.bench_function("discovery_alpha_1m_parallel", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_alpha_10m_linear(c: &mut Criterion) {
    let log = black_box(generate_linear_log(10_000_000, 50_000));

    c.bench_function("discovery_alpha_10m_linear", |b| {
        b.iter(|| {
            let miner = AlphaMiner::new();
            miner.discover(&log)
        });
    });
}

// ============================================================================
// INDUCTIVE MINER
// ============================================================================

fn bench_inductive_100k_loop(c: &mut Criterion) {
    let log = black_box(generate_loop_log(100_000, 2_000));

    c.bench_function("discovery_inductive_100k_loop", |b| {
        b.iter(|| {
            let miner = InductiveMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_inductive_100k_parallel(c: &mut Criterion) {
    let log = black_box(generate_parallel_log(100_000, 2_000));

    c.bench_function("discovery_inductive_100k_parallel", |b| {
        b.iter(|| {
            let miner = InductiveMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_inductive_1m_loop(c: &mut Criterion) {
    let log = black_box(generate_loop_log(1_000_000, 10_000));

    c.bench_function("discovery_inductive_1m_loop", |b| {
        b.iter(|| {
            let miner = InductiveMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_inductive_1m_parallel(c: &mut Criterion) {
    let log = black_box(generate_parallel_log(1_000_000, 10_000));

    c.bench_function("discovery_inductive_1m_parallel", |b| {
        b.iter(|| {
            let miner = InductiveMiner::new();
            miner.discover(&log)
        });
    });
}

// ============================================================================
// DFG MINER
// ============================================================================

fn bench_dfg_100k_linear(c: &mut Criterion) {
    let log = black_box(generate_linear_log(100_000, 2_000));

    c.bench_function("discovery_dfg_100k_linear", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_100k_parallel(c: &mut Criterion) {
    let log = black_box(generate_parallel_log(100_000, 2_000));

    c.bench_function("discovery_dfg_100k_parallel", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_100k_loop(c: &mut Criterion) {
    let log = black_box(generate_loop_log(100_000, 2_000));

    c.bench_function("discovery_dfg_100k_loop", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_1m_linear(c: &mut Criterion) {
    let log = black_box(generate_linear_log(1_000_000, 10_000));

    c.bench_function("discovery_dfg_1m_linear", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_1m_parallel(c: &mut Criterion) {
    let log = black_box(generate_parallel_log(1_000_000, 10_000));

    c.bench_function("discovery_dfg_1m_parallel", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_1m_loop(c: &mut Criterion) {
    let log = black_box(generate_loop_log(1_000_000, 10_000));

    c.bench_function("discovery_dfg_1m_loop", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_10m_linear(c: &mut Criterion) {
    let log = black_box(generate_linear_log(10_000_000, 50_000));

    c.bench_function("discovery_dfg_10m_linear", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

fn bench_dfg_10m_parallel(c: &mut Criterion) {
    let log = black_box(generate_parallel_log(10_000_000, 50_000));

    c.bench_function("discovery_dfg_10m_parallel", |b| {
        b.iter(|| {
            let miner = DFGMiner::new();
            miner.discover(&log)
        });
    });
}

// ============================================================================
// SCALABILITY COMPARISON
// ============================================================================

fn bench_discovery_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("discovery_scalability");
    group.sample_size(5);

    for size in &[100_000, 1_000_000] {
        let num_traces = size / 50;

        // Alpha
        let log = black_box(generate_linear_log(*size, num_traces));
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("alpha", format!("{}k_events", size / 1000)),
            size,
            |b, _| {
                b.iter(|| {
                    let miner = AlphaMiner::new();
                    miner.discover(&log)
                });
            },
        );

        // Inductive
        let log = black_box(generate_loop_log(*size, num_traces));
        group.bench_with_input(
            BenchmarkId::new("inductive", format!("{}k_events", size / 1000)),
            size,
            |b, _| {
                b.iter(|| {
                    let miner = InductiveMiner::new();
                    miner.discover(&log)
                });
            },
        );

        // DFG
        let log = black_box(generate_parallel_log(*size, num_traces));
        group.bench_with_input(
            BenchmarkId::new("dfg", format!("{}k_events", size / 1000)),
            size,
            |b, _| {
                b.iter(|| {
                    let miner = DFGMiner::new();
                    miner.discover(&log)
                });
            },
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
        .sample_size(5);
    targets =
        bench_alpha_100k_linear,
        bench_alpha_100k_parallel,
        bench_alpha_1m_linear,
        bench_alpha_1m_parallel,
        bench_alpha_10m_linear,
        bench_inductive_100k_loop,
        bench_inductive_100k_parallel,
        bench_inductive_1m_loop,
        bench_inductive_1m_parallel,
        bench_dfg_100k_linear,
        bench_dfg_100k_parallel,
        bench_dfg_100k_loop,
        bench_dfg_1m_linear,
        bench_dfg_1m_parallel,
        bench_dfg_1m_loop,
        bench_dfg_10m_linear,
        bench_dfg_10m_parallel,
        bench_discovery_scalability
);

criterion_main!(benches);
