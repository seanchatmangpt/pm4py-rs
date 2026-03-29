# Performance Tuning Guide

**Optimize PM4Py Rust for speed, memory, and accuracy on large datasets**

---

## Table of Contents

1. [Benchmarking](#benchmarking)
2. [Discovery Optimization](#discovery-optimization)
3. [Conformance Optimization](#conformance-optimization)
4. [Statistics Optimization](#statistics-optimization)
5. [Memory Optimization](#memory-optimization)
6. [Real-World Optimizations](#real-world-optimizations)

---

## Benchmarking

### Method 1: Manual Timing

```rust
use std::time::Instant;

fn benchmark_discovery(log: &EventLog) {
    println!("=== Discovery Benchmarks ===\n");

    let algorithms = vec![
        ("Alpha Miner", Box::new(AlphaMiner::new())),
        ("Heuristic Miner", Box::new(HeuristicMiner::new())),
        ("Inductive Miner", Box::new(InductiveMiner::new())),
    ];

    for (name, miner) in algorithms {
        let start = Instant::now();
        let _model = miner.discover(log);
        let elapsed = start.elapsed();

        println!("{}: {:?}", name, elapsed);
    }
}
```

### Method 2: Using Criterion (Benchmark Library)

Create `benches/my_benchmark.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pm4py::discovery::AlphaMiner;

fn create_bench_log(size: usize) -> pm4py::log::EventLog {
    // Create log of specified size
    let mut log = pm4py::log::EventLog::new();
    // ... populate ...
    log
}

fn benchmark_alpha(c: &mut Criterion) {
    c.bench_function("alpha_1k", |b| {
        let log = black_box(create_bench_log(1000));
        b.iter(|| {
            AlphaMiner::new().discover(&log)
        });
    });
}

criterion_group!(benches, benchmark_alpha);
criterion_main!(benches);
```

Run:
```bash
cargo bench --bench my_benchmark
```

### Method 3: Profiling with Perf (Linux)

```bash
# Build with debug symbols
cargo build --release

# Profile
perf record -g ./target/release/my_app
perf report

# Flamegraph visualization
cargo install flamegraph
cargo flamegraph --bin my_app
```

---

## Discovery Optimization

### Optimization 1: Algorithm Selection

**Speed ranking:**
```
Alpha Miner        1 ms per 1K events
Heuristic Miner    3 ms per 1K events
Inductive Miner   50 ms per 1K events
```

```rust
fn optimize_for_speed(log: &EventLog) {
    // For max speed (loss of quality):
    let model = AlphaMiner::new().discover(log);  // ⚡⚡⚡

    // For balance:
    let model = HeuristicMiner::new().discover(log);  // ⚡⚡

    // For quality (loss of speed):
    let model = InductiveMiner::new().discover(log);  // ⚡
}
```

### Optimization 2: Pre-filtering

```rust
fn optimize_with_filtering(log: &EventLog) {
    // Filter infrequent activities (threshold = 0.95)
    let filtered = log.filter_activities_by_threshold(0.95);

    println!("Before: {} activities", log.statistics().num_activities);
    println!("After: {} activities", filtered.statistics().num_activities);
    println!("Result: {} places in model",
             AlphaMiner::new().discover(&filtered).places.len());

    // Rule of thumb:
    // Each activity removed → discovery ~10% faster
}
```

### Optimization 3: Sampling

```rust
fn optimize_with_sampling(log: &EventLog) {
    let original_size = log.num_events();

    // Sample to 10K events
    let sampled = log.sample(10000);

    println!("Before: {} events", original_size);
    println!("After: {} events", sampled.num_events());

    let start = Instant::now();
    let model = AlphaMiner::new().discover(&sampled);
    println!("Discovery time: {:?}", start.elapsed());

    // Rule of thumb:
    // log.sample(n) gives proportional speedup
    // 100K → 10K = ~10x faster
}
```

### Optimization 4: Parallel Processing

```rust
use rayon::prelude::*;

fn discover_multiple_logs_parallel(logs: Vec<EventLog>) {
    let models: Vec<_> = logs
        .par_iter()  // Parallel iterator
        .map(|log| {
            AlphaMiner::new().discover(log)
        })
        .collect();

    println!("Discovered {} models in parallel", models.len());
}
```

### Optimization 5: Caching Results

```rust
use std::collections::HashMap;

fn discover_with_cache(log: &EventLog, cache: &mut HashMap<String, PetriNet>) {
    let key = format!("alpha_{}", log.traces().len());

    if let Some(cached) = cache.get(&key) {
        println!("Cache hit!");
        return cached.clone();
    }

    println!("Cache miss, discovering...");
    let model = AlphaMiner::new().discover(log);
    cache.insert(key, model.clone());
    model
}
```

---

## Conformance Optimization

### Optimization 1: Faster Conformance Methods

```rust
fn conformance_speed_comparison(log: &EventLog, model: &PetriNet) {
    use pm4py::conformance::{TokenReplay, DFGConformance};

    let start = Instant::now();
    let _results1 = TokenReplay::new().replay(log, model);
    println!("Token Replay: {:?}", start.elapsed());

    let start = Instant::now();
    let _results2 = DFGConformance::new().check(log, model);
    println!("DFG Check: {:?}", start.elapsed());

    // DFG is ~10x faster but less accurate
}
```

### Optimization 2: Sampling for Conformance

```rust
fn conformance_with_sampling(log: &EventLog, model: &PetriNet) {
    // Test only 10% of traces
    let sample_size = (log.traces().len() / 10).max(100);
    let sampled = log.sample_traces(sample_size);

    let start = Instant::now();
    let results = TokenReplay::new().replay(&sampled, model);
    println!("Conformance (10% sample): {:?}", start.elapsed());

    // Extrapolate to full log
    let avg_fitness: f64 = results.iter().map(|r| r.fitness).sum::<f64>()
                          / results.len() as f64;
    println!("Estimated average fitness: {:.2}", avg_fitness);
}
```

### Optimization 3: Batch Processing

```rust
fn conformance_batched(log: &EventLog, model: &PetriNet) {
    let batch_size = 1000;
    let mut all_results = Vec::new();

    for batch in log.traces().chunks(batch_size) {
        let batch_log = EventLog::from_traces(batch.to_vec());
        let results = TokenReplay::new().replay(&batch_log, model);
        all_results.extend(results);
    }

    println!("Processed {} traces in batches", log.traces().len());
}
```

---

## Statistics Optimization

### Optimization 1: Incremental Updates

```rust
fn optimize_statistics_calculation(log: &EventLog) {
    // Instead of recalculating all stats...
    let stats = log.statistics();

    // Store and update incrementally
    struct CachedStats {
        num_traces: usize,
        num_events: usize,
        num_activities: usize,
    }

    let mut cached = CachedStats {
        num_traces: stats.num_traces,
        num_events: stats.num_events,
        num_activities: stats.num_activities,
    };

    // When adding new trace:
    // cached.num_traces += 1;
    // cached.num_events += new_trace.len();
    // ... update activities ...
}
```

### Optimization 2: Lazy Evaluation

```rust
fn optimize_with_lazy_evaluation(log: &EventLog) {
    // Don't compute all stats upfront
    // Only compute what you need

    let num_traces = log.traces().len();
    if num_traces > 100_000 {
        println!("Large log - computing sample statistics only");
        let sample = log.sample(1000);
        let stats = sample.statistics();
        // Extrapolate
    } else {
        let stats = log.statistics();
        // Use all
    }
}
```

---

## Memory Optimization

### Optimization 1: Stream Processing

```rust
fn process_streaming(log: &EventLog) -> Result<PetriNet> {
    let batch_size = 5000;

    let mut accumulated_model = None;

    for batch in log.traces().chunks(batch_size) {
        let batch_log = EventLog::from_traces(batch.to_vec());
        let batch_model = AlphaMiner::new().discover(&batch_log);

        // Merge with accumulated model
        accumulated_model = Some(merge_models(
            accumulated_model,
            batch_model
        ));

        // Batch log dropped here, freeing memory
    }

    Ok(accumulated_model.unwrap())
}

fn merge_models(m1: Option<PetriNet>, m2: PetriNet) -> PetriNet {
    match m1 {
        None => m2,
        Some(model) => model.merge(m2),
    }
}
```

### Optimization 2: Data Structure Selection

```rust
// Use more efficient data structures

use std::collections::HashMap;
use indexmap::IndexMap;

fn optimize_data_structures() {
    // For ordered access: IndexMap (not HashMap)
    let ordered = IndexMap::new();

    // For sparse data: use indices instead of strings
    let mut activities = vec![];
    let activity_to_id = HashMap::new();
    // Now store activity IDs (u32) instead of String

    // Estimates:
    // String "ActivityName" → 28 bytes + allocation
    // u32 ID → 4 bytes
    // Savings: ~7x reduction for large logs
}
```

### Optimization 3: Pooling and Reuse

```rust
use object_pool::ObjectPool;

fn optimize_with_pooling() {
    // Create pool of pre-allocated Event objects
    let pool = ObjectPool::new(|| Event::new("", Utc::now()));

    // Reuse instead of allocate
    for _ in 0..10000 {
        let mut event = pool.pull(|| Event::new("", Utc::now()));
        // Use event...
        // Dropped back into pool automatically
    }
}
```

### Optimization 4: Memory Profiling

```bash
# With heaptrack (Linux)
heaptrack ./target/release/my_app
heaptrack_gui heaptrack.my_app.*.gz

# With valgrind
valgrind --tool=massif ./target/release/my_app
```

---

## Real-World Optimizations

### Scenario 1: Real-time Processing of Incoming Events

```rust
use tokio::sync::mpsc;

async fn real_time_processing() {
    let (tx, mut rx) = mpsc::channel(100);

    // Event producer
    tokio::spawn(async move {
        for event in incoming_events() {
            tx.send(event).await.ok();
        }
    });

    // Event consumer with batching
    let mut batch = Vec::new();
    let batch_size = 100;

    while let Some(event) = rx.recv().await {
        batch.push(event);

        if batch.len() >= batch_size {
            // Process batch
            let log = EventLog::from_events(batch.drain(..));
            discover_incrementally(&log).await;
        }
    }
}
```

### Scenario 2: Multi-tenant System

```rust
use dashmap::DashMap;

struct TenantCache {
    models: DashMap<String, PetriNet>,
    stats: DashMap<String, Statistics>,
}

impl TenantCache {
    fn get_or_discover(&self, tenant_id: &str, log: &EventLog) -> PetriNet {
        if let Some(model) = self.models.get(tenant_id) {
            return model.value().clone();
        }

        // Discover and cache
        let model = AlphaMiner::new().discover(log);
        self.models.insert(tenant_id.to_string(), model.clone());
        model
    }
}
```

### Scenario 3: Large-Scale Batch Processing

```rust
use std::fs;
use rayon::prelude::*;

fn batch_process_files(dir: &str) {
    let files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "xes"))
        .collect();

    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            let log = read_xes(path).unwrap();
            let model = AlphaMiner::new().discover(&log);
            (path.clone(), model)
        })
        .collect();

    println!("Processed {} files", results.len());
}
```

---

## Optimization Checklist

### For Discovery

- [ ] Choose appropriate algorithm (Alpha for speed, Inductive for quality)
- [ ] Pre-filter activities by threshold
- [ ] Sample large logs (>100K events)
- [ ] Use parallel processing for multiple logs
- [ ] Cache results if reprocessing same logs

### For Conformance

- [ ] Use simpler method (DFG) if speed critical
- [ ] Sample traces (10-20%)
- [ ] Batch process if memory limited
- [ ] Run in parallel for multiple models

### For Memory

- [ ] Process in chunks/batches
- [ ] Use iterators, not `.collect()`
- [ ] Pool reusable objects
- [ ] Profile with heaptrack/valgrind

### For Overall

- [ ] Benchmark before and after optimization
- [ ] Profile to find actual bottlenecks
- [ ] Measure with realistic data sizes
- [ ] Document optimization rationale

---

## Example: Complete Optimized Pipeline

```rust
use pm4py::discovery::AlphaMiner;
use pm4py::conformance::TokenReplay;
use pm4py::io::xes::read_xes;
use std::time::Instant;

fn optimized_pipeline(log_path: &str) -> Result<()> {
    let start = Instant::now();

    // 1. Load (streaming)
    println!("Loading log...");
    let mut log = read_xes(log_path)?;

    // 2. Filter (remove noise)
    println!("Filtering activities...");
    log = log.filter_activities_by_threshold(0.95);

    // 3. Sample if large
    let original_size = log.num_events();
    if original_size > 50_000 {
        println!("Sampling to 50K events...");
        log = log.sample(50_000);
    }

    // 4. Discover with fast algorithm
    println!("Discovering model...");
    let model = AlphaMiner::new().discover(&log);

    // 5. Check conformance on sample
    println!("Checking conformance...");
    let sample = log.sample(log.traces().len().min(1000));
    let results = TokenReplay::new().replay(&sample, &model);

    let fitness: f64 = results.iter()
        .map(|r| r.fitness)
        .sum::<f64>() / results.len() as f64;

    println!("Results:");
    println!("  Original events: {}", original_size);
    println!("  Final events: {}", log.num_events());
    println!("  Model size: {} transitions", model.transitions.len());
    println!("  Average fitness: {:.2}", fitness);
    println!("  Total time: {:?}", start.elapsed());

    Ok(())
}
```

---

**Next:** [Advanced Topics](ADVANCED_TOPICS.md) for specialized optimization techniques.
