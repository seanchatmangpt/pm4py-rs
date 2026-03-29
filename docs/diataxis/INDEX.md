# PM4PY-Rust Diataxis Documentation Index

Comprehensive Diátaxis-structured documentation for process mining, load testing, and performance optimization.

## What's New (2026-03-25)

**Four new guides for process mining fundamentals:**

1. **Tutorial:** [First Process Mining Analysis](tutorials/first-process-mining-analysis.md) — Load an XES file and extract statistics in <15 minutes
2. **How-To:** [Extract Process Patterns](how-to/extract-process-patterns.md) — Find common activity sequences and export to CSV
3. **Reference:** [Rust API Complete](reference/rust-api-complete.md) — Full API documentation with examples (Event, Trace, EventLog, VariantAnalysis, AlphaMiner, etc.)
4. **Reference:** [XES Format Guide](reference/xes-format-guide.md) — XES XML structure, attributes, security, and real-world examples

**Target Audience:** New users learning process mining with pm4py-rust

## Documentation Structure

This documentation follows the **Diátaxis** framework: tutorials, how-to guides, explanations, and references for optimal learning and reference.

```
docs/diataxis/
├── INDEX.md                                    ← YOU ARE HERE
│
├── tutorials/
│   ├── load-testing-quickstart.md             [Learn to run load tests]
│   └── first-process-mining-analysis.md       [NEW: Load XES & extract stats in 15 min]
│
├── how-to/
│   ├── baseline-measurement.md                [Capture & use baselines]
│   ├── speedup-validation.md                  [Interpret speedup results]
│   └── extract-process-patterns.md            [NEW: Find common activity sequences]
│
├── explanation/
│   ├── memory-optimization.md                 [Why & how memory optimization works]
│   └── cache-aware-optimization.md            [CPU cache efficiency patterns]
│
└── reference/
    ├── performance-guide-index.md             [Complete reference matrix]
    ├── rust-api-complete.md                   [NEW: Full API reference]
    └── xes-format-guide.md                    [NEW: XES structure & standards]
```

## Quick Start

**I want to:** → **Read this:**

**New to pm4py-rust?**
- **Analyze your first process** → [First Process Mining Analysis](tutorials/first-process-mining-analysis.md)
- **Extract process patterns** → [Extract Process Patterns](how-to/extract-process-patterns.md)
- **Understand XES format** → [XES Format Guide](reference/xes-format-guide.md)
- **See all API methods** → [Rust API Complete](reference/rust-api-complete.md)

**Performance & Testing?**
- **Run load tests** → [Load Testing Quick Start](tutorials/load-testing-quickstart.md)
- **Measure performance baseline** → [Baseline Measurement Guide](how-to/baseline-measurement.md)
- **Validate distributed speedup** → [Speedup Validation Guide](how-to/speedup-validation.md)
- **Understand memory optimization** → [Memory Optimization](explanation/memory-optimization.md)
- **Learn cache efficiency** → [Cache-Aware Optimization](explanation/cache-aware-optimization.md)
- **Find test details** → [Performance Guide Index](reference/performance-guide-index.md)

## Document Roadmap

### 1. Tutorials (Learning by Doing)

**[Load Testing Quick Start](tutorials/load-testing-quickstart.md)** (~15 min read)

Learn to:
- Run concurrent load tests (10, 50, 100 simultaneous operations)
- Run distributed speedup tests (2, 3, 5, 8 nodes)
- Run memory profiling at different scales
- Run stress scenario tests (pathological logs, cascading failures)
- Interpret test output

**Test files covered:**
- `tests/load_testing.rs` — Concurrent operations
- `tests/distributed_speedup_test.rs` — Multi-node speedup
- `tests/memory_profiling_test.rs` — Memory usage profiling
- `tests/stress_scenarios.rs` — Edge cases

---

### 2. How-To Guides (Goal-Oriented)

**[Baseline Measurement Guide](how-to/baseline-measurement.md)** (~20 min)

