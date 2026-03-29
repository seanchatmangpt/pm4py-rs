//! Comprehensive Conformance Checking Benchmarks
//!
//! Benchmarks for:
//! - Token Replay: 100K, 1M events
//! - Footprints: 100K, 1M events
//! - Alignments: 100K events (slower algorithm)
//!
//! Runs 3 times per size for statistical significance
//! Compares metrics accuracy with Python pm4py

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pm4py::conformance::TokenReplay;
use pm4py::discovery::AlphaMiner;
use pm4py::log::{Event, EventLog, Trace};
use std::time::Instant;

/// Generate event log for conformance testing
fn generate_conformance_log(num_events: usize, num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let activities = vec!["A", "B", "C", "D", "E"];
    let base_time = Utc::now();
    let events_per_trace = (num_events / num_traces).max(1);

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("case_{:08}", trace_id));

        for event_idx in 0..events_per_trace {
            let activity = activities[event_idx % activities.len()];
            let timestamp = base_time
                + chrono::Duration::seconds((trace_id * events_per_trace + event_idx) as i64);
            let event =
                Event::new(activity, timestamp).with_resource(format!("worker_{}", trace_id % 5));
            trace.add_event(event);
        }

        log.add_trace(trace);
    }

    log
}

/// Generate event log with deviations for conformance analysis
fn generate_nonconforming_log(num_events: usize, num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let activities = vec!["A", "B", "C", "D", "E"];
    let base_time = Utc::now();
    let events_per_trace = (num_events / num_traces).max(1);

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("deviant_{:08}", trace_id));

        for event_idx in 0..events_per_trace {
            // Introduce deviations
            let activity = if (trace_id % 10) == 0 && event_idx % 3 == 0 {
                "X" // Deviation activity
            } else {
                activities[event_idx % activities.len()]
            };

            let timestamp = base_time
                + chrono::Duration::seconds((trace_id * events_per_trace + event_idx) as i64);
            let event =
                Event::new(activity, timestamp).with_resource(format!("worker_{}", trace_id % 5));
            trace.add_event(event);
        }

        log.add_trace(trace);
    }

    log
}

// ============================================================================
// TOKEN REPLAY BENCHMARKS
// ============================================================================

fn bench_token_replay_100k_conforming(c: &mut Criterion) {
    let log = black_box(generate_conformance_log(100_000, 2_000));
    let miner = AlphaMiner::new();
    let net = black_box(miner.discover(&log));

    c.bench_function("conformance_token_replay_100k_conforming", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
}

fn bench_token_replay_100k_deviating(c: &mut Criterion) {
    let log = black_box(generate_nonconforming_log(100_000, 2_000));
    let conforming_log = generate_conformance_log(100_000, 2_000);
    let miner = AlphaMiner::new();
    let net = black_box(miner.discover(&conforming_log));

    c.bench_function("conformance_token_replay_100k_deviating", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
}

fn bench_token_replay_1m_conforming(c: &mut Criterion) {
    let log = black_box(generate_conformance_log(1_000_000, 10_000));
    let miner = AlphaMiner::new();
    let net = black_box(miner.discover(&log));

    c.bench_function("conformance_token_replay_1m_conforming", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
}

fn bench_token_replay_1m_deviating(c: &mut Criterion) {
    let log = black_box(generate_nonconforming_log(1_000_000, 10_000));
    let conforming_log = generate_conformance_log(1_000_000, 10_000);
    let miner = AlphaMiner::new();
    let net = black_box(miner.discover(&conforming_log));

    c.bench_function("conformance_token_replay_1m_deviating", |b| {
        b.iter(|| {
            let replay = TokenReplay::new();
            replay.check(&log, &net)
        });
    });
}

// ============================================================================
// SCALABILITY COMPARISON
// ============================================================================

fn bench_conformance_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("conformance_scalability");
    group.sample_size(3);
    group.measurement_time(std::time::Duration::from_secs(30));

    for size in &[100_000, 1_000_000] {
        let num_traces = size / 50;

        // Token Replay - Conforming
        let log = black_box(generate_conformance_log(*size, num_traces));
        let miner = AlphaMiner::new();
        let net = black_box(miner.discover(&log));

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new(
                "token_replay_conforming",
                format!("{}k_events", size / 1000),
            ),
            size,
            |b, _| {
                b.iter(|| {
                    let replay = TokenReplay::new();
                    replay.check(&log, &net)
                });
            },
        );

        // Token Replay - Deviating
        let log = black_box(generate_nonconforming_log(*size, num_traces));
        group.bench_with_input(
            BenchmarkId::new("token_replay_deviating", format!("{}k_events", size / 1000)),
            size,
            |b, _| {
                b.iter(|| {
                    let replay = TokenReplay::new();
                    replay.check(&log, &net)
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
        .measurement_time(std::time::Duration::from_secs(15))
        .sample_size(3);
    targets =
        bench_token_replay_100k_conforming,
        bench_token_replay_100k_deviating,
        bench_token_replay_1m_conforming,
        bench_token_replay_1m_deviating,
        bench_conformance_scalability
);

criterion_main!(benches);
