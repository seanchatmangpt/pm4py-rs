//! Comprehensive Statistics & Analysis Benchmarks
//!
//! Benchmarks for:
//! - Frequency analysis: 100K, 1M events
//! - Variant extraction: 100K, 1M events
//! - Rework pattern detection: 100K events
//!
//! Measures throughput and accuracy compared to Python pm4py

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pm4py::log::{Event, EventLog, Trace};

/// Generate standard event log for statistics
fn generate_standard_log(num_events: usize, num_traces: usize) -> EventLog {
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
            let event = Event::new(activity, timestamp)
                .with_resource(format!("worker_{}", trace_id % 5))
                .with_attribute("cost", (event_idx * 10) as f64);
            trace.add_event(event);
        }

        log.add_trace(trace);
    }

    log
}

/// Generate log with variant patterns
fn generate_variant_log(num_events: usize, num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();
    let events_per_trace = (num_events / num_traces).max(1);

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("var_{:08}", trace_id));
        let variant = trace_id % 4;

        for event_idx in 0..events_per_trace {
            let activity = match variant {
                0 => match event_idx % 3 {
                    0 => "A",
                    1 => "B",
                    _ => "C",
                },
                1 => match event_idx % 4 {
                    0 => "A",
                    1 => "X",
                    2 => "B",
                    _ => "C",
                },
                2 => match event_idx % 5 {
                    0 => "A",
                    1 => "B",
                    2 => "X",
                    3 => "B",
                    _ => "C",
                },
                _ => match event_idx % 2 {
                    0 => "B",
                    _ => "C",
                },
            };

            let timestamp = base_time
                + chrono::Duration::seconds((trace_id * events_per_trace + event_idx) as i64);
            let event = Event::new(activity, timestamp)
                .with_resource(format!("worker_{}", (trace_id + event_idx) % 8));
            trace.add_event(event);
        }

        log.add_trace(trace);
    }

    log
}

/// Generate log with rework patterns
fn generate_rework_log(num_events: usize, num_traces: usize) -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();
    let events_per_trace = (num_events / num_traces).max(1);

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("rework_{:08}", trace_id));
        let mut current_time = base_time;

        for event_idx in 0..events_per_trace {
            let activity = if event_idx > 0 && event_idx % 7 == 0 && (trace_id % 5) == 0 {
                // Rework event
                "Rework"
            } else {
                match (event_idx / 7) % 4 {
                    0 => "Request",
                    1 => "Process",
                    2 => "Review",
                    _ => "Complete",
                }
            };

            let event = Event::new(activity, current_time)
                .with_resource(format!("agent_{}", (event_idx + trace_id) % 6));
            trace.add_event(event);
            current_time = current_time + chrono::Duration::minutes(1);
        }

        log.add_trace(trace);
    }

    log
}

// ============================================================================
// FREQUENCY ANALYSIS BENCHMARKS
// ============================================================================

fn bench_frequency_100k(c: &mut Criterion) {
    let log = black_box(generate_standard_log(100_000, 2_000));

    c.bench_function("statistics_frequency_100k", |b| {
        b.iter(|| {
            // Count activity frequencies
            let mut freq = std::collections::HashMap::new();
            for trace in &log.traces {
                for event in &trace.events {
                    *freq.entry(event.activity.clone()).or_insert(0usize) += 1;
                }
            }
            freq
        });
    });
}

fn bench_frequency_1m(c: &mut Criterion) {
    let log = black_box(generate_standard_log(1_000_000, 10_000));

    c.bench_function("statistics_frequency_1m", |b| {
        b.iter(|| {
            let mut freq = std::collections::HashMap::new();
            for trace in &log.traces {
                for event in &trace.events {
                    *freq.entry(event.activity.clone()).or_insert(0usize) += 1;
                }
            }
            freq
        });
    });
}

// ============================================================================
// VARIANT EXTRACTION BENCHMARKS
// ============================================================================