Learn to:
- Prepare test environment (disable frequency scaling, etc.)
- Run single-node baselines (discovery, conformance, statistics)
- Capture memory baselines at 1M, 10M, 100M event scales
- Run distributed speedup baseline
- Create baseline report with templates
- Store baseline for comparison
- Compare against previous baseline
- Detect performance regressions

**Key templates included:**
- System information template
- Single-node baseline template
- Memory profiling template
- Distributed speedup template
- JSON export format
- Python comparison script

---

**[Speedup Validation Guide](how-to/speedup-validation.md)** (~20 min)

Learn to:
- Calculate speedup: `T(1) / T(n)`
- Calculate efficiency: `Speedup(n) / n`
- Interpret target values (1.7x for 2-node, etc.)
- Verify speedup targets are met
- Analyze efficiency degradation curve
- Identify scaling bottlenecks
- Extract metrics from test logs
- Create speedup validation report
- Detect Byzantine fault tolerance
- Interpret Amdahl's law predictions

**Key analysis tools:**
- Speedup calculation formulas
- Efficiency curve plotting
- Bottleneck analysis
- Regression detection

---

### 3. Explanations (Conceptual Understanding)

**[Memory Optimization](explanation/memory-optimization.md)** (~20 min)

Understand:
- **6 optimization techniques:**
  1. String interning (15-25% reduction)
  2. Attribute deduplication via Arc (20-30% reduction)
  3. Adjacency list optimization (10-15% reduction)
  4. Conformance result streaming (30-40% reduction)
  5. Incremental statistics (20-25% reduction)
  6. Cache-line alignment (2-5% speedup)

- Memory landscape before optimization (800MB for 100M events)
- Top 5 memory consumers
- Implementation in `src/memory/allocator.rs`
- Correctness guarantees (all optimizations maintain semantics)
- Trade-offs for each technique

**Target:** 40-45% total reduction across all components

---

**[Cache-Aware Optimization](explanation/cache-aware-optimization.md)** (~25 min)

Understand:
- **CPU cache fundamentals** (L1/L2/L3/RAM latency costs)
- **Process mining cache problem** (poor locality in graph algorithms)
- **6 cache optimization patterns:**
  1. Adjacency caching (O(m) → O(1) edge lookups, 15-30% speedup)
  2. Cache-line alignment (2-5% speedup on hot paths)
  3. Single-pass aggregation (O(n log n) → O(n log k), 50-200% speedup)
  4. Early-termination BFS (average-case speedup)
  5. Memoization (50-500% speedup on repeated calculations)
  6. Parallel activity detection (O(n²) → O(n), 100x+ speedup)

- CPU cache basics with latency timeline
- Before/after code examples for each pattern
- Performance benchmarks
- Profiling with Linux perf

**Combined effect:** 30-45% overall speedup on discovery and conformance

---

### 4. Reference Documentation

**[Performance Guide Index](reference/performance-guide-index.md)** (~30 min)

Complete reference including:
- Full test organization (all test files and locations)
- Test matrix (12 concurrent, 10 distributed, 10 memory, 10 stress tests)
- Test specifications (file paths, line numbers, targets)
- Optimization implementation map (which optimization is in which file)
- Quick reference commands (copy-paste ready)
- Baseline measurement templates location
- Performance metrics to track
- Navigation map for self-discovery

**Additional reference files** (in `pm4py-rust/docs/`):

- `BASELINE_MEASUREMENT_TEMPLATES.md` — Capture & format templates
- `DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md` — Analysis & interpretation templates
- `PERFORMANCE_BENCHMARKING.md` — Benchmark methodology
- `PERFORMANCE_TUNING_GUIDE.md` — Parameter tuning
- `VARIANT_PERFORMANCE_ANALYSIS.md` — Variant-specific optimizations

---

## Test File Organization

### Concurrent Load Tests (`tests/load_testing.rs`)

