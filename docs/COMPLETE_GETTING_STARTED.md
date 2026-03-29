# Complete Getting Started Guide for PM4Py Rust

**Last Updated:** 2026-03-24
**Target Audience:** Developers new to process mining
**Time to complete:** 5-15 minutes

---

## Table of Contents

1. [Installation (5 min)](#installation)
2. [Your First Discovery (5 min)](#first-discovery)
3. [Your First Conformance Check (5 min)](#first-conformance)
4. [Common Patterns](#common-patterns)
5. [FAQ](#faq)
6. [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

- **Rust 1.70+** — Required minimum version ([install here](https://rustup.rs/))
- **Cargo** — Comes with Rust
- **2 GB RAM** — For small logs; 8+ GB for benchmarking

### Verify Installation

```bash
rustc --version   # Should be 1.70.0 or newer
cargo --version
```

### Create Your First Project

```bash
cargo new pm4py_demo --bin
cd pm4py_demo
```

### Add PM4Py Dependency

Edit `Cargo.toml`:

```toml
[package]
name = "pm4py_demo"
version = "0.1.0"
edition = "2021"

[dependencies]
pm4py = "0.3"
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1.0"
```

Then run:

```bash
cargo build
```

This downloads and compiles pm4py (takes 1-3 minutes on first run).

---

## Your First Discovery

### Step 1: Create a Simple Event Log

Replace `src/main.rs` with:

```rust
use pm4py::log::{Event, EventLog, Trace};
use pm4py::discovery::AlphaMiner;
use chrono::Utc;

fn main() {
    // Create an event log with 3 simple cases
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // Case 1: Order → Payment → Ship
    let mut trace1 = Trace::new("case_1");
    trace1.add_event(Event::new("Order", base_time));
    trace1.add_event(Event::new("Payment", base_time + chrono::Duration::hours(1)));
    trace1.add_event(Event::new("Ship", base_time + chrono::Duration::hours(2)));
    log.add_trace(trace1);

    // Case 2: Same sequence
    let mut trace2 = Trace::new("case_2");
    trace2.add_event(Event::new("Order", base_time + chrono::Duration::days(1)));
    trace2.add_event(Event::new("Payment", base_time + chrono::Duration::days(1) + chrono::Duration::hours(1)));
    trace2.add_event(Event::new("Ship", base_time + chrono::Duration::days(1) + chrono::Duration::hours(2)));
    log.add_trace(trace2);

    // Case 3: Same sequence
    let mut trace3 = Trace::new("case_3");
    trace3.add_event(Event::new("Order", base_time + chrono::Duration::days(2)));
    trace3.add_event(Event::new("Payment", base_time + chrono::Duration::days(2) + chrono::Duration::hours(1)));
    trace3.add_event(Event::new("Ship", base_time + chrono::Duration::days(2) + chrono::Duration::hours(2)));
    log.add_trace(trace3);

    println!("✓ Created event log with {} cases", log.traces().len());

    // Step 2: Discover a process model
    let miner = AlphaMiner::new();
    let petri_net = miner.discover(&log);

    println!("✓ Discovered Petri Net:");
    println!("  - Places: {}", petri_net.places.len());
    println!("  - Transitions: {}", petri_net.transitions.len());
    println!("  - Arcs: {}", petri_net.arcs.len());
}
```

### Step 2: Run It

```bash
cargo run
```

**Expected output:**
```
✓ Created event log with 3 cases
✓ Discovered Petri Net:
  - Places: 4
  - Transitions: 3
  - Arcs: 8
```

### What Just Happened?

1. You created an **event log** with 3 traces (cases)
2. Each trace has 3 events (Order → Payment → Ship)
3. The **Alpha Miner** discovered a **Petri Net** model
4. The Petri Net captures the process flow

---

## Your First Conformance Check

Conformance checking validates that actual event logs conform to a process model.

### Complete Example

```rust
use pm4py::log::{Event, EventLog, Trace};
use pm4py::discovery::AlphaMiner;
use pm4py::conformance::TokenReplay;
use chrono::Utc;

fn main() {
    let base_time = Utc::now();

    // Create a training log (normal behavior)
    let mut training_log = EventLog::new();
    for i in 0..3 {
        let mut trace = Trace::new(format!("train_{}", i));
        let offset = base_time + chrono::Duration::days(i as i64);

        trace.add_event(Event::new("A", offset));
        trace.add_event(Event::new("B", offset + chrono::Duration::hours(1)));
        trace.add_event(Event::new("C", offset + chrono::Duration::hours(2)));
        training_log.add_trace(trace);
    }

    // Discover model from training data
    let miner = AlphaMiner::new();
    let model = miner.discover(&training_log);
    println!("✓ Discovered model from {} training cases", training_log.traces().len());

    // Create a test log (some normal, one deviant)
    let mut test_log = EventLog::new();

    // Normal case
    let mut trace1 = Trace::new("test_1");
    trace1.add_event(Event::new("A", base_time));
    trace1.add_event(Event::new("B", base_time + chrono::Duration::hours(1)));
    trace1.add_event(Event::new("C", base_time + chrono::Duration::hours(2)));
    test_log.add_trace(trace1);

    // Deviant case: B happens before A
    let mut trace2 = Trace::new("test_2");
    trace2.add_event(Event::new("B", base_time + chrono::Duration::days(1)));
    trace2.add_event(Event::new("A", base_time + chrono::Duration::days(1) + chrono::Duration::hours(1)));
    trace2.add_event(Event::new("C", base_time + chrono::Duration::days(1) + chrono::Duration::hours(2)));
    test_log.add_trace(trace2);

    // Check conformance
    let checker = TokenReplay::new();
    let results = checker.replay(&test_log, &model);

    for result in results {
        println!("Case {}: fitness = {:.2}", result.trace_id, result.fitness);
    }
}
```

**Expected output:**
```
✓ Discovered model from 3 training cases
Case test_1: fitness = 1.00
Case test_2: fitness = 0.67
```

The first case conforms perfectly (fitness = 1.0). The second case deviates (fitness < 1.0).

---

## Common Patterns

### Pattern 1: Load Log from File

```rust
use pm4py::io::xes::read_xes;

fn load_and_discover() {
    // Load from XES file
    match read_xes("path/to/log.xes") {
        Ok(log) => {
            println!("Loaded {} cases", log.traces().len());
            // Now discover...
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Pattern 2: Filter Logs

```rust
use pm4py::log::AdvancedFilter;

fn filter_example(log: &EventLog) {
    // Keep only activities that appear in 90% of traces
    let filtered = log.filter_activities_by_threshold(0.9);
    println!("Filtered from {} to {} activities",
             log.statistics().num_activities,
             filtered.statistics().num_activities);
}
```

### Pattern 3: Analyze Statistics

```rust
fn statistics_example(log: &EventLog) {
    let stats = log.statistics();

    println!("Log Statistics:");
    println!("  Traces: {}", stats.num_traces);
    println!("  Events: {}", stats.num_events);
    println!("  Activities: {}", stats.num_activities);
    println!("  Variants: {}", stats.num_variants);
    println!("  Avg trace length: {:.1}", stats.avg_trace_length);
}
```

### Pattern 4: Try Multiple Algorithms

```rust
use pm4py::discovery::{AlphaMiner, HeuristicMiner, TreeMiner};

fn compare_algorithms(log: &EventLog) {
    let alpha = AlphaMiner::new().discover(log);
    let heuristic = HeuristicMiner::new().discover(log);
    let tree = TreeMiner::new().discover(log);

    println!("Alpha: {} transitions", alpha.transitions.len());
    println!("Heuristic: {} transitions", heuristic.transitions.len());
    println!("Tree model size: {}", tree.size);
}
```

---

## FAQ

### Q1: What's the difference between discovery algorithms?

**Alpha Miner**
- Best for: Clean, well-structured logs
- Speed: ⚡ Very fast
- Limitation: Struggles with loops

**Heuristic Miner**
- Best for: Noisy, real-world logs
- Speed: ⚡ Fast
- Feature: Filters by frequency threshold

**Inductive Miner**
- Best for: Complex processes with many loops
- Speed: ⚡⚡ Medium
- Feature: Recursive decomposition

**Tree Miner**
- Best for: Simple processes, easy to understand
- Speed: ⚡⚡⚡ Slowest
- Feature: Process trees are human-readable

### Q2: How do I handle large logs?

For logs with >100K events:

```rust
use pm4py::log::AdvancedFilter;

fn handle_large_log(log: &EventLog) {
    // Filter by activity frequency
    let filtered = log.filter_activities_by_threshold(0.95);

    // Sample traces
    let sample_size = (filtered.traces().len() / 10).max(1000);
    let sampled = filtered.sample(sample_size);

    // Now discover
    let model = AlphaMiner::new().discover(&sampled);
    println!("Discovered model from {} sampled traces", sampled.traces().len());
}
```

### Q3: What format should my event log be?

PM4Py supports:
- **XES** (standard XML format) — Recommended
- **CSV** (comma-separated values) — Simple
- **JSON** (JavaScript Object Notation) — Flexible
- **In-memory** (Rust EventLog struct) — Direct API

### Q4: Why is conformance checking slow?

Token replay is O(n × m) where:
- n = number of events
- m = number of possible paths in model

For large logs:
1. Use sampling (test 10% of cases)
2. Use simpler conformance methods (DFG)
3. Parallelize across multiple cores

### Q5: How do I optimize discovery?

```rust
// Start simple, add complexity as needed
let mut log = load_log("events.xes");

// 1. Filter infrequent activities
log = log.filter_activities_by_threshold(0.95);

// 2. For large logs, sample first
if log.num_events() > 100_000 {
    log = log.sample(10_000);
}

// 3. Choose algorithm: fast for exploration, slower for accuracy
let model = if need_speed {
    DFGMiner::new().discover(&log)
} else {
    InductiveMiner::new().discover(&log)
};
```

---

## Troubleshooting

### Problem: "Cannot find pm4py crate"

**Solution:** Check Cargo.toml has the dependency:
```toml
[dependencies]
pm4py = "0.3"
```

Then run: `cargo update`

### Problem: Discovery produces empty model

**Cause:** Log may have no common patterns.

**Solution:**
```rust
let stats = log.statistics();
println!("Traces: {}", stats.num_traces);
println!("Unique activities: {}", stats.num_activities);

if stats.num_traces < 3 {
    eprintln!("Need at least 3 traces for discovery");
}
```

### Problem: Conformance results all show fitness 0.0

**Cause:** Model and log use different activity names.

**Solution:**
```rust
// Print activities in both
println!("Model activities: {:?}", model.get_activities());
println!("Log activities: {:?}", log.get_activities());

// They should match exactly
```

### Problem: Out of memory on large logs

**Solution:**
```rust
// Process in chunks
let chunk_size = 10_000;
for chunk in log.traces().chunks(chunk_size) {
    let chunk_log = EventLog::from_traces(chunk.to_vec());
    process_chunk(&chunk_log);
}
```

---

## Next Steps

- **[Algorithm Deep-Dive](ALGORITHM_DEEPDIVE.md)** — Understand how each algorithm works
- **[API Reference](API_REFERENCE.md)** — Complete API documentation
- **[Examples](../examples/)** — 20+ working examples
- **[Performance Guide](PERFORMANCE_TUNING.md)** — Optimize for your use case

---

**Happy process mining!** 🚀
