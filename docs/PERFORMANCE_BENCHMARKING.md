# Performance Benchmarking: pm4py-rust vs Python pm4py

## Overview

This document describes the comprehensive performance benchmarking suite for pm4py-rust,
comparing Rust implementation against the Python reference implementation (pm4py).

## Benchmark Coverage

### 1. Process Discovery Algorithms

**Algorithms Tested:**
- Alpha Miner
- Inductive Miner
- DFG (Directly-Follows Graph) Miner

**Event Scale:**
- 100K events (baseline)
- 1M events (enterprise scale)
- 10M events (large organization)

**Pattern Types:**
- Linear traces (sequential activities)
- Parallel flows (concurrent activities)
- Complex loops (rework patterns)

**Metrics Collected:**
- Wall-clock time (milliseconds)
- Throughput (events/second)
- Scalability characteristics

### 2. Conformance Checking

**Algorithms Tested:**
- Token Replay (primary conformance metric)

**Scenarios:**
- Conforming logs (100% fitness)
- Deviating logs (with violations)

**Event Scale:**
- 100K events
- 1M events

**Metrics Collected:**
- Fitness scores
- Token replay efficiency
- Memory usage during checking

### 3. Statistical Analysis

**Operations Tested:**
- Frequency analysis (activity counts)
- Variant extraction (unique trace patterns)
- Rework pattern detection

**Event Scale:**
- 100K events
- 1M events

**Metrics Collected:**
- Execution time per operation
- Pattern discovery efficiency
- Throughput (operations/second)

## Running the Benchmarks

### Quick Start: All Benchmarks

```bash
./scripts/run_benchmarks.sh
```

This runs:
1. Rust benchmarks (all suites)
2. Python benchmarks (if pm4py installed)
3. Comparison report generation

### Rust Only

```bash
# Run all Rust benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench scale_benchmarks
cargo bench --bench discovery_bench
cargo bench --bench comprehensive_conformance_bench
cargo bench --bench comprehensive_statistics_bench
```

### Python Only

```bash
python3 scripts/python_benchmark.py \
    --output benchmark_results_python.json
```

### Generate Comparison Report

```bash
python3 scripts/compare_benchmarks.py \
    path/to/rust_results.json \
    path/to/python_results.json \
    --output-json comparison.json \
    --output-md PERFORMANCE_COMPARISON.md
```

## Benchmark Details

### Discovery Benchmarks

**Location:** `benches/discovery_bench.rs`

**Test Cases:**

| Algorithm | Pattern | Scale | Purpose |
|-----------|---------|-------|---------|
| Alpha | Linear | 100K, 1M, 10M | Sequential baseline |
| Alpha | Parallel | 100K, 1M | Control flow complexity |
| Inductive | Loop | 100K, 1M | Rework/redo patterns |
| Inductive | Parallel | 100K, 1M | Branching complexity |
| DFG | Linear | 100K, 1M, 10M | High throughput |
| DFG | Parallel | 100K, 1M, 10M | Complex flows |
| DFG | Loop | 100K, 1M | Rework detection |

**Expected Results:**
- DFG should be fastest (linear-time algorithm)
- Alpha Miner shows consistent performance
- Inductive Miner slower but handles complex patterns

### Conformance Benchmarks

**Location:** `benches/comprehensive_conformance_bench.rs`

**Scenarios:**

```
Conforming Log         → All traces follow discovered model
                         Expected fitness: 100%

Deviating Log         → ~10% of traces have violations
                         Expected fitness: 90%
```

**Expected Results:**
- Conforming logs: Faster execution (fewer token conflicts)
- Deviating logs: Slower (more complex token paths)
- Speedup: 2-3x over Python for large logs

### Statistics Benchmarks

**Location:** `benches/comprehensive_statistics_bench.rs`

**Operations:**

1. **Frequency Analysis**
   - Count activity occurrences
   - Time complexity: O(n)
   - Expected: Rust 3-5x faster

2. **Variant Extraction**
   - Identify unique trace patterns
   - Time complexity: O(n log n)
   - Expected: Rust 2-4x faster

3. **Rework Detection**
   - Find loop/rework patterns
   - Time complexity: O(n)
   - Expected: Rust 3-5x faster

## Statistical Significance

### Sampling Strategy

- **Sample Size:** 3-5 runs per benchmark
- **Measurement Time:** 10-30 seconds per run
- **Warmup Runs:** 1 run (cache/JIT warmup)
- **Variability:** Reported as min/max/mean ± stdev

### Variance Handling

Python results show higher variance due to:
- Garbage collection pauses
- JIT compilation overhead
- Dynamic typing costs

Rust results more stable:
- Predictable memory management
- Zero-cost abstractions
- Compile-time optimizations

## Expected Performance Characteristics

### Speedup Targets

Based on language characteristics:

