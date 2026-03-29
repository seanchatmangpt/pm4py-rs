# Performance Guide Index

Comprehensive index linking all load testing, benchmarking, and optimization documentation.

## Quick Navigation

### Start Here

**First Time?** → [Load Testing Quick Start](../tutorials/load-testing-quickstart.md)
- How to run tests
- Expected results
- Troubleshooting

### How-To Guides

**Running Baselines** → [Baseline Measurement Guide](../how-to/baseline-measurement.md)
- Capture baseline metrics
- Compare against regressions
- Create benchmark scripts

**Validating Speedup** → [Speedup Validation Guide](../how-to/speedup-validation.md)
- Interpret distributed speedup results
- Analyze efficiency curves
- Create speedup reports

### Explanations

**Memory Architecture** → [Memory Optimization](../explanation/memory-optimization.md)
- 6 optimization techniques
- Before/after examples
- Implementation details

**Cache Efficiency** → [Cache-Aware Optimization](../explanation/cache-aware-optimization.md)
- CPU cache fundamentals
- 6 cache optimization patterns
- Profiling with perf

### Test Files & Documentation

## Test Organization

```
tests/
├── load_testing.rs                    ← Concurrent load tests
├── distributed_speedup_test.rs        ← Multi-node speedup validation
├── memory_profiling_test.rs           ← Memory usage analysis
├── stress_scenarios.rs                ← Edge cases & pathological logs
└── memory_allocator_test.rs           ← Allocator unit tests

src/
├── optimization/
│   ├── cache_aware.rs                 ← Adjacency caching, alignment
│   └── hotspot_elimination.rs         ← BFS, memoization, aggregation
└── memory/
    └── allocator.rs                   ← String interning, attributes, adjacency
```

## Reference Files (docs/)

```
docs/
├── PERFORMANCE.md                     ← Overview & benchmarks
├── PERFORMANCE_BENCHMARKING.md        ← Benchmark methodology
├── PERFORMANCE_RESULTS_TEMPLATE.md    ← Result formatting
├── PERFORMANCE_TUNING_GUIDE.md        ← Tuning parameters
├── VARIANT_PERFORMANCE_ANALYSIS.md    ← Variant-specific analysis
├── OPTIMIZATION_ANALYSIS.md           ← Optimization deep dives
└── diataxis/
    ├── tutorials/
    │   └── load-testing-quickstart.md       ← THIS SECTION
    ├── how-to/
    │   ├── baseline-measurement.md
    │   └── speedup-validation.md
    ├── explanation/
    │   ├── memory-optimization.md
    │   └── cache-aware-optimization.md
    └── reference/
        ├── performance-guide-index.md       ← YOU ARE HERE
        └── load-test-matrix.md
```

## Test Matrix Reference

### Concurrent Load Tests (load_testing.rs)

| Test Name | Scale | Focus | Target | File |
|-----------|-------|-------|--------|------|
| discovery_10 | 10 simultaneous | Basic concurrency | < 30s | load_testing.rs:109 |
| discovery_50 | 50 simultaneous | Moderate load | < 60s | load_testing.rs:161 |
| discovery_100 | 100 simultaneous | High load | < 60s | load_testing.rs:205 |
| conformance_10 | 10 simultaneous | Token replay | < 30s | load_testing.rs:249 |
| conformance_50 | 50 simultaneous | Medium conformance | < 30s | load_testing.rs:303 |
| conformance_100 | 100 simultaneous | Heavy conformance | < 30s | load_testing.rs:351 |
| statistics_50 | 50 simultaneous | Parallel stats | < 30s | load_testing.rs:399 |
| mixed_ops | 30+30+30 threads | All operations | < 60s | load_testing.rs:445 |
| large_log | 100k events | Single thread | < 60s | load_testing.rs:538 |
| resource_contention | 90 threads total | Max contention | < 180s | load_testing.rs:570 |
| batch_discovery | 100 sequential | Sequential load | Any | load_testing.rs:645 |
| memory_stability | 1000 iterations | Memory leaks | < 600s | load_testing.rs:676 |

### Distributed Speedup Tests (distributed_speedup_test.rs)

