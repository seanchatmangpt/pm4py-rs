# Load Testing Quick Start

## Overview

This guide helps you run load tests for pm4py-rust to validate system behavior under high concurrency and extreme conditions.

## Files & Test Locations

| Test File | Purpose | Location |
|-----------|---------|----------|
| **load_testing.rs** | Concurrent discovery, conformance, statistics | `tests/load_testing.rs` |
| **distributed_speedup_test.rs** | Multi-node parallelism validation | `tests/distributed_speedup_test.rs` |
| **memory_profiling_test.rs** | Memory usage profiling & optimization | `tests/memory_profiling_test.rs` |
| **stress_scenarios.rs** | Pathological & edge case handling | `tests/stress_scenarios.rs` |

## Running Load Tests

### 1. Run All Load Tests
```bash
cd pm4py-rust
cargo test --test load_testing --release -- --nocapture
```

### 2. Run Specific Concurrency Test
```bash
# Concurrent discovery (10 simultaneous)
cargo test --test load_testing test_concurrent_discovery_10_simultaneous --release -- --nocapture

# Concurrent conformance (50 simultaneous)
cargo test --test load_testing test_concurrent_conformance_50_simultaneous --release -- --nocapture

# Large log processing (100k events)
cargo test --test load_testing test_large_log_single_thread --release -- --nocapture
```

### 3. Run Memory Profiling
```bash
# 1M events EventLog profiling
cargo test --test memory_profiling_test profile_eventlog_1m_events --release -- --nocapture

# 100M events (large scale)
cargo test --test memory_profiling_test profile_eventlog_100m_events --release -- --nocapture --test-threads=1

# Complete memory analysis
cargo test --test memory_profiling_test --release -- --nocapture
```

### 4. Run Distributed Speedup Tests
```bash
# Single-node baseline (1M events)
cargo test --test distributed_speedup_test test_single_node_baseline_1m_events --release -- --nocapture

# 2-node speedup validation
cargo test --test distributed_speedup_test test_two_node_speedup_1m_events --release -- --nocapture

# Full speedup suite (2, 3, 5, 8 nodes)
cargo test --test distributed_speedup_test test_two_node_speedup_1m_events test_three_node_speedup_2m_events test_five_node_speedup_5m_events test_eight_node_speedup_10m_events --release -- --nocapture
```

### 5. Run Stress Scenario Tests
```bash
# Rapid-fire logging (5000 events burst)
cargo test --test stress_scenarios scenario_1_rapid_fire_logging --release -- --nocapture

# Memory pressure test
cargo test --test stress_scenarios scenario_2_memory_pressure --release -- --nocapture

# Pathological logs (deep, branching, loops)
cargo test --test stress_scenarios scenario_3_pathological_deep_sequence --release -- --nocapture
cargo test --test stress_scenarios scenario_3_pathological_branching --release -- --nocapture
cargo test --test stress_scenarios scenario_3_pathological_loops --release -- --nocapture

# Sustained load (1000 operations)
cargo test --test stress_scenarios scenario_5_sustained_load_1000_operations --release -- --nocapture
```

## Expected Test Results

### Concurrent Load Tests (load_testing.rs)

| Test | Scale | Target Time | Expectation |
|------|-------|-------------|------------|
| discovery_10 | 100 traces × 10 activities | < 30s | 10/10 success |
| discovery_50 | 100 traces × 10 activities | < 60s | 50/50 success |
| discovery_100 | 100 traces × 10 activities | < 60s | 100/100 success |
| conformance_10 | 100 traces × 10 activities | < 30s | ≥5 success |
| conformance_50 | 100 traces × 10 activities | < 30s | ≥25 success |
| conformance_100 | 100 traces × 10 activities | < 30s | ≥50 success |
| statistics_50 | 100 traces × 10 activities | < 30s | 50/50 success |
| mixed_ops | 30 discovery + 30 conformance + 30 stats | < 60s | All operations complete |
| large_log | 100k events single thread | < 60s | Model discovered |
| resource_contention | 30×3 threads (90 total) | < 180s | 90 ops completed |
| batch_discovery | 100 sequential operations | Any | 100/100 success |
| memory_stability | 1000 iterations | < 600s | 1000/1000 success |

