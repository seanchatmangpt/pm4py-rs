# Scale Testing and Performance Benchmarking Results

## Summary

This document template shows how to report results from scale testing and benchmarking runs.

## Test Execution Results

### Scale Tests (15 comprehensive tests)

Run scale tests with:
```bash
cd /Users/sac/chatmangpt/pm4py-rust
cargo test --test scale_benchmarks_test --release -- --nocapture
```

#### Expected Results for Each Test Category

**Baseline Tests (100K events)**
- `test_discovery_alpha_100k_baseline`: <1s ✓
- `test_discovery_dfg_100k_baseline`: <500ms ✓
- `test_conformance_token_replay_100k_baseline`: <2s ✓

**Enterprise Tests (1M events)**
- `test_discovery_alpha_1m_enterprise`: <5s
- `test_discovery_inductive_1m_enterprise`: <10s
- `test_discovery_dfg_1m_enterprise`: <3s
- `test_conformance_token_replay_1m_enterprise`: <5s

**Large Organization Tests (10M events)**
- `test_discovery_alpha_10m_large`: <30s
- `test_discovery_dfg_10m_large`: <15s

**Scalability Tests**
- `test_scalability_alpha_miner_linear`: Ratio <20x (100K→1M)
- `test_scalability_dfg_miner_linear`: Ratio <15x (100K→1M)

**Accuracy Tests**
- `test_accuracy_fitness_synthetic_log`: Fitness >0.7
- `test_accuracy_complex_patterns`: Fitness >0.5

**Stress Tests**
- `test_stress_many_activities`: 500K with 20 activities <10s
- `test_stress_many_traces`: 100K across 50K traces <5s
- `test_stress_long_traces`: 100K across 10 long traces <5s

### Benchmark Results (Criterion.rs)

Run benchmarks with:
```bash
cd /Users/sac/chatmangpt/pm4py-rust
cargo bench --bench scale_benchmarks
```

View detailed HTML reports in: `target/criterion/report/index.html`

## Metrics Collection Template

When running benchmarks, collect these metrics:

| Algorithm | Size | Time (mean) | Throughput | Memory | Accuracy |
|-----------|------|------------|-----------|--------|----------|
| Alpha | 100K | | events/s | | |
| Alpha | 1M | | events/s | | |
| Alpha | 10M | | events/s | | |
| DFG | 100K | | events/s | | |
| DFG | 1M | | events/s | | |
| DFG | 10M | | events/s | | |
| Inductive | 100K | | events/s | | |
| Inductive | 1M | | events/s | | |
| Token Replay | 100K | | events/s | | |
| Token Replay | 1M | | events/s | | |

## Scalability Analysis

### Linear Scaling Check

Alpha Miner:
- 100K: baseline
- 1M: should be <20x baseline
- Ratio: ___x (linear desired, <20x acceptable)

DFG Miner:
- 100K: baseline
- 1M: should be <15x baseline
- 10M: should be <100x baseline (from 100K)
- 100K→1M ratio: ___x
- 1M→10M ratio: ___x

## Bottleneck Identification

### If Performance is Suboptimal

1. **Algorithm complexity**: Check Big-O growth vs event count
2. **Memory allocation**: Use `valgrind` or profiler
3. **String/Vector operations**: Look for unbounded growth
4. **Transition lookup**: Verify hash-based access (not linear)

### Profiling Commands

```bash
# Profile with perf (Linux only)
perf record cargo test --test scale_benchmarks_test test_discovery_alpha_1m_enterprise --release
perf report

# Profile with Instruments (macOS)
cargo instruments -t "System Trace" --test scale_benchmarks_test

# Memory profiling with valgrind (Linux)
valgrind --tool=massif cargo test --test scale_benchmarks_test test_discovery_alpha_1m_enterprise --release
ms_print massif.out.<pid>
```

## Comparison with Python Baseline

To compare Rust performance with Python pm4py:

```python
# python_baseline.py
import pm4py
import time
import random

# Generate 1M event log
log = pm4py.objects.log.event_log.EventLog()
for case_id in range(10_000):
    trace = pm4py.objects.log.event_log.Trace()
    for event_idx in range(100):
        evt = pm4py.objects.log.event_log.Event()
        evt['case:concept:name'] = f'case_{case_id}'
        evt['concept:name'] = ['A', 'B', 'C', 'D', 'E'][event_idx % 5]
        trace.append(evt)
    log.append(trace)

# Time Alpha Miner
start = time.time()
net, im, fm = pm4py.discover_petri_net_alpha(log)
elapsed = time.time() - start

print(f"Python Alpha 1M: {elapsed:.2f}s")

# Time Token Replay
fitness = pm4py.fitness_token_based_replay(log, net, im, fm)
print(f"Fitness: {fitness['average_trace_fitness']:.4f}")
```

Then compare with Rust benchmark output.

## Red Flags and Solutions

| Issue | Indicator | Solution |
|-------|-----------|----------|
| Quadratic scaling | 100K→1M ratio >20x | Review algorithm, check for nested loops |
| Memory leak | OOM on 10M | Check for unbounded HashMap/Vec growth |
| Inefficient lookup | Slow discovery on high-activity logs | Use HashMap instead of Vec::contains |
| Conformance bottleneck | Token Replay slower than discovery | Optimize transition lookup |
| Garbage collection | Sawtooth performance pattern | Reduce allocation churn |

## Next Steps

1. **Establish baseline**: Run tests on stable hardware, document results
2. **Continuous monitoring**: Add benchmarks to CI/CD
3. **Regression detection**: Use Criterion baseline comparison
4. **Optimization**: Profile bottlenecks and iteratively improve
5. **Documentation**: Update this file with actual results

## References

- Full guide: `docs/SCALE_BENCHMARKING_GUIDE.md`
- Test file: `tests/scale_benchmarks_test.rs`
- Benchmark file: `benches/scale_benchmarks.rs`
- Criterion.rs docs: https://bheisler.github.io/criterion.rs/book/