| Test Name | Nodes | Events | Speedup Target | Efficiency Target | File |
|-----------|-------|--------|-----------------|-------------------|------|
| baseline_1m | 1 | 1M | — | — | distributed_speedup_test.rs:297 |
| 2node_1m | 2 | 1M | ≥1.7x | ≥85% | distributed_speedup_test.rs:316 |
| 3node_2m | 3 | 2M | ≥2.5x | ≥83% | distributed_speedup_test.rs:354 |
| 5node_5m | 5 | 5M | ≥3.8x | ≥76% | distributed_speedup_test.rs:384 |
| 8node_10m | 8 | 10M | ≥5.5x | ≥69% | distributed_speedup_test.rs:414 |
| partition_correctness | 2-8 | 100k | All patterns | 100% | distributed_speedup_test.rs:444 |
| merge_completeness | 5 | 500k | All edges | Match single | distributed_speedup_test.rs:487 |
| byzantine_tolerance | 5 | 2M | Tolerate 2 | > baseline/3 | distributed_speedup_test.rs:520 |
| scalability_curve | 2,4,6,8 | 5M | Smooth | Smooth | distributed_speedup_test.rs:553 |
| e2e_distributed | 4 | 1M | Diff < 1% | Fitness match | distributed_speedup_test.rs:597 |

### Memory Profiling Tests (memory_profiling_test.rs)

| Test Name | Input Size | Memory Target | Comment | File |
|-----------|-----------|----------------|---------|------|
| eventlog_1m | 1M events | < 1200 MB | Standard | memory_profiling_test.rs:160 |
| eventlog_10m | 10M events | < 6000 MB | High volume | memory_profiling_test.rs:189 |
| eventlog_100m | 100M events | < 6000 MB | Large scale | memory_profiling_test.rs:212 |
| petri_medium | 100+100 | < 10 MB | Medium net | memory_profiling_test.rs:239 |
| petri_high | 500+500 | < 50 MB | Complex net | memory_profiling_test.rs:262 |
| conformance_1m | 1M events | < 1200 MB | TokenReplay | memory_profiling_test.rs:289 |
| dfg_1m | 1M events | < 1200 MB | Discovery | memory_profiling_test.rs:318 |
| stats_1m | 1M events | < 1200 MB | Statistics | memory_profiling_test.rs:345 |
| memory_consumers | 100k | Identify top 5 | Analysis | memory_profiling_test.rs:371 |
| correctness_verify | 10k | Verify integrity | Sanity check | memory_profiling_test.rs:411 |

### Stress Scenario Tests (stress_scenarios.rs)

| Test Name | Scenario | Scale | Success Criteria | File |
|-----------|----------|-------|-----------------|------|
| rapid_fire | Burst 5000 events | 5k events | Create + discover < 30s | stress_scenarios.rs:169 |
| memory_pressure | Large allocation | 50k events | Allocate & free | stress_scenarios.rs:219 |
| deep_sequence | Pathological | 500 deep × 10 | Discover < 60s | stress_scenarios.rs:273 |
| branching | High branching | 500 × 100 branches | Discover < 60s | stress_scenarios.rs:294 |
| loops | Loop intensive | 500 × 50 loops | Discover < 60s | stress_scenarios.rs:315 |
| fragmented | Many unique | 5000 single-event | Discover < 60s | stress_scenarios.rs:336 |
| complex_structure | Complex nesting | 50 × 100 events | Discover < 60s | stress_scenarios.rs:357 |
| cascading_failures | Multiple errors | 30 threads × 3 types | System stable | stress_scenarios.rs:382 |
| sustained_load | Long running | 1000 operations | Success > 90% | stress_scenarios.rs:477 |
| 24hr_simulation | Concurrent | 100 threads × 30s | Ops > 50 | stress_scenarios.rs:526 |

## Optimization Implementation Map

### By Component

**EventLog (40% reduction target)**
- String interning: `src/memory/allocator.rs::StringIntern`
- Attribute deduplication: `src/memory/allocator.rs::CompactAttributes`
- Related tests: `memory_profiling_test.rs` (profile_eventlog_*)

**PetriNet (20% reduction target)**
- Adjacency caching: `src/optimization/cache_aware.rs::OptimizedPetriNet`
- Adjacency lists: `src/memory/allocator.rs::AdjacencyLists`
- Related tests: `memory_profiling_test.rs` (profile_petri_net_*)

**Conformance Checking (30% reduction target)**
- Result streaming: `src/conformance/*.rs`
- Cache alignment: `src/optimization/cache_aware.rs::CacheAlignedMarking`
- Related tests: `memory_profiling_test.rs` (profile_conformance_*)

**Statistics (25% reduction target)**
- Incremental aggregation: `src/statistics/*.rs`
- Related tests: `memory_profiling_test.rs` (profile_statistics_*)

