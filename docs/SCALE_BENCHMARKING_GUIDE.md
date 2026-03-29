# Scale Testing and Performance Benchmarking Guide

## Overview

This guide explains the comprehensive scale testing and benchmarking infrastructure for pm4py-rust, designed to verify performance characteristics at enterprise scales.

## Test Coverage

### 1. Scale Test Suite (`tests/scale_benchmarks_test.rs`)

**15 tests covering 4 log size categories and stress scenarios**

#### Baseline Tests (100K events - target <1 sec)
- `test_discovery_alpha_100k_baseline` - Alpha Miner discovery
- `test_discovery_dfg_100k_baseline` - DFG Miner discovery
- `test_conformance_token_replay_100k_baseline` - Token replay conformance

#### Standard Enterprise Tests (1M events - target <5 sec)
- `test_discovery_alpha_1m_enterprise` - Alpha Miner on 1M events
- `test_discovery_inductive_1m_enterprise` - Inductive Miner on complex patterns
- `test_discovery_dfg_1m_enterprise` - DFG Miner on 1M events
- `test_conformance_token_replay_1m_enterprise` - Token replay on 1M events

#### Large Organization Tests (10M events - target <30 sec)
- `test_discovery_alpha_10m_large` - Alpha Miner on 10M events
- `test_discovery_dfg_10m_large` - DFG Miner on 10M events

#### Scalability Tests
- `test_scalability_alpha_miner_linear` - Verifies linear scaling (100K → 1M)
- `test_scalability_dfg_miner_linear` - Verifies linear scaling for DFG

#### Accuracy Tests
- `test_accuracy_fitness_synthetic_log` - Fitness validation
- `test_accuracy_complex_patterns` - Complex pattern handling

#### Stress Tests
- `test_stress_many_activities` - High activity diversity (20+ activities)
- `test_stress_many_traces` - Many short traces (50K traces)
- `test_stress_long_traces` - Very long traces (10 events per trace)

### 2. Benchmark Suite (`benches/scale_benchmarks.rs`)

**13 benchmark groups with Criterion.rs for detailed statistical analysis**

#### Discovery Algorithm Benchmarks
- Alpha Miner: 100K, 1M, 10M events
- Inductive Miner: 100K, 1M events
- DFG Miner: 100K, 1M, 10M events

#### Conformance Checking Benchmarks
- Token Replay: 100K, 1M events

#### Scalability Analysis
- `scalability_alpha`: Linear growth analysis (100K → 1M)
- `scalability_dfg`: Linear growth analysis (100K → 1M → 10M)

#### Throughput Analysis
- Events per second metrics for DFG Miner

## Execution

### Running Scale Tests

```bash
# Run all scale tests
cd /Users/sac/chatmangpt/pm4py-rust
cargo test --test scale_benchmarks_test -- --nocapture

# Run specific test
cargo test --test scale_benchmarks_test test_discovery_alpha_1m_enterprise -- --nocapture

# Run with verbose output
cargo test --test scale_benchmarks_test -- --nocapture --test-threads=1
```

### Running Benchmarks

```bash
# Run all benchmarks (time intensive, ~1-2 hours)
cargo bench --bench scale_benchmarks

# Run specific benchmark group
cargo bench --bench scale_benchmarks scalability_alpha

# Run single benchmark
cargo bench --bench scale_benchmarks alpha_miner_100k

# Generate HTML reports
cargo bench --bench scale_benchmarks -- --verbose
# Reports available in: target/criterion/
```

### Abbreviated Runs for Development

```bash
# Quick validation (baseline tests only)
cargo test --test scale_benchmarks_test test_discovery_alpha_100k_baseline -- --nocapture

# Quick benchmark (100K events only)
cargo bench --bench scale_benchmarks alpha_miner_100k

# Run with sample_size=1 for faster iteration
cargo bench --bench scale_benchmarks dfg_miner_100k -- --sample-size=1
```

## Metrics Collected

### 1. Timing Metrics

| Metric | Collected | Target |
|--------|-----------|--------|
| **CPU seconds** | All tests/benchmarks | <1s (100K), <5s (1M), <30s (10M) |
| **Wall-clock time** | Criterion benchmarks | Reported in HTML reports |
| **Throughput** | DFG benchmarks | Events/sec metric |

### 2. Accuracy Metrics

| Metric | Method | Target |
|--------|--------|--------|
| **Fitness** | Token Replay | >0.5 for synthetic, >0.7 baseline |
| **Precision** | Conformance checking | Validated via fitness |
| **Generalization** | Model capability | Tested on complex patterns |

### 3. Scalability Metrics

| Metric | Formula | Target |
|--------|---------|--------|
| **Scaling ratio** | Time(1M) / Time(100K) | <20x (linear preferred) |
| **Complexity class** | Log(Time) / Log(Events) | <1.5 (sub-linear) |
| **Memory efficiency** | Peak heap / Events | Constant per event |

### 4. Stress Metrics

| Scenario | Metric | Target |
|----------|--------|--------|
| **High activity diversity** | Time for 20 activities | <1s per 500K events |
| **Many traces** | Time for 50K traces | <1s per 100K events |
| **Long traces** | Time per 10K-event trace | <100ms |

## Test Data Characteristics