| Category | Expected Speedup | Confidence |
|----------|------------------|-----------|
| Discovery (DFG) | 2-5x | High |
| Discovery (Alpha) | 2-4x | High |
| Discovery (Inductive) | 2-4x | High |
| Conformance (Token Replay) | 2-3x | Medium |
| Statistics (Frequency) | 3-5x | High |
| Statistics (Variants) | 2-3x | Medium |

### Factors Affecting Performance

**Advantages for Rust:**
- No GC pauses
- Zero-cost abstractions
- SIMD optimizations
- Memory efficiency
- Predictable performance

**Advantages for Python:**
- NumPy optimizations (C backend)
- Mature libraries
- Potentially better cache locality
- JIT compilation (PyPy)

## Accuracy Verification

All metrics must match Python pm4py within tolerance:

```
Fitness scores:    ±1e-10
Precision/Recall:  ±1e-10
Event counts:      Exact match (integer)
```

### Verification Steps

1. Generate identical event logs (seed-based)
2. Run both implementations
3. Compare output metrics
4. Report discrepancies (if any)

## Memory Profiling

Track memory usage during benchmarks:

```bash
# Rust memory profiling (using valgrind/heaptrack)
valgrind --tool=massif cargo bench

# Python memory profiling
python3 -m memory_profiler scripts/python_benchmark.py
```

**Metrics to Monitor:**
- Peak heap allocation
- Average heap usage
- GC pause time (Python)
- Memory growth rate

## Output Formats

### Criterion.rs Output (Rust)

Standard Criterion HTML reports in `target/criterion/`:
- Per-benchmark charts
- Historical trends
- Regression detection

### JSON Results

Machine-readable format:
```json
{
  "timestamp": "2024-03-24T...",
  "benchmarks": {
    "discovery": {
      "alpha_100k": {
        "time_mean": 1.234,
        "time_stdev": 0.045,
        "events": 100000,
        "throughput_events_per_sec": 81000
      }
    }
  }
}
```

### Markdown Comparison

Human-readable comparison tables:
- Speedup factors
- Timing comparisons
- Performance recommendations

## Optimization Opportunities

### Identified Bottlenecks

1. **Memory Allocation**
   - Use arena allocators for temporary objects
   - Reduce Vec reallocations in hot loops

2. **Graph Operations**
   - Cache common transitions
   - Use bit-packed sets for large transition matrices

3. **String Operations**
   - Intern activity names
   - Use Arc<str> for shared strings

4. **Algorithm-Specific**
   - Parallel processing for large logs
   - SIMD for distance calculations

### Future Improvements

- [ ] Parallel discovery algorithms
- [ ] Incremental conformance checking
- [ ] Streaming statistics computation
- [ ] GPU acceleration for large matrices

## CI/CD Integration

### GitHub Actions Workflow

Run benchmarks on every push:

```yaml
- name: Run benchmarks
  run: ./scripts/run_benchmarks.sh --rust-only

- name: Upload results
  uses: actions/upload-artifact@v2
  with:
    name: benchmark-results
    path: target/criterion
```

### Performance Regression Detection

```bash
cargo bench -- --baseline main
```

Automatic regression detection when:
- New benchmark > 10% slower than baseline
- New benchmark > 2 stdev slower than historical

## Troubleshooting

### Inconsistent Results

**Causes:**
- System load during benchmarking
- Thermal throttling
- Background processes

**Solutions:**
- Run on dedicated machine
- Close unnecessary applications
- Set CPU governor to performance mode
- Use isolated CPUs (Linux)

### Python Benchmark Failures

**Issue:** ImportError for pm4py
```bash
pip install pm4py psutil pandas
```

**Issue:** Slow Python benchmarks
- Use PyPy for faster execution
- Consider sampling smaller logs

### Rust Build Issues

**Issue:** Out of memory during compilation
```bash
# Use less parallelism
cargo bench -j 2
```

## References

### Documentation
- [Criterion.rs Guide](https://bheisler.github.io/criterion.rs/book/)
- [pm4py Documentation](https://pm4py.fit.fraunhofer.de/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

### Papers
- [pm4py: A Python Package for Process Mining](https://arxiv.org/abs/1905.06169)
- [Process Mining in Python](https://arxiv.org/abs/1905.06169)

## Contributing

To add new benchmarks:

1. Create benchmark file in `benches/`
2. Define test functions following Criterion pattern
3. Add to Cargo.toml `[[bench]]` section
4. Update this documentation

### Benchmark Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_algorithm(c: &mut Criterion) {
    let log = black_box(generate_test_log(100_000));

    c.bench_function("my_algorithm_100k", |b| {
        b.iter(|| {
            // Algorithm under test
        });
    });
}

criterion_group!(benches, bench_my_algorithm);
criterion_main!(benches);
```

## License

This benchmarking suite is part of pm4py-rust and follows the same AGPL-3.0+ license.
