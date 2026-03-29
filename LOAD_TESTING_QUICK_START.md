# Load Testing Quick Start Guide

## Prerequisites

```bash
# Ensure Rust 1.70+ is installed
rustc --version

# Navigate to pm4py-rust directory
cd /Users/sac/chatmangpt/pm4py-rust
```

## Run All Load Tests

```bash
# Compile and run (with output)
cargo test --test load_testing -- --nocapture --test-threads=2

# Takes approximately 3-5 minutes
```

## Run All Stress Scenarios

```bash
# Compile and run (with output)
cargo test --test stress_scenarios -- --nocapture --test-threads=1

# Takes approximately 5-8 minutes
```

## Run Specific Test Categories

### Concurrent Discovery Tests

```bash
# 10 simultaneous
cargo test --test load_testing test_concurrent_discovery_10_simultaneous -- --nocapture

# 50 simultaneous
cargo test --test load_testing test_concurrent_discovery_50_simultaneous -- --nocapture

# 100 simultaneous
cargo test --test load_testing test_concurrent_discovery_100_simultaneous -- --nocapture
```

### Concurrent Conformance Tests

```bash
# 10 simultaneous
cargo test --test load_testing test_concurrent_conformance_10_simultaneous -- --nocapture

# 50 simultaneous
cargo test --test load_testing test_concurrent_conformance_50_simultaneous -- --nocapture

# 100 simultaneous
cargo test --test load_testing test_concurrent_conformance_100_simultaneous -- --nocapture
```

### Mixed Operations

```bash
cargo test --test load_testing test_mixed_concurrent_operations -- --nocapture
```

### Stability Tests

```bash
# Large log (100k events)
cargo test --test load_testing test_large_log_single_thread -- --nocapture

# 1000 iterations memory stability
cargo test --test load_testing test_memory_stability_iterations -- --nocapture

# Resource contention (90 threads)
cargo test --test load_testing test_resource_contention_all_concurrent -- --nocapture
```

## Run Specific Stress Scenarios

### Rapid-Fire Logging

```bash
cargo test --test stress_scenarios scenario_1_rapid_fire_logging -- --nocapture
```

### Memory Pressure

```bash
cargo test --test stress_scenarios scenario_2_memory_pressure -- --nocapture
```

### Pathological Logs (All 5)

```bash
# Deep sequences
cargo test --test stress_scenarios scenario_3_pathological_deep_sequence -- --nocapture

# High branching
cargo test --test stress_scenarios scenario_3_pathological_branching -- --nocapture

# Loop-intensive
cargo test --test stress_scenarios scenario_3_pathological_loops -- --nocapture

# Fragmented traces
cargo test --test stress_scenarios scenario_3_pathological_fragmented -- --nocapture

# Complex structure
cargo test --test stress_scenarios scenario_3_pathological_complex -- --nocapture
```

### Cascading Failures

```bash
cargo test --test stress_scenarios scenario_4_cascading_failures -- --nocapture
```

### Sustained Load

```bash
# 1000 sequential operations
cargo test --test stress_scenarios scenario_5_sustained_load_1000_operations -- --nocapture

# 24-hour simulation (100 concurrent, 30 sec)
cargo test --test stress_scenarios scenario_5_24hr_simulation_concurrent -- --nocapture
```

## Performance Testing (Release Build)

For accurate performance metrics, use release build:

```bash
# Compile in release mode
cargo test --test load_testing --release -- --nocapture --test-threads=2

# Compile both test files in release
cargo test --test stress_scenarios --release -- --nocapture --test-threads=1
```

## Expected Output Examples

### Load Test Success
```
test test_concurrent_discovery_100_simultaneous ... ok
=== CONCURRENT DISCOVERY: 100 Simultaneous ===
  Completed in: 45.230s
  Successes: 100

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

### Stress Test Success
```
test scenario_1_rapid_fire_logging ... ok
=== SCENARIO 1: RAPID-FIRE LOGGING (5000 events burst) ===
  Created 5000 events in: 2.340s
  Event creation rate: 2136 events/sec

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured
```

## Interpreting Results

### Load Tests

✓ **PASS**: All concurrent operations complete within timeout
✗ **FAIL**: Timeout or assertion failure indicates:
  - Deadlock (operation hangs)
  - Data corruption (assertion fails)
  - Resource exhaustion (panic)

### Stress Tests

✓ **PASS**: Operation completes without panic, gracefully handles errors
✗ **FAIL**: Panic or unexpected error indicates:
  - Unhandled pathological case
  - Memory exhaustion (OOM)
  - Algorithm failure on edge case

## Performance Benchmarks

Typical performance on modern hardware (Apple Silicon M2 Pro):

| Test | Concurrency | Time |
|------|-------------|------|
| Discovery 10 | 10 threads | ~15s |
| Discovery 100 | 100 threads | ~45s |
| Conformance 100 | 100 threads | ~25s |
| Mixed 30 | 30 threads | ~40s |
| Large log 100k | 1 thread | ~35s |
| Memory 1000 | 1 thread | ~4-5 min |
| Pathological deep | 1 thread | ~20s |
| Sustained 1000 | 1 thread | ~8-10 min |

*Note: Timings vary by hardware; focus on completion rather than absolute time.*

## Troubleshooting

### Test Timeout

If tests timeout:

1. Reduce concurrency: edit test file and lower thread count
2. Increase timeout: modify `assert!(elapsed.as_secs() < X)` constants
3. Check system resources: ensure enough CPU cores available

### Memory Issues

If memory tests fail:

```bash
# Monitor system memory during test
cargo test --test load_testing test_memory_stability_iterations -- --nocapture &
watch -n 1 'free -h'
```

### Cascading Failures Expected

Cascading failure test expects some errors to occur - this is normal.
Look for "System remained stable" message.

## CI/CD Integration

### GitHub Actions

```yaml
name: Load Tests
on: [push, pull_request]

jobs:
  load-test:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --test load_testing -- --nocapture
      - run: cargo test --test stress_scenarios -- --nocapture
```

## Next Steps

1. **Run baseline**: Execute all tests once to establish baseline metrics
2. **Monitor trends**: Track performance over time as code evolves
3. **Profile hotspots**: Use `perf` or `flamegraph` on slow tests
4. **Tune parameters**: Adjust load patterns based on your use case
5. **Integrate to CI**: Add tests to continuous integration pipeline

## Additional Resources

- **Full documentation**: See `LOAD_TESTING_SUMMARY.md`
- **Test source**: `/tests/load_testing.rs` and `/tests/stress_scenarios.rs`
- **Performance analysis**: Run with `--nocapture` to see detailed timing
- **Memory analysis**: Use `valgrind` or `heaptrack` for leak detection

---

**Ready to load test?** Run:
```bash
cargo test --test load_testing -- --nocapture
```
