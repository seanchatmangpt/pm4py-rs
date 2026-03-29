# Performance Optimization Analysis & Bottleneck Identification

## Overview

This guide helps identify and fix performance bottlenecks using benchmark results
and profiling tools. Includes analysis templates for different algorithm types.

## Phase 1: Analyze Benchmark Results

### Step 1: Review Raw Metrics

```bash
# View Criterion results
open target/criterion/report/index.html

# Export JSON for analysis
cat target/criterion/*/raw.json | jq '.
```

### Step 2: Calculate Scaling Efficiency

For each algorithm and scale:

```
Ideal speedup (linear) = 10x improvement for 10x events
Actual speedup = Time(1M) / Time(100K)

Efficiency = Actual / Ideal × 100%
- 100% = Perfect linear scaling
- 50% = O(n log n) behavior
- 10% = O(n²) or overhead-bound
```

### Step 3: Identify Regression Candidates

Algorithms showing:
- **Slower than expected:** Check for O(n²) behavior
- **Worse scaling:** Check for allocation overhead
- **High variance:** Check for GC/OS interference

## Phase 2: Profiling Techniques

### 1. Criterion-Based Profiling (Built-in)

**No setup required**, automatic with benchmarks:

```
Criterion measures:
✓ Wall-clock time
✓ Throughput (events/sec)
✓ Statistical variance
✓ Baseline regression
```

### 2. Flamegraph-Based Profiling

**Setup:**

```bash
# Install flamegraph tools
cargo install flamegraph
brew install graphviz (on macOS)
```

**Profile a benchmark:**

```bash
# Generate flamegraph for DFG discovery
cargo flamegraph --bench scale_benchmarks \
  -p pm4py --test dfg_miner_1m
```

**Interpret:**
- Wider bars = more CPU time
- Nested boxes = call stack
- Left-to-right = timeline

### 3. Memory Profiling

**Valgrind (Linux):**

```bash
# Profile memory allocation
valgrind --tool=massif \
  target/release/deps/scale_benchmarks-*

# View results
ms_print massif.out.* | head -100
```

**Heaptrack (Linux):**

```bash
heaptrack target/release/deps/scale_benchmarks-*
heaptrack_gui heaptrack.app.*.zst
```

**macOS/OSX:**

```bash
# Using Xcode instruments
cargo build --release --bench scale_benchmarks
instruments -t "System Trace" -o trace.trace \
  target/release/deps/scale_benchmarks-*

open trace.trace
```

### 4. CPU Profiling

**perf (Linux):**

```bash
# Record CPU samples
perf record -g cargo bench --bench scale_benchmarks

# Analyze
perf report

# Export flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > perf.svg
```

**Instruments (macOS):**

```bash
cargo build --release --bench scale_benchmarks
instruments -t "CPU Profiler" \
  target/release/deps/scale_benchmarks-*
```

## Phase 3: Algorithm-Specific Analysis

### DFG Miner Optimization

**Baseline Characteristics:**
- Time complexity: O(n) - linear
- Expected speedup: 3-5x
- Memory: O(unique_edges)

**Common Bottlenecks:**

1. **HashMap Operations** (50% of time)
   ```rust
   // Problem: Repeated rehashing
   let mut dfg = HashMap::new();
   for edge in &log {
       dfg.entry(edge).or_insert(0) += 1;  // Rehash on growth
   }

   // Solution: Pre-allocate
   let mut dfg = HashMap::with_capacity(estimated_edges);
   ```

2. **String Cloning** (20% of time)
   ```rust
   // Problem: Clone every activity name
   let activity = event.activity.clone();

   // Solution: Use references or Arc
   use std::sync::Arc;
   let activity = Arc::clone(&event.activity);
   ```

3. **Vector Growing** (15% of time)
   ```rust
   // Problem: Repeated allocations
   let mut edges = Vec::new();
   for trace in &log { edges.push(...); }  // May reallocate

   // Solution: Reserve capacity
   let mut edges = Vec::with_capacity(log.len() * 10);
   ```

**Optimization Priority:**
1. HashMap operations (biggest impact)
2. String handling (medium impact)
3. Memory allocation (long-tail)

### Alpha Miner Optimization

**Baseline Characteristics:**
- Time complexity: O(n log n) - sorting-bound
- Expected speedup: 2-4x
- Memory: O(n) - holds all pairs

