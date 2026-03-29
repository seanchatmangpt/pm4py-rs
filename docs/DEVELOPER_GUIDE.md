# PM4Py Developer Guide

> Complete guide for integrating pm4py-rust into your applications and understanding the library architecture.

---

## Table of Contents

1. [Quick Start (5 Minutes)](#quick-start-5-minutes)
2. [Installation](#installation)
3. [Core Concepts](#core-concepts)
4. [Common Workflows](#common-workflows)
5. [Library Architecture](#library-architecture)
6. [Performance Tips](#performance-tips)
7. [Troubleshooting](#troubleshooting)

---

## Quick Start (5 Minutes)

### Step 1: Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
pm4py = "0.3"
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1.0"
```

### Step 2: Create Event Log

```rust
use pm4py::log::{Event, EventLog, Trace};
use chrono::Utc;

fn main() {
    let mut log = EventLog::new();
    let now = Utc::now();

    // Create a trace (case)
    let mut trace = Trace::new("order_001");

    // Add events
    trace.add_event(Event::new("receive", now));
    trace.add_event(Event::new("process", now + chrono::Duration::minutes(30)));
    trace.add_event(Event::new("ship", now + chrono::Duration::hours(2)));

    log.add_trace(trace);

    println!("Created log with {} traces", log.traces().len());
}
```

### Step 3: Discover Process Model

```rust
use pm4py::discovery::InductiveMiner;

let miner = InductiveMiner::new();
let model = miner.mine(&log);

println!("Discovered model with {} places", model.places.len());
```

### Step 4: Check Conformance

```rust
use pm4py::conformance::TokenReplay;

let checker = TokenReplay::new();
let result = checker.replay(&log, &model);

println!("Fitness: {:.2}%", result.fitness * 100.0);
```

**That's it!** You now have:
- ✓ Imported pm4py
- ✓ Created an event log
- ✓ Discovered a process model
- ✓ Checked conformance

---

## Installation

### Option 1: Cargo (Recommended)

```bash
cargo add pm4py
```

### Option 2: Direct Dependency

```toml
[dependencies]
pm4py = { version = "0.3", features = ["visualization"] }
```

### Option 3: From Source

```bash
git clone https://github.com/seanchatmangpt/pm4py-rust.git
cd pm4py-rust
cargo build --release
```

### Features

| Feature | Description | Default |
|---------|-------------|---------|
| `std` | Standard library (required) | Yes |
| `visualization` | SVG/visualization output | Optional |
| `python` | Python bindings (PyO3) | Optional |

Enable features:

```toml
pm4py = { version = "0.3", features = ["visualization", "python"] }
```

---

## Core Concepts

### Events

Events represent activities in a business process.

```rust
use pm4py::log::Event;

// Basic event
let event = Event::new("activity_name", timestamp);

// With additional context
let event = event
    .with_resource("john")
    .with_attribute("amount", 1000)
    .with_attribute("priority", "high");

// Access event data
println!("Activity: {}", event.name());
println!("Time: {}", event.timestamp());
println!("Resource: {:?}", event.resource());
```

**Event Structure:**

| Field | Type | Required | Example |
|-------|------|----------|---------|
| name | String | Yes | "receive_order" |
| timestamp | DateTime | Yes | 2026-03-24T10:00:00Z |
| resource | String | No | "clerk" |
| attributes | Map | No | {"amount": 1000} |

### Traces

Traces are sequences of events for a specific case.

```rust
use pm4py::log::Trace;

let mut trace = Trace::new("order_001");

// Add events (must be chronological)
trace.add_event(event1);
trace.add_event(event2);
trace.add_event(event3);

// Access trace data
println!("Case ID: {}", trace.trace_id());
println!("Length: {}", trace.events().len());
println!("Duration: {:?}", trace.case_duration());
```

**Properties:**
- Events must be in chronological order
- Automatically calculates case duration
- Can have attributes at trace level

### Event Logs

Event logs contain all traces for analysis.

```rust
use pm4py::log::EventLog;

let mut log = EventLog::new();

// Add traces
log.add_trace(trace1);
log.add_trace(trace2);

// Access log data
println!("Traces: {}", log.traces().len());
println!("Events: {}", log.num_events());

// Filter logs
let filtered = log.filter_by_activity("receive_order");
```

**Log Statistics:**
- Total traces and events
- Unique activities
- Trace length distribution
- Case duration range

### Process Models

Different representations of discovered processes:

#### Petri Nets

```rust
use pm4py::models::PetriNet;

// PetriNet has:
// - Places: passive elements (state)
// - Transitions: active elements (activities)
// - Arcs: connections
// - Markings: token distribution

let petri_net = miner.mine(&log);
println!("Places: {}", petri_net.places.len());
println!("Transitions: {}", petri_net.transitions.len());
```

#### Process Trees

```rust
use pm4py::models::ProcessTree;

// Process trees show:
// - Sequence (→)
// - Choice (×)
// - Parallel (⋈)
// - Loop (⟳)

let tree = TreeMiner::new().mine(&log);
println!("Tree: {:?}", tree);
```

#### DFG (Directly-Follows Graph)

```rust
use pm4py::discovery::DFGMiner;

let miner = DFGMiner::new();
let dfg = miner.mine(&log);

// DFG shows direct activity succession
for (from, to, freq) in dfg.edges {
    println!("{} -> {} ({})", from, to, freq);
}
```

---

## Common Workflows

### Workflow 1: Process Discovery

**Goal:** Understand what processes actually happen.

```rust
use pm4py::discovery::{AlphaMiner, InductiveMiner, HeuristicMiner};
use pm4py::log::EventLog;

fn discover_process(log: &EventLog) {
    // Choose algorithm based on process characteristics
    let model = if log.traces().len() < 100 {
        // Small logs: use precise alpha
        AlphaMiner::new().mine(log)
    } else if has_many_loops(log) {
        // Loops: use inductive
        InductiveMiner::new().mine(log)
    } else {
        // Noisy: use heuristic
        HeuristicMiner::new()
            .with_frequency_threshold(0.2)
            .mine(log)
    };

    // Export model
    model.to_pnml("output.pnml");
}

fn has_many_loops(log: &EventLog) -> bool {
    // Check if log has repeated activities per trace
    log.traces()
        .iter()
        .filter(|t| {
            let activities: Vec<_> = t.events().iter().map(|e| e.name()).collect();
            activities.len() != activities.windows(2).count() + 1
        })
        .count()
        > log.traces().len() / 2
}
```

**Algorithm Selection:**

- **Alpha Miner:** Well-structured processes, few deviations
- **Inductive Miner:** Complex structures, loops, recursion
- **Heuristic Miner:** Noisy logs, frequency-based filtering
- **Split Miner:** Advanced splitting point detection
- **DFG:** Quick overview without formal semantics

### Workflow 2: Conformance Checking

**Goal:** Verify if reality matches the process definition.

```rust
use pm4py::conformance::{TokenReplay, ConformanceChecker};

fn check_conformance(log: &EventLog, model: &PetriNet) {
    let checker = TokenReplay::new();
    let result = checker.replay(log, model);

    // Overall metrics
    println!("Fitness: {:.2}%", result.fitness * 100.0);
    println!("Precision: {:.2}%", result.precision * 100.0);
    println!("Generalization: {:.2}%", result.generalization * 100.0);

    // Find deviant cases
    for trace_id in &result.deviant_traces {
        println!("Deviant case: {}", trace_id);

        // Get details
        if let Some(details) = result.conformance_details.get(trace_id) {
            println!("  Remaining tokens: {}", details.remaining_tokens);
            println!("  Enabled transitions: {:?}", details.enabled_transitions);
        }
    }

    // Root cause analysis
    if result.fitness < 0.95 {
        println!("\nFitness low - possible causes:");
        println!("  1. Process definition incomplete");
        println!("  2. Exceptional variants in data");
        println!("  3. Data quality issues");
    }
}
```

**Interpreting Metrics:**

| Metric | Good Range | Issue |
|--------|------------|-------|
| Fitness | > 0.95 | Low = process definition incomplete |
| Precision | > 0.90 | Low = model too permissive |
| Generalization | > 0.80 | Low = model too restrictive |

### Workflow 3: Performance Analysis

**Goal:** Find bottlenecks and inefficiencies.

```rust
use pm4py::statistics::analyze_tree;
use pm4py::performance::PerformanceMetrics;

fn analyze_performance(log: &EventLog) {
    // Calculate activity metrics
    let stats = log.statistics();

    for activity in stats.activities {
        println!("Activity: {}", activity.name);
        println!("  Frequency: {}", activity.frequency);
        println!("  Mean duration: {:?}", activity.mean_duration);
        println!("  Std dev: {:?}", activity.std_duration);

        // Find bottlenecks (high mean duration)
        if activity.mean_duration > chrono::Duration::hours(2) {
            println!("  ⚠️  BOTTLENECK: {}h average",
                     activity.mean_duration.num_hours());
        }
    }

    // Calculate case-level metrics
    let case_stats = log.traces()
        .iter()
        .map(|t| (t.trace_id(), t.case_duration()))
        .collect::<Vec<_>>();

    println!("\nCase Duration Analysis:");
    println!("  Mean: {}", mean_duration(&case_stats));
    println!("  Median: {}", median_duration(&case_stats));
    println!("  P95: {}", percentile_duration(&case_stats, 0.95));
}

fn mean_duration(durations: &[(String, chrono::Duration)]) -> chrono::Duration {
    let total: chrono::Duration = durations.iter().map(|(_, d)| d).sum();
    total / durations.len() as i32
}
```

### Workflow 4: Variant Analysis

**Goal:** Understand different execution paths.

```rust
use std::collections::HashMap;

fn analyze_variants(log: &EventLog) {
    let mut variants: HashMap<Vec<&str>, usize> = HashMap::new();

    // Group traces by activity sequence
    for trace in log.traces() {
        let pattern: Vec<&str> = trace.events()
            .iter()
            .map(|e| e.name().as_str())
            .collect();

        *variants.entry(pattern).or_insert(0) += 1;
    }

    // Sort by frequency
    let mut variants_vec: Vec<_> = variants.into_iter().collect();
    variants_vec.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

    println!("Top 10 Variants:");
    for (i, (pattern, count)) in variants_vec.iter().take(10).enumerate() {
        let pct = (count as f64 / log.traces().len() as f64) * 100.0;
        println!("{:2}. {:?} - {} cases ({:.1}%)", i+1, pattern, count, pct);
    }

    // Identify rare variants
    let rare_threshold = log.traces().len() / 100; // < 1%
    let rare_variants: Vec<_> = variants_vec
        .iter()
        .filter(|(_, count)| count < &rare_threshold)
        .collect();

    if rare_variants.len() > 10 {
        println!("\n⚠️  {} rare variants found (< 1%)", rare_variants.len());
        println!("  Consider data quality review or detailed case analysis");
    }
}
```

### Workflow 5: Log Filtering

**Goal:** Focus analysis on specific subsets.

```rust
use pm4py::log::AdvancedFilter;

fn filter_logs(log: &EventLog) {
    // By activity
    let log_with_orders = log.filter_by_activity("receive_order");

    // By trace length
    let log_short = log.filter_by_trace_length_range(2, 5);

    // By time range
    let log_recent = log.filter_by_date_range(
        "2026-03-01T00:00:00Z",
        "2026-03-31T23:59:59Z"
    );

    // By attribute value
    let log_vip = log.filter_by_attribute("customer_type", "VIP");

    // Chain filters
    let filtered = log
        .filter_by_attribute("status", "completed")
        .filter_by_trace_length_range(3, 10)
        .filter_by_activity("process");

    println!("Original: {} traces", log.traces().len());
    println!("Filtered: {} traces", filtered.traces().len());
}
```

---

## Library Architecture

### Module Structure

```
pm4py
├── log/           # Event log structures
│   ├── event.rs    # Individual events
│   ├── trace.rs    # Trace (case) sequences
│   └── operations/ # Log operations & filters
├── discovery/     # Process mining algorithms
│   ├── alpha_miner.rs
│   ├── inductive_miner.rs
│   ├── heuristic_miner.rs
│   └── ...
├── conformance/   # Conformance checking
│   ├── token_replay.rs
│   ├── alignment.rs
│   └── ...
├── models/        # Process representations
│   ├── petri_net.rs
│   ├── process_tree.rs
│   └── ...
├── statistics/    # Log analysis
└── utils/         # Utility functions
```

### Key Types

```rust
// Log structures
pub struct EventLog { traces: Vec<Trace> }
pub struct Trace { trace_id: String, events: Vec<Event> }
pub struct Event { name: String, timestamp: DateTime<Utc>, ... }

// Models
pub struct PetriNet {
    places: Vec<Place>,
    transitions: Vec<Transition>,
    arcs: Vec<Arc>,
}

// Results
pub struct ConformanceResult {
    fitness: f64,
    precision: f64,
    generalization: f64,
    deviant_traces: Vec<String>,
}
```

### Data Flow

```
Event Log (CSV/XES/JSON)
    ↓
EventLog struct
    ├→ Discovery Algorithm → Process Model
    │       ↓
    │   PetriNet/ProcessTree
    │       ↓
    │   Conformance Checker → ConformanceResult
    │
    └→ Statistics → LogStatistics
```

---

## Performance Tips

### 1. Use Appropriate Data Structures

```rust
// ✓ Good: EventLog with indices
let log = EventLog::new();
for trace in data {
    log.add_trace(trace);
}

// ✗ Avoid: Recreating EventLog multiple times
for i in 0..100 {
    let mut log = EventLog::new();  // ← Inefficient
    log.add_trace(trace);
}
```

### 2. Batch Operations

```rust
// ✓ Good: Single discovery pass
let model = InductiveMiner::new().mine(&log);

// ✗ Avoid: Multiple passes
for trace in log.traces() {
    let m = InductiveMiner::new().mine(&log);  // ← Redundant
}
```

### 3. Use Frequency Filtering

```rust
// For large, noisy logs, use frequency threshold
let miner = HeuristicMiner::new()
    .with_frequency_threshold(0.15)  // ← Filters noise
    .mine(&log);

// Without filtering, discovery is slower and results are noisier
```

### 4. Parallel Processing

```rust
use rayon::prelude::*;

// Parallel log processing
let results: Vec<_> = logs
    .par_iter()
    .map(|log| {
        let model = InductiveMiner::new().mine(log);
        TokenReplay::new().replay(log, &model)
    })
    .collect();
```

### 5. Cache Models

```rust
use std::collections::HashMap;

let mut model_cache = HashMap::new();

fn get_model(log_id: &str, log: &EventLog) -> PetriNet {
    if let Some(model) = model_cache.get(log_id) {
        return model.clone();
    }

    let model = InductiveMiner::new().mine(log);
    model_cache.insert(log_id.to_string(), model.clone());
    model
}
```

### 6. Memory Management

```rust
// ✓ Good: Stream large files
let log = EventLog::from_csv_stream("large_file.csv")?;

// ✗ Avoid: Loading all into memory
let content = std::fs::read_to_string("large_file.csv")?;
let log = EventLog::from_json(&content)?;  // ← OOM risk
```

### 7. Algorithm Selection by Log Size

| Log Size | Traces | Algorithm |
|----------|--------|-----------|
| Tiny | < 50 | Alpha (most precise) |
| Small | 50-1K | Inductive (balanced) |
| Medium | 1K-100K | Heuristic (scalable) |
| Large | 100K+ | DFG (minimal processing) |

```rust
fn choose_algorithm(log: &EventLog) -> Box<dyn DiscoveryAlgorithm> {
    match log.traces().len() {
        0..=50 => Box::new(AlphaMiner::new()),
        51..=1000 => Box::new(InductiveMiner::new()),
        1001..=100000 => Box::new(HeuristicMiner::new()),
        _ => Box::new(DFGMiner::new()),
    }
}
```

---

## Troubleshooting

### Issue: "No viable path for event replay"

**Cause:** Log contains activity not in model.

```rust
// Check model coverage
let model_activities: HashSet<_> = model.transitions.iter().collect();
let log_activities: HashSet<_> = log.traces()
    .iter()
    .flat_map(|t| t.events().iter().map(|e| e.name()))
    .collect();

let missing = log_activities.difference(&model_activities);
println!("Activities in log but not model: {:?}", missing);
```

**Solution:** Either:
1. Rediscover model from complete log
2. Filter log to only activities in model
3. Manually add missing activities to model

### Issue: "Fitness is very low (< 0.5)"

**Cause:** Model doesn't match log behavior.

```rust
fn diagnose_low_fitness(log: &EventLog, model: &PetriNet) {
    let result = TokenReplay::new().replay(log, model);

    // Analyze deviant traces
    println!("Deviant cases: {}", result.deviant_traces.len());
    for case_id in result.deviant_traces.iter().take(5) {
        if let Some(trace) = log.traces().iter().find(|t| t.trace_id() == case_id) {
            println!("Case: {:?}", trace.events().iter().map(|e| e.name()).collect::<Vec<_>>());
        }
    }

    // Try different algorithm
    let alt_model = HeuristicMiner::new()
        .with_frequency_threshold(0.05)
        .mine(log);
    let alt_result = TokenReplay::new().replay(log, &alt_model);
    println!("With heuristic: fitness = {}", alt_result.fitness);
}
```

### Issue: "Out of Memory"

**Cause:** Log too large for available memory.

```rust
// Solution 1: Stream processing
fn process_large_log(filename: &str) -> Result<()> {
    let stream = EventLog::from_csv_stream(filename)?;
    // Process batch by batch
    Ok(())
}

// Solution 2: Filter first
let log = EventLog::from_csv("data.csv")?
    .filter_by_date_range("2026-01-01", "2026-01-31");

// Solution 3: Sample
let sample_log = sample_traces(&log, 1000);
```

### Issue: "Model has no valid initial marking"

**Cause:** Petri net structure malformed.

```rust
// Validate model structure
fn validate_model(model: &PetriNet) -> Result<(), String> {
    if model.initial_marking.is_empty() {
        return Err("No initial marking".to_string());
    }
    if model.final_marking.is_empty() {
        return Err("No final marking".to_string());
    }
    if model.transitions.is_empty() {
        return Err("No transitions".to_string());
    }
    Ok(())
}
```

### Issue: "Discovery takes too long"

**Cause:** Large log with expensive algorithm.

```rust
// Solutions in order of preference:
// 1. Use frequency threshold
let model1 = HeuristicMiner::new()
    .with_frequency_threshold(0.2)  // Filters 80% of infrequent edges
    .mine(&log);

// 2. Use faster algorithm
let model2 = DFGMiner::new().mine(&log);  // ~1s even for large logs

// 3. Filter log first
let filtered = log.filter_by_date_range("2026-03-01", "2026-03-31");
let model3 = InductiveMiner::new().mine(&filtered);

// 4. Sample traces
let sample = sample_traces(&log, 5000);
let model4 = InductiveMiner::new().mine(&sample);
```

---

## Additional Resources

- **Examples:** `examples/` directory
- **API Reference:** `docs/API_REFERENCE.md`
- **OpenAPI Spec:** `docs/OPENAPI_SPEC.yaml`
- **GitHub:** https://github.com/seanchatmangpt/pm4py-rust
- **Original pm4py:** https://pm4py.fit.fraunhofer.de/

---

## Getting Help

1. **Check examples:** Most use cases covered in `examples/`
2. **Read API docs:** `docs/API_REFERENCE.md`
3. **Search issues:** https://github.com/seanchatmangpt/pm4py-rust/issues
4. **Open new issue:** Include minimal reproduction case
5. **Email:** info@chatmangpt.com

---

**Happy process mining!** 🚀
