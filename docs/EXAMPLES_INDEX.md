# PM4Py Rust Examples Index

Complete guide to all working code examples. Each example is standalone and can be run independently.

---

## Quick Reference

All examples can be run with:
```bash
cargo run --example <name>
```

For example:
```bash
cargo run --example 01_simple_alpha_miner
cargo run --example 02_heuristic_with_filtering
```

---

## Discovery Examples

### Example 01: Simple Alpha Miner
**File:** `examples/01_simple_alpha_miner.rs`
**Time:** 5 minutes
**Complexity:** Beginner

The simplest discovery example. Demonstrates:
- Creating a basic event log with 5 traces
- Using Alpha Miner for discovery
- Examining discovered model structure

**Key code:**
```rust
let miner = AlphaMiner::new();
let petri_net = miner.discover(&log);
```

**When to use:** Learning process mining basics, quick proof-of-concept

---

### Example 02: Heuristic with Filtering
**File:** `examples/02_heuristic_with_filtering.rs`
**Time:** 10 minutes
**Complexity:** Beginner

Introduces noise handling. Demonstrates:
- Creating logs with variants and noise
- Adjusting Heuristic Miner threshold (0.1 vs 0.5 vs 0.9)
- Comparing model complexity at different thresholds

**Key code:**
```rust
let miner = HeuristicMiner::with_threshold(0.5);
let model = miner.discover(&log);
```

**When to use:** Working with real-world messy data, tuning noise tolerance

---

### Example 03: Inductive Decomposition
**File:** `examples/03_inductive_decomposition.rs`
**Time:** 15 minutes
**Complexity:** Intermediate

Deep dive into hierarchical discovery. Demonstrates:
- Creating logs with choice and loops
- Discovering process trees (not Petri Nets)
- Visualizing tree structure with operators (→, ⊕, ∥, ↻)

**Key code:**
```rust
let miner = InductiveMiner::new();
let tree = miner.discover(&log);
println!("Depth: {}, Leaves: {}", tree.depth(), tree.leaf_count());
```

**When to use:** Complex processes with loops, understanding hierarchical structure

---

### Example 04: Algorithm Comparison
**File:** `examples/04_algorithm_comparison.rs`
**Time:** 10 minutes
**Complexity:** Beginner

Side-by-side comparison of all algorithms. Demonstrates:
- Running all 4 discovery algorithms on same log
- Comparing output sizes
- Measuring execution time
- When to use each algorithm

**Key code:**
```rust
let alpha_model = AlphaMiner::new().discover(&log);
let heuristic_model = HeuristicMiner::new().discover(&log);
let inductive_model = InductiveMiner::new().discover(&log);
```

**When to use:** Choosing which algorithm for your problem, benchmarking

---

## Conformance Examples

### Example 05: Token Replay Basic
**File:** `examples/05_token_replay_basic.rs`
**Time:** 10 minutes
**Complexity:** Beginner

Foundation of conformance checking. Demonstrates:
- Creating training and test logs
- Discovering model from training data
- Replaying test traces on model
- Interpreting fitness scores

**Key code:**
```rust
let checker = TokenReplay::new();
let results = checker.replay(&test_log, &model);

for result in results {
    println!("Case {}: fitness = {:.2}", result.trace_id, result.fitness);
}
```

**When to use:** First-time conformance checking, validating models

**Fitness interpretation:**
- 1.0 = perfect fit (no deviations)
- 0.8-1.0 = good (minor deviations)
- 0.6-0.8 = fair (some deviations)
- <0.6 = poor (many deviations)

---

## Statistics Examples

### Example 06: Statistical Analysis
**File:** `examples/06_statistics_analysis.rs`
**Time:** 10 minutes
**Complexity:** Beginner

Learn about log statistics. Demonstrates:
- Basic log metrics (traces, events, activities)
- Activity frequency analysis
- Trace length distribution
- Temporal analysis (case duration, span)
- Process variant extraction

**Key code:**
```rust
let stats = log.statistics();
println!("Traces: {}", stats.num_traces);
println!("Activities: {}", stats.num_activities);
println!("Avg length: {:.2}", stats.avg_trace_length);
```

**When to use:** Understanding log characteristics, sanity checking

---

## Filtering Examples

### Example 07: Filtering Techniques
**File:** `examples/07_filtering_techniques.rs`
**Time:** 15 minutes
**Complexity:** Intermediate

Master log preparation. Demonstrates:
- Activity frequency filtering (threshold-based)
- Removing specific activities
- Filtering by trace length
- Random sampling
- Combining multiple filters

**Key code:**
```rust
let filtered = log.filter_activities_by_threshold(0.90);
let without_inspect = filter_remove_activity(&log, "Inspect");
let sample = log.sample(100);
```

**When to use:** Data cleaning, noise reduction, pre-processing

---

## Performance Examples

### Example 08: Performance Benchmarking
**File:** `examples/08_performance_benchmarking.rs`
**Time:** 5 minutes
**Complexity:** Intermediate

Measure algorithm performance. Demonstrates:
- Creating logs of different sizes
- Timing algorithm execution
- Creating benchmark table
- Comparing performance characteristics

**Key code:**
```rust
let start = Instant::now();
let model = AlphaMiner::new().discover(&log);
let elapsed = start.elapsed().as_secs_f64() * 1000.0;  // milliseconds
```

