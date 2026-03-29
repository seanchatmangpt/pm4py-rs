# PM4Py Rust Benchmarking Guide

Quick reference for running benchmarks and performance tests.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run discovery benchmarks
cargo bench --bench discovery

# Run specific algorithm
cargo bench alpha_miner

# Run with custom sample size
cargo bench -- --sample-size 20

# Generate HTML reports
cargo bench -- --verbose
```

## Benchmark Suites

### 1. Discovery Algorithms (`benches/discovery.rs`)

Benchmarks for all major discovery algorithms on logs of 100, 1K, 10K, and 100K events.

```bash
cargo bench --bench discovery

# Specific algorithm
cargo bench --bench discovery alpha_miner
cargo bench --bench discovery inductive_miner
cargo bench --bench discovery heuristic_miner
cargo bench --bench discovery dfg_miner

# Comparative analysis
cargo bench --bench discovery discovery_comparison
```

**Key Metrics:**
- Execution time by event count
- Algorithm comparison at 10K events
- Scaling behavior

### 2. Conformance Checking (`benches/conformance.rs`)

Benchmarks for token replay, footprints, and other conformance algorithms.

```bash
cargo bench --bench conformance

# Token replay per trace
cargo bench --bench conformance token_replay_per_trace

# Comparison of approaches
cargo bench --bench conformance conformance_comparison
```

**Key Metrics:**
- Token replay fitness computation
- Per-trace replay performance
- Conformance algorithm comparison

### 3. I/O Operations (`benches/io.rs`)

Benchmarks for XES and CSV import/export operations.

```bash
cargo bench --bench io

# XES operations
cargo bench --bench io xes_export
cargo bench --bench io xes_import

# CSV operations
cargo bench --bench io csv_export
cargo bench --bench io csv_import

# Round-trip testing
cargo bench --bench io round_trip

# Format comparison
cargo bench --bench io format_comparison
```

**Key Metrics:**
- Export performance by event count
- Import performance by event count
- Format speed comparison

### 4. Statistical Analysis (`benches/analysis.rs`)

Benchmarks for log and trace statistics computation.

```bash
cargo bench --bench analysis

# Specific statistics
cargo bench --bench analysis log_statistics
cargo bench --bench analysis activity_statistics
cargo bench --bench analysis variant_analysis

# Comparison
cargo bench --bench analysis statistics_comparison
```

**Key Metrics:**
- Log statistics computation
- Activity frequency analysis
- Trace variant enumeration

## Performance Tests (SLA Verification)

Performance tests verify Service Level Agreements are met.

```bash
# Run all performance tests
cargo test --test performance -- --nocapture

# Run specific SLA test
cargo test --test performance test_alpha_miner_10k_events_sla -- --nocapture

# Run all ignored tests (detailed analysis)
cargo test --test performance -- --ignored --nocapture

# Comparative benchmark
cargo test --test performance test_comparative_performance -- --ignored --nocapture

# Scaling analysis
cargo test --test performance test_scaling_behavior -- --ignored --nocapture
```

**SLA Targets:**

```
✓ Alpha Miner:          1K < 50ms,    10K < 100ms
✓ Inductive Miner:      1K < 200ms,   10K < 500ms
✓ Heuristic Miner:      1K < 100ms,   10K < 150ms
✓ DFG Miner:            10K < 20ms,   100K < 100ms
✓ Token Replay:         10K < 100ms
✓ Log Statistics:       10K < 50ms
✓ Activity Statistics:  10K < 50ms
✓ Trace Variants:       10K < 100ms
```

## Profiling & Analysis

### Generate Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Profile discovery benchmark
cargo flamegraph --bench discovery -o discovery_profile.svg

# Profile specific algorithm
cargo flamegraph --bench discovery -- alpha_miner -o alpha_profile.svg
```

### Using Linux `perf`

```bash
# Record performance data
perf record -g ./target/release/deps/discovery-*

# Generate report
perf report

# Convert to flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

### Benchmark Results

HTML reports generated in `target/criterion/`:

```bash
# Open the report
open target/criterion/report/index.html
```

## Baseline Comparison

Compare against a baseline to detect regressions:

```bash
# Set baseline on main branch
cargo bench --bench discovery -- --baseline main

# Run on feature branch
cargo bench --bench discovery

# Compare and see which benchmarks changed
```

## Performance Guidelines

See `docs/PERFORMANCE.md` for:

- **Complete benchmark results** with hardware specs
- **Performance targets** for each algorithm
- **Scaling behavior** analysis
- **Algorithm characteristics** and when to use each
- **Optimization tips** for users and developers
- **Memory usage** estimates
- **Profiling guide** and hot paths

## Common Issues

### Benchmarks too slow?

1. Use `--sample-size 5` to reduce samples:
   ```bash
   cargo bench --bench discovery -- --sample-size 5
   ```

2. Skip slow algorithms:
   ```bash
   cargo bench --bench discovery -- alpha_miner dfg_miner
   ```

### Out of memory?

1. Reduce max event count in benchmark generators
2. Run one benchmark at a time
3. Increase system swap/memory

### Variance in results?

1. Increase sample size: `--sample-size 30`
2. Close other applications
3. Disable CPU frequency scaling:
   ```bash
   sudo cpupower frequency-set -g performance
   ```

## Continuous Integration

For CI/CD pipelines:

```bash
# Quick performance check (5 samples)
cargo bench --bench discovery -- --sample-size 5

# Full benchmark suite (with baselines)
cargo bench -- --baseline main --verbose
```

## Reporting Performance Results

When reporting performance improvements/regressions:

1. Include hardware specifications
2. Show before/after times
3. Include sample size and confidence intervals
4. Link to relevant benchmark runs
5. Include flamegraph for hot paths

Example:

```
Performance Improvement:

Algorithm: Alpha Miner (10K events)
Before:    47.2ms ± 2.1ms (n=10)
After:     38.5ms ± 1.8ms (n=10)
Change:    -18.4% improvement ✓

Flamegraph: [link to updated flamegraph]
Benchmark:  cargo bench discovery alpha_miner
```

---

**For detailed performance analysis, see `docs/PERFORMANCE.md`**