**Common Bottlenecks:**

1. **Sorting Algorithm** (40% of time)
   ```rust
   // Problem: General-purpose sort
   pairs.sort_by_key(|p| p.frequency);

   // Solution: Radix sort for integers
   // Or unstable sort if ordering doesn't matter
   pairs.sort_unstable_by_key(|p| p.frequency);
   ```

2. **Directly-Follows Relation** (30% of time)
   ```rust
   // Problem: O(n²) pair generation
   for i in 0..trace.len() {
       for j in 0..trace.len() {
           if (i+1) == j { pairs.push((i,j)); }
       }
   }

   // Solution: Single pass
   for i in 0..trace.len()-1 {
       pairs.push((i, i+1));
   }
   ```

3. **Causality Checking** (20% of time)
   ```rust
   // Profile causality matrix operations
   // May benefit from bit-packed sets
   ```

**Optimization Priority:**
1. Causality checking (biggest unknown)
2. Sorting strategy (if O(n²) detected)
3. Memory layout (minor)

### Inductive Miner Optimization

**Baseline Characteristics:**
- Time complexity: O(n log n) - recursion bound
- Expected speedup: 2-3x (lowest)
- Memory: O(n) - recursion stack

**Common Bottlenecks:**

1. **Recursion Overhead** (50% of time)
   ```rust
   // Problem: Deep recursion with copies
   fn discover_recursive(log: EventLog, params: Params) -> Petri {
       let split_log = log.clone();  // Expensive copy
       discover_recursive(split_log, params)
   }

   // Solution: Use references + allocator
   fn discover_recursive(log: &EventLog, ...) { ... }
   ```

2. **Log Filtering** (30% of time)
   ```rust
   // Problem: Repeated log scanning
   let filtered = log.traces.iter()
       .filter(|t| condition(t))
       .collect::<Vec<_>>();

   // Solution: Index-based tracking
   let indices: Vec<usize> = (0..log.traces.len())
       .filter(|&i| condition(&log.traces[i]))
       .collect();
   ```

3. **Cut Detection** (15% of time)
   ```rust
   // Profile different cut detection strategies
   // May benefit from early termination
   ```

**Optimization Priority:**
1. Recursion overhead (biggest impact)
2. Log filtering (medium impact)
3. Cut detection (fine-tuning)

## Phase 4: Benchmarking Optimization Results

### Before/After Comparison

```bash
# Establish baseline
cargo bench --bench scale_benchmarks -- --baseline before

# Make optimization
# ... edit code ...

# Rebuild and benchmark
cargo bench --bench scale_benchmarks

# View comparison
cargo bench --bench scale_benchmarks -- --baseline before
```

**Expected Improvements:**

- HashMap: 5-20% improvement
- String interning: 10-30% improvement
- Memory pre-allocation: 5-15% improvement
- Algorithm changes: 20-50% improvement

### Statistical Validation

```bash
# Run 10+ iterations for confidence
for i in {1..10}; do
    cargo bench --bench scale_benchmarks >> results_$i.txt
done

# Analyze variance
cat results_*.txt | grep "time:" | \
  awk '{print $3}' | sort -n | \
  python3 -c "import sys; data=[float(x) for x in sys.stdin]; print(f'Mean: {sum(data)/len(data)}, StdDev: {(sum((x-sum(data)/len(data))**2 for x in data)/len(data))**0.5}')"
```

## Phase 5: Implementation Checklist

### Before Optimization

- [ ] Establish baseline benchmark (3+ runs)
- [ ] Review statistical variance
- [ ] Identify top 3 bottlenecks
- [ ] Profile with flamegraph
- [ ] Document current behavior

### During Optimization

- [ ] Make single change per iteration
- [ ] Compile in release mode only
- [ ] Run benchmarks after each change
- [ ] Document change rationale
- [ ] Keep old code as comments

### After Optimization

- [ ] Compare against baseline
- [ ] Verify accuracy unchanged
- [ ] Check memory usage
- [ ] Review code for maintainability
- [ ] Document optimization in comments

## Common Optimization Patterns

### 1. Pre-allocation