12 tests validating system behavior under high concurrency:
- 3 discovery concurrency levels (10, 50, 100 simultaneous)
- 3 conformance concurrency levels (10, 50, 100 simultaneous)
- 1 statistics test (50 simultaneous)
- 1 mixed operations test (30 discovery + 30 conformance + 30 stats)
- 1 large log test (100k events)
- 1 resource contention test (90 total threads)
- 1 batch operations test (100 sequential)
- 1 memory stability test (1000 iterations)

**Speedup targets:** All < 30-60 seconds, success > 90%

---

### Distributed Speedup Tests (`tests/distributed_speedup_test.rs`)

10 tests validating multi-node parallelism:
- 1 single-node baseline (1M events)
- 4 speedup tests (2, 3, 5, 8 nodes with increasing event counts)
- 1 partition correctness test (all patterns)
- 1 merge completeness test (edge preservation)
- 1 Byzantine fault tolerance test (tolerate 2/5 failures)
- 1 scalability curve test (efficiency degradation analysis)
- 1 end-to-end distributed conformance test

**Speedup targets:**
- 2-node: ≥1.7x speedup, ≥85% efficiency
- 3-node: ≥2.5x speedup, ≥83% efficiency
- 5-node: ≥3.8x speedup, ≥76% efficiency
- 8-node: ≥5.5x speedup, ≥69% efficiency

---

### Memory Profiling Tests (`tests/memory_profiling_test.rs`)

10 tests measuring memory usage at different scales:
- 3 EventLog profiling (1M, 10M, 100M events)
- 2 PetriNet profiling (medium, high complexity)
- 2 Conformance tests (TokenReplay, DFG)
- 1 Statistics test (incremental aggregation)
- 1 memory consumer analysis (identify top 5)
- 1 correctness verification

**Memory targets:**
- 1M events: < 1200 MB
- 10M events: < 6000 MB
- 100M events: < 6000 MB

---

### Stress Scenario Tests (`tests/stress_scenarios.rs`)

10 tests validating edge cases and pathological logs:
- Rapid-fire logging (5000 events/sec burst)
- Memory pressure (large allocation stress)
- 5 pathological logs (deep, branching, loops, fragmented, complex)
- Cascading failures (simultaneous errors)
- Sustained load (1000 sequential operations)
- 24-hour simulation (100 concurrent threads)

**Success criteria:** No panics, graceful degradation, < 60 seconds per scenario

---

## Integration with Main Documentation

This diataxis documentation complements existing guides:

| Existing Doc | Purpose | Diataxis Equivalent |
|---|---|---|
| `PERFORMANCE.md` | Overview | Reference: memory/cache fundamentals |
| `PERFORMANCE_BENCHMARKING.md` | Methodology | How-to: baseline measurement |
| `PERFORMANCE_TUNING_GUIDE.md` | Parameter tuning | Explanation: optimization patterns |
| `OPTIMIZATION_ANALYSIS.md` | Deep analysis | Explanation: memory/cache optimization |

**Relationship:** Diátaxis docs provide **learning path** (tutorials → guides → explanations → reference), while existing docs provide **technical depth**.

---

## How to Use This Documentation

### For First-Time Users
1. Read [Load Testing Quick Start](tutorials/load-testing-quickstart.md) (15 min)
2. Run one load test to verify setup
3. Read [Baseline Measurement Guide](how-to/baseline-measurement.md) (20 min)
4. Capture your first baseline

### For Optimization Work
1. Read [Memory Optimization](explanation/memory-optimization.md) (20 min)
2. Read [Cache-Aware Optimization](explanation/cache-aware-optimization.md) (25 min)
3. Run memory profiling tests
4. Implement optimization in `src/`
5. Compare baseline using [Baseline Measurement Guide](how-to/baseline-measurement.md)

### For Speedup Analysis
1. Read [Speedup Validation Guide](how-to/speedup-validation.md) (20 min)
2. Run distributed speedup tests
3. Analyze efficiency curve using guide templates
4. Document findings in speedup report

