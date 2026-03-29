use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pm4py::statistics::log_statistics;
use pm4py::{Event, EventLog, Trace};

fn generate_event_log(num_events: usize) -> EventLog {
    let mut log = EventLog::new();
    let activities = vec!["A", "B", "C", "D", "E"];
    let num_traces = (num_events / 50).max(1);

    for trace_id in 0..num_traces {
        let mut trace = Trace::new(format!("case_{}", trace_id));
        let trace_length = ((num_events / num_traces).min(200)).max(5);
        let now = Utc::now();

        for i in 0..trace_length {
            let activity = activities[i % activities.len()];
            trace.add_event(Event::new(activity, now));
        }

        log.add_trace(trace);
    }

    log
}

fn bench_log_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("log_statistics");
    group.sample_size(10);

    for events in [1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_events", events)),
            events,
            |b, &events| {
                let log = generate_event_log(events);
                b.iter(|| black_box(log_statistics(&black_box(log.clone()))));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_log_statistics);
criterion_main!(benches);