```rust
// Before
let mut events = Vec::new();
for trace in &log.traces {
    events.extend(trace.events.iter());
}

// After
let mut events = Vec::with_capacity(log.event_count());
for trace in &log.traces {
    events.extend(trace.events.iter());
}
```

**Expected gain:** 5-10% for large logs

### 2. Reference Handling

```rust
// Before
fn process(activity: String) { ... }
let activity = event.activity.clone();
process(activity);

// After
fn process(activity: &str) { ... }
process(&event.activity);
```

**Expected gain:** 10-15% for string-heavy code

### 3. Iterator Chaining

```rust
// Before
let mut results = Vec::new();
for item in collection {
    if condition(&item) {
        results.push(transform(&item));
    }
}

// After
let results: Vec<_> = collection.iter()
    .filter(|item| condition(item))
    .map(|item| transform(item))
    .collect();
```

**Expected gain:** 2-5% (compiler optimization)

### 4. Early Termination

```rust
// Before
let mut found = false;
for item in &collection {
    if matches(item) {
        found = true;
    }
}

// After
let found = collection.iter().any(|item| matches(item));
```

**Expected gain:** Variable (depends on probability)

### 5. Lazy Evaluation

```rust
// Before
let all_filtered = collection.iter()
    .filter(|x| expensive_check(x))
    .collect::<Vec<_>>();
// Then use subset
for item in &all_filtered { ... }

// After
collection.iter()
    .filter(|x| expensive_check(x))
    .for_each(|item| { ... })
```

**Expected gain:** 10-20% (less memory)

## Benchmarking Tools Comparison

| Tool | Setup | Cost | Output | When to Use |
|------|-------|------|--------|------------|
| Criterion (built-in) | None | 0 | HTML reports | Always - baseline |
| Flamegraph | 5 min | 0 | Call stacks | CPU hotspots |
| Valgrind | Linux only | High | Memory details | Memory leaks |
| perf | Linux only | Medium | CPU samples | Detailed analysis |
| Instruments | macOS only | None | UI tools | Production profiling |

## Performance Targets

### Discovery Algorithms

```
Algorithm | Scale | Current | Target | Gap
----------|-------|---------|--------|------
Alpha     | 100K  | 2-3x    | 3-4x   | 20-30%
Alpha     | 1M    | 2-3x    | 3-4x   | 20-30%
Inductive | 100K  | 2-2.5x  | 3-4x   | 40-50%
Inductive | 1M    | 2-2.5x  | 3-4x   | 40-50%
DFG       | 100K  | 4-5x    | 5-6x   | 10-20%
DFG       | 1M    | 4-5x    | 5-6x   | 10-20%
DFG       | 10M   | 4-5x    | 6-8x   | 30-50%
```

## Regression Prevention

### Continuous Monitoring

```bash
# Store baseline
cargo bench --bench scale_benchmarks -- --baseline v1.0

# Set up CI to compare
if cargo bench --bench scale_benchmarks | grep -i regression; then
    echo "Performance regression detected!"
    exit 1
fi
```

### Acceptable Variance

- **Within 5%:** Normal variance, ignore
- **5-10%:** Investigate, may be real
- **10%+:** Likely real regression, fix

## Documentation Template

For each optimization:

```markdown
### Optimization: [Name]

**Type:** [Bottleneck Type - see Phase 3]
**Status:** [In Progress / Completed / Abandoned]

**Problem:**
[Describe the bottleneck in detail]

**Solution:**
[Code change or algorithm change]

**Impact:**
- Time: [X ms → Y ms, Z% improvement]
- Memory: [A MB → B MB]
- Accuracy: [Impact on results]

**Trade-offs:**
[Any downsides or complexity costs]

**Verification:**
[How improvement was measured]
```

## Resources

### Tools
- [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph](https://www.brendangregg.com/flamegraphs.html)
- [perf examples](https://www.brendangregg.com/perf.html)

### Rust Performance
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Fastware](https://www.rust-lang.org/what/wg-wg-libs-performance/)

### Process Mining
- [PM4PY Performance Tips](https://pm4py.fit.fraunhofer.de/documentation)

## Support

When optimization isn't working:

1. Verify baseline was established correctly
2. Check that release mode is enabled
3. Review compiler flags (LTO, codegen units)
4. Consider if bottleneck is inherent to algorithm
5. Profile with multiple tools to verify finding
