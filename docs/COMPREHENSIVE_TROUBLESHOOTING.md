# Comprehensive Troubleshooting Runbook

**Quick fixes for common problems in PM4Py Rust**

---

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Compilation Errors](#compilation-errors)
3. [Runtime Errors](#runtime-errors)
4. [Discovery Problems](#discovery-problems)
5. [Conformance Issues](#conformance-issues)
6. [Performance Issues](#performance-issues)
7. [Data Format Issues](#data-format-issues)
8. [Memory Issues](#memory-issues)

---

## Installation Issues

### Problem: "Cannot find pm4py in registry"

**Symptoms:**
```
error: failed to resolve: use of undeclared type `pm4py`
```

**Diagnosis:**
Cargo.toml missing or incorrect dependency declaration.

**Solution:**

1. Check `Cargo.toml`:
```toml
[dependencies]
pm4py = "0.3"
```

2. Update Cargo index:
```bash
cargo update
cargo build
```

3. If still failing, try explicit version:
```toml
pm4py = { version = "0.3.0", registry = "crates-io" }
```

---

### Problem: Wrong Rust version

**Symptoms:**
```
error: package requires rustc 1.70 or newer
```

**Diagnosis:**
Your Rust is too old.

**Solution:**
```bash
# Check current version
rustc --version

# Update if needed
rustup update stable
rustup set default stable
```

---

## Compilation Errors

### Problem: "cannot find function `discover` in module"

**Symptoms:**
```rust
let model = miner.discover(&log);  // ERROR!
```

**Diagnosis:**
The discovery function might have a different name or signature.

**Solution:**

Check the correct API:
```rust
// Correct for PM4Py 0.3
use pm4py::discovery::AlphaMiner;

let miner = AlphaMiner::new();
let petri_net = miner.discover(&log);  // ✓ Correct
```

If error persists, check what methods are available:
```bash
cargo doc --open
# Look for your miner in the docs
```

---

### Problem: "type mismatch: expected EventLog, found &EventLog"

**Symptoms:**
```rust
let log = EventLog::new();
let model = miner.discover(log);  // ERROR: expected &EventLog
```

**Diagnosis:**
Discovery methods take references, not owned values.

**Solution:**
```rust
// Wrong - move semantics
let log = EventLog::new();
let model = miner.discover(log);  // ❌ log moved

// Right - borrowing
let log = EventLog::new();
let model = miner.discover(&log);  // ✓ borrow log
```

---

### Problem: "cannot find function after update"

**Symptoms:**
```
error[E0425]: cannot find function `xy` in module
```

**Diagnosis:**
API changed between versions.

**Solution:**

Check CHANGELOG:
```bash
cat pm4py-rust/CHANGELOG.md
```

If upgrading, note breaking changes:
```toml
# Lock version while migrating
pm4py = "0.2"  # Old version

# Later:
pm4py = "0.3"  # New version - check docs
```

---

## Runtime Errors

### Problem: "panicked at index out of bounds"

**Symptoms:**
```
thread 'main' panicked at 'index out of bounds: ...'
```

**Diagnosis:**
Accessing array with invalid index.

**Solution:**

```rust
// Don't use direct indexing
let places = &petri_net.places;
let place = places[999];  // ❌ PANIC if only 10 places

// Use safe access
if let Some(place) = places.get(999) {
    println!("Found: {:?}", place);
} else {
    println!("Index out of bounds");
}

// Or check bounds first
if index < places.len() {
    let place = &places[index];
}
```

---

### Problem: "unwrap() on None"

**Symptoms:**
```
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value'
```

**Diagnosis:**
Calling `.unwrap()` on optional value that's None.

**Solution:**

Replace unwrap with safe alternatives:

```rust
// ❌ Risky
let activity = log.get_activity(0).unwrap();

// ✓ Safe - handle None explicitly
match log.get_activity(0) {
    Some(activity) => println!("Activity: {}", activity),
    None => eprintln!("No activity found"),
}

// ✓ Also safe - provide default
let activity = log.get_activity(0).unwrap_or("UNKNOWN");
```

---

### Problem: "cannot borrow as mutable and immutable"

**Symptoms:**
```
error: cannot borrow `log` as mutable because it is also borrowed as immutable
```

**Diagnosis:**
Violating Rust's borrow checker rules.

**Solution:**

```rust
// ❌ Wrong - holding immutable borrow
let activity = &log.activity(0);
log.add_event(...);  // ❌ Can't borrow mutably while holding ref

// ✓ Right - drop the borrow first
let activity = log.activity(0).clone();  // Get owned value
drop(activity);  // Explicitly release
log.add_event(...);  // ✓ Now can borrow mutably

// ✓ Or use a scope
{
    let activity = &log.activity(0);
    println!("{}", activity);
}  // Borrow dropped here
log.add_event(...);  // ✓ Works
```

---

## Discovery Problems

### Problem: Empty or trivial model discovered

**Symptoms:**
```
Discovered model:
  Places: 2
  Transitions: 1
```

**Diagnosis:**
Either the log is empty or contains insufficient patterns.

**Solution:**

```rust
// 1. Check if log has data
let stats = log.statistics();
println!("Traces: {}", stats.num_traces);
println!("Events: {}", stats.num_events);
println!("Activities: {}", stats.num_activities);

// 2. Need minimum data:
//    - At least 3 traces
//    - At least 2 activities
//    - At least 3 events total
if stats.num_traces < 3 {
    eprintln!("ERROR: Need at least 3 traces");
}

// 3. If log has data, try different algorithm
let model1 = AlphaMiner::new().discover(&log);
let model2 = HeuristicMiner::new().discover(&log);

println!("Alpha: {} transitions", model1.transitions.len());
println!("Heuristic: {} transitions", model2.transitions.len());
```

---

### Problem: Model too complex/hard to understand

**Symptoms:**
```
Discovered model:
  Places: 156
  Transitions: 89
```

**Diagnosis:**
Log is noisy or has many variants.

**Solution:**

```rust
// 1. Filter the log first
let filtered = log.filter_activities_by_threshold(0.95);
println!("Filtered {} to {} activities",
         log.statistics().num_activities,
         filtered.statistics().num_activities);

// 2. Sample if too large
let sampled = filtered.sample(1000);

// 3. Try Inductive Miner (hierarchical)
let tree = InductiveMiner::new().discover(&sampled);
println!("Tree depth: {}", tree.depth());  // More understandable

// 4. Or use strict Heuristic
let miner = HeuristicMiner::with_threshold(0.9);
let model = miner.discover(&sampled);
```

---

### Problem: Loops not detected

**Symptoms:**
```
Log has: A → B → A → B → A (repeating)
But discovered model has: A → B (no loop)
```

**Diagnosis:**
Alpha Miner cannot detect loops.

**Solution:**

```rust
// Alpha Miner doesn't handle loops
// Use Inductive or Heuristic instead:

// Option 1: Inductive Miner (best for loops)
let tree = InductiveMiner::new().discover(&log);

// Option 2: Heuristic Miner
let heuristic = HeuristicMiner::new().discover(&log);

// NOT Alpha Miner
let alpha = AlphaMiner::new().discover(&log);  // ❌ Won't find loop
```

---

## Conformance Issues

### Problem: All cases show fitness 0.0

**Symptoms:**
```
Case 1: fitness = 0.00
Case 2: fitness = 0.00
Case 3: fitness = 0.00
```

**Diagnosis:**
Usually: activity names don't match between log and model.

**Solution:**

```rust
// 1. Check activity names match exactly
let model_activities = model.get_activities();
let log_activities = log.get_activities();

println!("Model activities: {:?}", model_activities);
println!("Log activities: {:?}", log_activities);

// 2. Case sensitivity matters!
// "Order" ≠ "order" ≠ "ORDER"

// 3. Check for whitespace
// "Order " (with space) ≠ "Order"

// 4. Fix if needed
let log_fixed = log.map_activities(|a| {
    a.trim().to_lowercase()  // Normalize
});

// 5. Then check conformance
let checker = TokenReplay::new();
let results = checker.replay(&log_fixed, &model);
```

---

### Problem: Some traces show fitness 0.0, others 1.0

**Symptoms:**
```
Case 1: fitness = 1.00 ✓
Case 2: fitness = 0.00 ✗
Case 3: fitness = 0.67 ~
```

**Diagnosis:**
This is normal! Some variants don't fit the model.

**Solution:**

```rust
// 1. Analyze deviations
for result in results {
    if result.fitness < 1.0 {
        println!("Case {} has deviations:", result.trace_id);
        println!("  Missing transitions: {:?}", result.missing_transitions);
        println!("  Remaining tokens: {:?}", result.remaining_tokens);
    }
}

// 2. If too many failures, retrain model
let deviating_cases: Vec<_> = results.iter()
    .filter(|r| r.fitness < 0.8)
    .count();
println!("Deviations: {}/{}", deviating_cases, results.len());

if deviating_cases > results.len() / 2 {
    eprintln!("Model doesn't fit log well!");
    eprintln!("Consider retraining with different algorithm");
}
```

---

## Performance Issues

### Problem: Discovery is very slow

**Symptoms:**
```
Starting discovery...
(5 minutes later...)
Still waiting...
```

**Diagnosis:**
Large log or complex algorithm choice.

**Solution:**

```rust
// 1. Use faster algorithm
// Ranking: Alpha > Heuristic > Inductive
let start = std::time::Instant::now();

// Choose based on needs
let model = if speed_critical {
    AlphaMiner::new().discover(&log)  // ⚡ ~50ms
} else if need_loops {
    HeuristicMiner::new().discover(&log)  // ⚡⚡ ~200ms
} else if need_formal {
    InductiveMiner::new().discover(&log)  // ⚡⚡⚡ ~5000ms
};

println!("Discovery took: {:?}", start.elapsed());

// 2. Reduce log size first
let filtered = log.filter_activities_by_threshold(0.95);
let sampled = filtered.sample(10000);

let model = AlphaMiner::new().discover(&sampled);
```

---

### Problem: Conformance checking is slow

**Symptoms:**
```
Replaying 100K events... ETA: 10 minutes
```

**Diagnosis:**
Token replay is O(n × m) where n = events, m = paths in model.

**Solution:**

```rust
// 1. Sample first
let sample_size = log.num_events().min(10000);
let log_sample = log.sample_events(sample_size);

let results = checker.replay(&log_sample, &model);

// 2. Use simpler conformance method
// Instead of token replay...
use pm4py::conformance::DFGConformance;

let checker = DFGConformance::new();  // Faster
let results = checker.check(&log, &model);

// 3. Parallelize if possible
use rayon::prelude::*;

let results: Vec<_> = log.traces()
    .par_iter()
    .map(|trace| {
        let mut single_log = EventLog::new();
        single_log.add_trace(trace.clone());
        checker.replay(&single_log, &model)
    })
    .flatten()
    .collect();
```

---

### Problem: Out of memory

**Symptoms:**
```
error: process exited with code 137 (killed: out of memory)
```

**Diagnosis:**
Log is too large for available RAM.

**Solution:**

```rust
// 1. Process in chunks
let chunk_size = 5000;
for chunk in log.traces().chunks(chunk_size) {
    let chunk_log = EventLog::from_traces(chunk.to_vec());
    process_chunk(&chunk_log);
}

// 2. Or stream-process
for trace in log.traces() {
    let mut single = EventLog::new();
    single.add_trace(trace.clone());
    process_single(&single);
    // Memory freed after each iteration
}

// 3. Use sampling
let sample = log.sample(10000);  // Keep only 10K events
let model = discover(&sample);

// 4. Check memory usage
println!("Traces: {}", log.traces().len());
println!("Approx size: {} MB", log.traces().len() * 2000 / 1_000_000);
```

---

## Data Format Issues

### Problem: "Error reading XES file: invalid XML"

**Symptoms:**
```
Error: XML parsing failed
```

**Diagnosis:**
XES file is malformed.

**Solution:**

```rust
use pm4py::io::xes::read_xes;

// 1. Validate XML first
match read_xes("log.xes") {
    Ok(log) => println!("✓ Loaded {} traces", log.traces().len()),
    Err(e) => {
        eprintln!("❌ Parse error: {}", e);
        eprintln!("Check that:");
        eprintln!("  1. File is valid XML");
        eprintln!("  2. Uses UTF-8 encoding");
        eprintln!("  3. Has proper closing tags");
    }
}

// 2. Try converting from another format first
use pm4py::io::csv::read_csv;
let csv_log = read_csv("log.csv")?;
```

---

### Problem: CSV import missing attributes

**Symptoms:**
```
Loaded CSV, but no attributes found
```

**Diagnosis:**
CSV doesn't have required columns.

**Solution:**

```rust
// CSV requires these columns:
// - case_id (or caseID)
// - activity (or Activity)
// - timestamp (or Timestamp)

// Example correct CSV:
/*
case_id,activity,timestamp
case_1,Order,2024-01-01 10:00:00
case_1,Payment,2024-01-01 10:30:00
case_1,Ship,2024-01-01 11:00:00
*/

// Check your CSV:
use pm4py::io::csv::read_csv;

match read_csv("log.csv") {
    Ok(log) => {
        println!("✓ Loaded {} events", log.num_events());
    }
    Err(e) => {
        eprintln!("❌ Error: {}", e);
        eprintln!("Ensure CSV has columns:");
        eprintln!("  - case_id");
        eprintln!("  - activity");
        eprintln!("  - timestamp");
    }
}
```

---

### Problem: Different timestamps formats

**Symptoms:**
```
Error: unable to parse timestamp "01/15/2024"
```

**Diagnosis:**
Timestamp format not recognized.

**Solution:**

```rust
// Supported formats:
// - ISO 8601: 2024-01-15T10:30:00Z
// - RFC 3339: 2024-01-15 10:30:00
// - Unix timestamp: 1705315800

// If using different format:
use chrono::NaiveDateTime;

let custom_timestamp = "01/15/2024 10:30";
match NaiveDateTime::parse_from_str(custom_timestamp, "%m/%d/%Y %H:%M") {
    Ok(dt) => println!("Parsed: {}", dt),
    Err(e) => eprintln!("Parse error: {}", e),
}

// Or preprocess CSV to convert dates
fn convert_csv_dates(input: &str) -> String {
    input.lines().map(|line| {
        // Parse and reformat each timestamp
        line  // Your conversion logic
    }).collect()
}
```

---

## Memory Issues

### Problem: Excessive memory for large logs

**Symptoms:**
```
Process using 12GB RAM
```

**Diagnosis:**
Keeping entire log in memory.

**Solution:**

```rust
// 1. Use iterators instead of collecting
// ❌ Wrong - collects all into Vec
let all_traces: Vec<_> = log.traces().collect();

// ✓ Right - iterates without collecting
for trace in log.traces() {
    // Process one trace at a time
}

// 2. Clear data as you process
let mut log = load_log();
let model = AlphaMiner::new().discover(&log);
drop(log);  // Free memory explicitly

// Now conformance checking uses model only
let checker = TokenReplay::new();
// Can load fresh small samples

// 3. Use filtering
let filtered = log
    .filter_activities_by_threshold(0.99)
    .sample(1000);
// Now much smaller
```

---

## Summary Table

| Problem | Cause | Fix |
|---------|-------|-----|
| Empty model | No data | Check log statistics |
| Fitness 0.0 | Name mismatch | Normalize activity names |
| Too slow | Large log + slow algo | Use Alpha, filter, sample |
| Out of memory | Entire log in RAM | Process in chunks |
| Won't compile | Borrow error | Use references correctly |
| Loops not found | Using Alpha | Use Inductive/Heuristic |

---

**Still stuck?** Check [API Reference](API_REFERENCE.md) or open an issue on [GitHub](https://github.com/seanchatmangpt/pm4py-rust).
