use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

fn bench_alpha_miner(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_miner");
    group.sample_size(10);

    let log = generate_event_log(10_000);

    group.bench_function("10k_events", |b| {
        b.iter(|| {
            let miner = pm4py::discovery::AlphaMiner::new();
            miner.discover(&log)
        });
    });

    group.finish();
}

fn bench_dfg_miner(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfg_miner");
    group.sample_size(10);

    let log = generate_event_log(100_000);

    group.bench_function("100k_events", |b| {
        b.iter(|| {
            let miner = pm4py::discovery::DFGMiner::new();
            miner.discover(&log)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_alpha_miner, bench_dfg_miner);
criterion_main!(benches);