**When to use:** Performance testing, choosing algorithms for large logs

---

## End-to-End Examples

### Example 09: Complete Pipeline
**File:** `examples/09_end_to_end_pipeline.rs`
**Time:** 10 minutes
**Complexity:** Intermediate

Full workflow example. Demonstrates:
- Loading an event log
- Filtering/cleaning
- Discovering model
- Checking conformance
- Generating summary report

**Key code:**
```rust
let log = load_event_log();
let log = log.filter_activities_by_threshold(0.90);
let model = AlphaMiner::new().discover(&log);
let results = TokenReplay::new().replay(&log, &model);
```

**When to use:** Understanding complete workflow, starting your own project

---

### Example 10: Variant Analysis
**File:** `examples/10_variant_analysis.rs`
**Time:** 10 minutes
**Complexity:** Intermediate

Understand process variants. Demonstrates:
- Extracting unique process variants
- Counting variant frequency
- Sorting by prevalence
- Characterizing variant types (happy path, rework, rejection)
- Calculating diversity metrics

**Key code:**
```rust
let mut variants = HashMap::new();
for trace in log.traces() {
    let sequence: Vec<String> = trace.events.iter()
        .map(|e| e.name.clone())
        .collect();
    *variants.entry(sequence).or_insert(0) += 1;
}
```

**When to use:** Understanding process variability, identifying patterns

---

## Learning Path

### Beginner Track (40 minutes)
1. **Example 01** - Simple Alpha Miner (5 min)
2. **Example 04** - Algorithm Comparison (10 min)
3. **Example 05** - Token Replay (10 min)
4. **Example 06** - Statistics Analysis (10 min)
5. **Example 09** - End-to-End Pipeline (5 min)

**Goal:** Understand core concepts of discovery and conformance

### Intermediate Track (60 minutes)
1. Complete Beginner Track (40 min)
2. **Example 02** - Heuristic Filtering (10 min)
3. **Example 03** - Inductive Decomposition (10 min)
4. **Example 07** - Filtering Techniques (10 min)
5. **Example 10** - Variant Analysis (10 min)

**Goal:** Master parameter tuning and data preparation

### Advanced Track (additional 30 minutes)
1. Complete Intermediate Track (60 min)
2. **Example 08** - Performance Benchmarking (5 min)
3. Modify examples to test your own scenarios (25 min)

**Goal:** Optimize for your specific use case

---

## Running All Examples

To compile all examples:
```bash
cargo build --examples
```

To run all examples:
```bash
for example in 01_simple_alpha_miner 02_heuristic_with_filtering \
               03_inductive_decomposition 04_algorithm_comparison \
               05_token_replay_basic 06_statistics_analysis \
               07_filtering_techniques 08_performance_benchmarking \
               09_end_to_end_pipeline 10_variant_analysis; do
    echo "=== Running $example ==="
    cargo run --example $example
    echo ""
done
```

---

## Example Features Matrix

| Example | Discovery | Conformance | Statistics | Filtering | Performance |
|---------|-----------|-------------|-----------|-----------|-------------|
| 01 | ✓ | | | | |
| 02 | ✓ | | | ✓ | |
| 03 | ✓ | | | | |
| 04 | ✓ | | | | ✓ |
| 05 | | ✓ | | | |
| 06 | | | ✓ | | |
| 07 | | | | ✓ | |
| 08 | ✓ | | | | ✓ |
| 09 | ✓ | ✓ | | ✓ | |
| 10 | | | ✓ | | |

---

## Common Tasks & Example Maps

### "I want to discover a process model"
→ Examples: 01, 02, 03, 04, 09

### "I want to check if data fits my model"
→ Examples: 05, 09

### "I want to understand my event log"
→ Examples: 06, 10

### "I want to optimize for speed"
→ Examples: 04, 08

### "I want to handle noisy data"
→ Examples: 02, 07

### "I want a complete example"
→ Example: 09 (then customize)

---

## Troubleshooting Examples

### Example doesn't compile
- Check Rust version: `rustc --version` (need 1.70+)
- Update dependencies: `cargo update`
- Clean rebuild: `cargo clean && cargo build --examples`

### Example panics/crashes
- Some examples create in-memory logs; ensure you have ~100MB free
- Check `println!` output for error messages
- Modify example to handle edge cases

### Example is too slow
- Try `cargo run --release --example <name>` for optimized build
- Reduce log size in example's `create_*_log()` function

---

## Extending Examples

### Modify Example 01 to handle 1000 traces:
```rust
// In create_simple_event_log()
for case_id in 0..1000 {  // Changed from 0..5
    // ... rest same ...
}
```

### Modify Example 05 to test different conformance methods:
```rust
use pm4py::conformance::DFGConformance;

let checker = DFGConformance::new();
let results = checker.check(&test_log, &model);
```

### Modify Example 08 to test more log sizes:
```rust
let sizes = vec![10, 50, 100, 250, 500, 1000];  // Add more
```

---

## Next Steps

1. Run all 10 examples
2. Read the [Complete Getting Started Guide](COMPLETE_GETTING_STARTED.md)
3. Dive into [Algorithm Deep-Dive](ALGORITHM_DEEPDIVE.md)
4. Check [Troubleshooting Guide](COMPREHENSIVE_TROUBLESHOOTING.md) if stuck

---

**Happy learning!** 🚀