### Distributed Speedup Tests (distributed_speedup_test.rs)

| Test | Nodes | Events | Target Speedup | Target Efficiency | Comment |
|------|-------|--------|----------------|-------------------|---------|
| baseline | 1 | 1M | — | — | Single-node reference |
| 2-node | 2 | 1M | ≥1.7x | ≥85% | Linear speedup is 2x |
| 3-node | 3 | 2M | ≥2.5x | ≥83% | Scaling with larger log |
| 5-node | 5 | 5M | ≥3.8x | ≥76% | Non-linear scaling |
| 8-node | 8 | 10M | ≥5.5x | ≥69% | Diminishing returns |

### Memory Profiling (memory_profiling_test.rs)

| Test | Input Size | Memory Target | Status |
|------|-----------|----------------|--------|
| EventLog 1M | 1M events | < 1200 MB | Should pass |
| EventLog 10M | 10M events | < 6000 MB | Should pass |
| EventLog 100M | 100M events | < 6000 MB | Should pass |
| PetriNet medium | 100 places/transitions | < 10 MB | Should pass |
| PetriNet high | 500 places/transitions | < 50 MB | Should pass |
| TokenReplay 1M | 1M events | < 1200 MB | Should pass |
| DFG Discovery 1M | 1M events | < 1200 MB | Should pass |

### Stress Scenarios (stress_scenarios.rs)

| Scenario | Scale | Success Criteria |
|----------|-------|------------------|
| Rapid-fire | 5000 events in 5sec | Model discovered, < 30s |
| Memory pressure | 50k events | Log allocated & freed |
| Deep sequence | 500 depth × 10 traces | Discovery completes < 60s |
| Branching | 500 traces × 100 branches | Discovery completes < 60s |
| Loops | 500 traces × 50 loops | Discovery completes < 60s |
| Fragmented | 5000 unique single-event traces | Discovery completes < 60s |
| Complex structure | 50 traces × 100 events | Discovery completes < 60s |
| Cascading failures | 30 threads × 3 types | System remains stable |
| Sustained load | 1000 operations | Success > 900, < 10min |
| 24-hr simulation | 100 concurrent × 30sec | Operations > 50, stable |

## Output Format

Each test prints results like:

```
=== CONCURRENT DISCOVERY: 10 Simultaneous ===
  Completed in: 5.234s
  Successes: 10, Errors: 0
```

or

```
=== Memory Profile: EventLog (1M events) ===
  Events: 1000000
  Traces: 10000
  Creation time: 2.34s
  Estimated memory: 450.25 MB
  ✓ Memory usage acceptable: 450.25MB
```

## Baseline Measurement

See `BASELINE_MEASUREMENT_TEMPLATES.md` for how to:

1. Capture baseline metrics from single-node runs
2. Format results for comparison
3. Track performance regressions

## Troubleshooting

### Tests Timeout
- Use `--test-threads=1` to run serially
- Increase timeout: check `assert!(duration < Duration::from_secs(X))`
- Check system load with `top` during tests

### Memory Tests Fail
- Check available heap: run on system with adequate RAM
- Reduce event counts in test if memory-constrained
- Enable swap space (not recommended for accurate profiling)

### Conformance Tests Return Low Scores
- This is expected for empty/simple models
- See `DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md` for fitness interpretation

## Next Steps

1. **Run baseline measurements** → `docs/diataxis/how-to/baseline-measurement.md`
2. **Analyze distributed speedup** → `docs/diataxis/how-to/speedup-validation.md`
3. **Optimize memory usage** → `docs/diataxis/explanation/memory-optimization.md`
4. **Understand cache optimization** → `docs/diataxis/explanation/cache-aware-optimization.md`