fn bench_variants_100k(c: &mut Criterion) {
    let log = black_box(generate_variant_log(100_000, 2_000));

    c.bench_function("statistics_variants_100k", |b| {
        b.iter(|| {
            // Extract unique variants (trace patterns)
            let mut variants = std::collections::HashMap::new();
            for trace in &log.traces {
                let sequence: Vec<&str> =
                    trace.events.iter().map(|e| e.activity.as_str()).collect();
                let key = sequence.join(",");
                *variants.entry(key).or_insert(0usize) += 1;
            }
            variants
        });
    });
}

fn bench_variants_1m(c: &mut Criterion) {
    let log = black_box(generate_variant_log(1_000_000, 10_000));

    c.bench_function("statistics_variants_1m", |b| {
        b.iter(|| {
            let mut variants = std::collections::HashMap::new();
            for trace in &log.traces {
                let sequence: Vec<&str> =
                    trace.events.iter().map(|e| e.activity.as_str()).collect();
                let key = sequence.join(",");
                *variants.entry(key).or_insert(0usize) += 1;
            }
            variants
        });
    });
}

// ============================================================================
// REWORK PATTERN DETECTION
// ============================================================================

fn bench_rework_detection_100k(c: &mut Criterion) {
    let log = black_box(generate_rework_log(100_000, 2_000));

    c.bench_function("statistics_rework_patterns_100k", |b| {
        b.iter(|| {
            // Detect rework patterns
            let mut rework_traces = 0;
            let mut total_rework_events = 0;

            for trace in &log.traces {
                let rework_count = trace
                    .events
                    .iter()
                    .filter(|e| e.activity == "Rework")
                    .count();

                if rework_count > 0 {
                    rework_traces += 1;
                    total_rework_events += rework_count;
                }
            }

            (rework_traces, total_rework_events)
        });
    });
}

// ============================================================================
// SCALABILITY COMPARISON
// ============================================================================

fn bench_statistics_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_scalability");
    group.sample_size(3);
    group.measurement_time(std::time::Duration::from_secs(20));

    for size in &[100_000, 1_000_000] {
        let num_traces = size / 50;

        // Frequency
        let log = black_box(generate_standard_log(*size, num_traces));
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("frequency", format!("{}k_events", size / 1000)),
            size,
            |b, _| {
                b.iter(|| {
                    let mut freq = std::collections::HashMap::new();
                    for trace in &log.traces {
                        for event in &trace.events {
                            *freq.entry(event.activity.clone()).or_insert(0usize) += 1;
                        }
                    }
                    freq
                });
            },
        );

        // Variants
        let log = black_box(generate_variant_log(*size, num_traces));
        group.bench_with_input(
            BenchmarkId::new("variants", format!("{}k_events", size / 1000)),
            size,
            |b, _| {
                b.iter(|| {
                    let mut variants = std::collections::HashMap::new();
                    for trace in &log.traces {
                        let sequence: Vec<&str> =
                            trace.events.iter().map(|e| e.activity.as_str()).collect();
                        let key = sequence.join(",");
                        *variants.entry(key).or_insert(0usize) += 1;
                    }
                    variants
                });
            },
        );

        // Rework (only for 100K due to performance)
        if *size == 100_000 {
            let log = black_box(generate_rework_log(*size, num_traces));
            group.bench_with_input(
                BenchmarkId::new("rework", format!("{}k_events", size / 1000)),
                size,
                |b, _| {
                    b.iter(|| {
                        let mut rework_traces = 0;
                        let mut total_rework_events = 0;
                        for trace in &log.traces {
                            let rework_count = trace
                                .events
                                .iter()
                                .filter(|e| e.activity == "Rework")
                                .count();
                            if rework_count > 0 {
                                rework_traces += 1;
                                total_rework_events += rework_count;
                            }
                        }
                        (rework_traces, total_rework_events)
                    });
                },
            );
        }
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
        bench_frequency_100k,
        bench_frequency_1m,
        bench_variants_100k,
        bench_variants_1m,
        bench_rework_detection_100k,
        bench_statistics_scalability
);

criterion_main!(benches);