**Discovery (General optimizations)**
- Hotspot elimination: `src/optimization/hotspot_elimination.rs`
- Parallel detection: `src/optimization/cache_aware.rs::ParallelActivityDetector`
- Single-scan aggregation: `src/optimization/hotspot_elimination.rs::SingleScanAggregator`

## Quick Reference: Run Commands

### Run All Tests
```bash
# All load tests
cargo test --test load_testing --release -- --nocapture

# All speedup tests
cargo test --test distributed_speedup_test --release -- --nocapture

# All memory tests
cargo test --test memory_profiling_test --release -- --nocapture

# All stress tests
cargo test --test stress_scenarios --release -- --nocapture
```

### Run Specific Test Category
```bash
# Concurrent discovery (10, 50, 100)
cargo test --test load_testing test_concurrent_discovery --release -- --nocapture

# Distributed speedup (2, 3, 5, 8 nodes)
cargo test --test distributed_speedup_test test_.*_speedup --release -- --nocapture

# Memory profiling at different scales
cargo test --test memory_profiling_test test_eventlog --release -- --nocapture

# Stress scenarios
cargo test --test stress_scenarios scenario --release -- --nocapture
```

### Run with Baseline Comparison
```bash
# Store baseline
mkdir -p .benchmarks
cargo test --test load_testing test_large_log_single_thread --release -- --nocapture 2>&1 | tee .benchmarks/baseline.log

# Compare new results
cargo test --test load_testing test_large_log_single_thread --release -- --nocapture 2>&1 | tee .benchmarks/current.log

# Calculate regression (script in baseline-measurement.md)
```

## Baseline Measurement Templates

See `BASELINE_MEASUREMENT_TEMPLATES.md` for:
- Baseline capture procedures
- Result formatting
- Regression detection
- Comparison scripts

## Distributed Speedup Validation Reports

See `DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md` for:
- Speedup targets and interpretation
- Efficiency curve analysis
- Byzantine fault tolerance
- Scalability degradation analysis

## Performance Tuning Parameters

Key tunables in test files:

| Parameter | File | Purpose | Default |
|-----------|------|---------|---------|
| num_traces | load_testing.rs:31 | Log generation size | 100 |
| events_per_trace | load_testing.rs:31 | Trace length | 10 |
| num_nodes | distributed_speedup_test.rs:125 | Partition count | 2-8 |
| max_depth | hotspot_elimination.rs:21 | BFS depth limit | 10 |
| cache_size | — | Object pool capacity | 100-1000 |

## Related Documentation

- **PM4PY Python Parity**: `docs/PM4PY_PARITY_GAP_ANALYSIS.md`
- **Algorithm Deep Dive**: `docs/ALGORITHM_DEEPDIVE.md`
- **Complete Getting Started**: `docs/COMPLETE_GETTING_STARTED.md`
- **Troubleshooting**: `docs/COMPREHENSIVE_TROUBLESHOOTING.md`

## Key Metrics to Track

When optimizing, monitor these metrics:

**Performance Metrics**
- Discovery time (target: 10s for 1M events)
- Conformance time (target: 8s for 1M events)
- Memory usage (target: 450 MB for 1M events)
- Speedup efficiency (target: ≥70% at 5 nodes)

**Correctness Metrics**
- Model size (places, transitions)
- Fitness scores (should remain unchanged)
- Edge preservation (100% on merge)
- Trace coverage (100%)

**Scalability Metrics**
- Memory per event (should decrease with optimization)
- Time per event (should remain flat or decrease)
- Cache hit rate (should increase)
- Throughput (events/second)

## How to Contribute

To add a new performance test:

1. Create test in appropriate file (`load_testing.rs`, `stress_scenarios.rs`, etc.)
2. Add to this index with file line numbers
3. Document expected results and targets
4. Add to baseline measurement template
5. Update quick reference commands

## Navigation Map

```
START HERE
    ↓
Load Testing Quick Start (tutorials/load-testing-quickstart.md)
    ↓
    ├→ Run baseline measurements (how-to/baseline-measurement.md)
    │  ↓
    │  Compare against previous (baseline-measurement.md)
    │
    ├→ Validate distributed speedup (how-to/speedup-validation.md)
    │  ↓
    │  Analyze efficiency curves
    │
    ├→ Optimize memory (explanation/memory-optimization.md)
    │  ↓
    │  Implement techniques 1-6
    │
    └→ Improve cache efficiency (explanation/cache-aware-optimization.md)
       ↓
       Implement patterns 1-6

Profile Results
    ↓
    ├→ Update BASELINE_MEASUREMENT_TEMPLATES.md
    └→ Update DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md
```