### Synthetic Log Generation

```rust
generate_synthetic_log(num_events, num_traces, num_activities)
  ├── Parameters:
  │   ├── num_events: Total event count
  │   ├── num_traces: Case count
  │   └── num_activities: Unique activity types
  ├── Pattern: A→B→C→D→E (rotating, num_activities-bounded)
  ├── Traces: Evenly distributed
  └── Timestamps: 1-second intervals per event
```

### Complex Log Generation

```rust
generate_complex_log(num_events, num_traces)
  ├── Pattern: Start→Initialize→[Path_C|Path_D]→Process→[Alt_F|Alt_G]→Complete
  ├── Branching: 50% Path_C vs Path_D (random by trace)
  ├── Rework: 33% of traces have alternative paths
  └── Timestamps: 5-second intervals between events
```

## Interpretation of Results

### Performance Targets

| Size | Target | Status |
|------|--------|--------|
| 100K events | <1 sec | Baseline (0.1-0.5s typical) |
| 1M events | <5 sec | Enterprise (0.5-3s typical) |
| 10M events | <30 sec | Large org (5-25s typical) |
| 100M events | <5 min | Distributed (requires special mode) |

### Red Flags

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| >20x scaling ratio (100K→1M) | Quadratic algorithm complexity | Optimize algorithm |
| >50% regression vs Python | Algorithmic difference | Review translation |
| OOM on 10M events | Unbounded memory growth | Implement streaming |
| Fitness <0.5 on synthetic | Model discovery bug | Verify discovery algorithm |

## Implementation Details

### Criterion.rs Configuration

- **Measurement time**: 10 seconds per benchmark
- **Sample size**: 10 iterations (reduced for large datasets)
- **Output**: HTML reports in `target/criterion/`
- **Confidence level**: 95%

### Black Box Optimization Prevention

```rust
let log = black_box(log);  // Prevents compiler optimization away
let miner = AlphaMiner::new();
miner.discover(&log)       // Time this operation
```

### Warm-up Runs

Criterion automatically performs warm-up iterations to stabilize CPU/caches.

## Continuous Integration

### GitHub Actions Integration

Add to `.github/workflows/bench.yml`:

```yaml
- name: Run scale benchmarks
  run: cargo bench --bench scale_benchmarks

- name: Run scale tests
  run: cargo test --test scale_benchmarks_test -- --nocapture
```

### Baseline Comparison

```bash
# Save baseline
cargo bench --bench scale_benchmarks -- --save-baseline my_baseline

# Compare against baseline
cargo bench --bench scale_benchmarks -- --baseline my_baseline
```

## Expected Performance Summary

### Alpha Miner
- 100K: 0.1-0.3s
- 1M: 0.5-1.5s
- 10M: 5-15s
- Scaling: ~O(n log n) or O(n^1.2)

### DFG Miner
- 100K: 0.05-0.2s
- 1M: 0.2-0.8s
- 10M: 1-5s
- Scaling: ~O(n) or O(n log n)

### Inductive Miner
- 100K: 0.2-0.5s
- 1M: 1-3s
- Scaling: ~O(n log n)

### Token Replay
- 100K: 0.1-0.3s
- 1M: 0.5-1.5s
- Scaling: ~O(n)

## Troubleshooting

### Benchmark Times Vary Wildly

**Cause**: CPU throttling, background processes
**Fix**:
- Run on quiet system
- Disable turbo boost: `echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo`
- Use `taskset` to pin to specific cores

### Out of Memory on Large Sizes

**Cause**: Insufficient RAM or algorithm has memory leak
**Fix**:
- Reduce size (100K instead of 10M)
- Use `valgrind` to detect leaks
- Check for unbounded data structures

### Fitness Too Low

**Cause**: Discovery algorithm not finding good model or conformance bug
**Fix**:
- Verify log generation is realistic
- Check discovery output (places, transitions)
- Run Python pm4py on same log for comparison

## Python Baseline Comparison

To compare against Python pm4py:

```python
# python_baseline.py
import pm4py
import time

# Generate synthetic log
log = ...  # 1M events
start = time.time()
net, im, fm = pm4py.discover_petri_net_alpha(log)
elapsed = time.time() - start

print(f"Python Alpha: {elapsed:.2f}s")
print(f"Fitness: {pm4py.fitness_token_based_replay(log, net, im, fm)}")
```

Then compare with Rust results in `target/criterion/report/`.

## File Structure

```
pm4py-rust/
├── tests/
│   └── scale_benchmarks_test.rs     # 15 scale tests
├── benches/
│   ├── scale_benchmarks.rs          # 13 benchmark groups
│   ├── discovery.rs                 # Existing benchmarks
│   └── ...
├── docs/
│   └── SCALE_BENCHMARKING_GUIDE.md  # This file
└── target/
    └── criterion/                   # HTML reports after bench run
        └── report/
            └── index.html
```

## See Also

- `tests/extended_discovery_integration_tests.rs` - Real-world discovery tests
- `benches/discovery.rs` - Basic discovery benchmarks
- `benches/conformance.rs` - Conformance benchmarks
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)

## Contact

Questions or issues? Open an issue on GitHub:
https://github.com/seanchatmangpt/pm4py-rust/issues