### For Troubleshooting
1. Check [Load Testing Quick Start](tutorials/load-testing-quickstart.md) troubleshooting section
2. Consult [Performance Guide Index](reference/performance-guide-index.md) for test organization
3. Review test output against expected results

---

## Key Metrics Dashboard

Track these metrics across versions:

**Performance:**
- Discovery time (1M events): target 10s
- Conformance time (1M events): target 8s
- Memory usage (1M events): target 450 MB
- 2-node speedup: target ≥1.7x
- 8-node efficiency: target ≥69%

**Correctness:**
- Model size (places + transitions)
- Fitness scores (should remain stable)
- Edge preservation (100% on merge)
- Trace coverage (100%)

**Scalability:**
- Memory per event (should decrease with optimization)
- Time per event (should remain flat)
- Cache hit rate (should increase)
- Speedup efficiency (should follow Amdahl's law)

---

## Document Navigation

```
        START HERE
            ↓
    [Load Testing QuickStart]
            ↓
        ├─→ [Want to run tests?]
        │   └─→ Quick commands in reference/
        │
        ├─→ [Want to measure performance?]
        │   └─→ [Baseline Measurement Guide]
        │   └─→ [BASELINE_MEASUREMENT_TEMPLATES.md]
        │
        ├─→ [Want to analyze speedup?]
        │   └─→ [Speedup Validation Guide]
        │   └─→ [DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md]
        │
        ├─→ [Want to optimize memory?]
        │   └─→ [Memory Optimization Explanation]
        │   └─→ [src/memory/allocator.rs]
        │
        └─→ [Want to optimize for cache?]
            └─→ [Cache-Aware Optimization Explanation]
            └─→ [src/optimization/cache_aware.rs]
            └─→ [src/optimization/hotspot_elimination.rs]

    [Performance Guide Index]
            ↑
    (For detailed test matrix & commands)
```

---

## File Locations

**Diataxis documentation:**
- Tutorials: `docs/diataxis/tutorials/`
- How-to guides: `docs/diataxis/how-to/`
- Explanations: `docs/diataxis/explanation/`
- References: `docs/diataxis/reference/`

**Template & report files:**
- Baseline templates: `docs/BASELINE_MEASUREMENT_TEMPLATES.md`
- Speedup report: `docs/DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md`

**Implementation:**
- Memory allocator: `src/memory/allocator.rs`
- Cache optimization: `src/optimization/cache_aware.rs`
- Hotspot elimination: `src/optimization/hotspot_elimination.rs`

**Tests:**
- Load tests: `tests/load_testing.rs`
- Speedup tests: `tests/distributed_speedup_test.rs`
- Memory tests: `tests/memory_profiling_test.rs`
- Stress tests: `tests/stress_scenarios.rs`

---

## Contributing to This Documentation

To add or update documentation:

1. **Tutorials:** Add step-by-step guides for running new test categories
2. **How-to guides:** Add goal-oriented guides for new measurement/optimization tasks
3. **Explanations:** Explain the "why" behind architectural decisions
4. **References:** Update test matrix and quick-start commands

Follow the existing structure and update this INDEX when adding new docs.

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-24 | Initial diataxis documentation suite |

---

## Questions?

- **How do I run tests?** → [Load Testing Quick Start](tutorials/load-testing-quickstart.md)
- **How do I set up baselines?** → [Baseline Measurement Guide](how-to/baseline-measurement.md)
- **Why is my speedup < target?** → [Speedup Validation Guide](how-to/speedup-validation.md)
- **Where's the complete test list?** → [Performance Guide Index](reference/performance-guide-index.md)
- **How do I optimize memory?** → [Memory Optimization](explanation/memory-optimization.md)
- **How do I optimize for cache?** → [Cache-Aware Optimization](explanation/cache-aware-optimization.md)

---

**Last Updated:** 2026-03-24
