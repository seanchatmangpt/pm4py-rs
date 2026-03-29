# Benchmarking Quick Start Guide

## 30-Second Version

### Run All Benchmarks

```bash
cd pm4py-rust
./scripts/run_benchmarks.sh
```

Output appears in current directory:
- JSON results with timing data
- Performance comparison report
- Markdown documentation

## 5-Minute Version

### Prerequisites

```bash
# Rust (already installed if you're building)
rustc --version  # 1.70+

# Python (optional, for comparison)
python3 --version
pip install pm4py psutil pandas
```

### Quick Benchmark

```bash
# Just Rust benchmarks (fastest)
./scripts/run_benchmarks.sh --rust-only
# Output: rust_benchmarks_*.json + benchmark charts in target/criterion/

# Just Python benchmarks
./scripts/run_benchmarks.sh --python-only
# Output: python_benchmarks_*.json

# Full comparison
./scripts/run_benchmarks.sh
# Output: comparison report + performance_comparison_*.md
```

### View Results

```bash
# HTML charts (Rust)
open target/criterion/report/index.html

# JSON data
cat rust_benchmarks_*.json | jq .

# Comparison
cat PERFORMANCE_COMPARISON_*.md
```

## What Gets Tested

### Scale & Patterns

```
Event Counts:
  100K events  → Individual application
  1M events    → Enterprise
  10M events   → Large organization

Pattern Types:
  Linear       → Sequential processes (A→B→C)
  Parallel     → Concurrent activities
  Loops        → Rework/iterations
```

### Algorithms

```
Discovery:
  ✓ Alpha Miner
  ✓ Inductive Miner
  ✓ DFG Miner

Conformance:
  ✓ Token Replay

Statistics:
  ✓ Frequency
  ✓ Variants
  ✓ Rework patterns
```

## Expected Results

### Speedup (Rust vs Python)

```
Discovery:      2-5x faster
Conformance:    2-3x faster
Statistics:     3-5x faster
```

Lower numbers = more complex algorithm
Higher numbers = simpler/more parallelizable

## Common Tasks

### Run Only Discovery Benchmarks

```bash
cargo bench --bench discovery_bench
cargo bench --bench scale_benchmarks
```

### Run Only Conformance

```bash
cargo bench --bench comprehensive_conformance_bench
```

### Run Only Statistics

```bash
cargo bench --bench comprehensive_statistics_bench
```

### Custom Output Directory

```bash
./scripts/run_benchmarks.sh --output ./my_results
```

### Detailed Python Benchmarks

```bash
python3 scripts/python_benchmark.py \
  --output my_results.json \
  --discovery-only
  --warmup 2
```

## Interpreting Results

### Criterion Output (Rust)

```
discovery_dfg_100k_linear
  time:   [12.345 ms 12.456 ms 12.567 ms]
                    ↑ min        ↑ mean        ↑ max
```

Lower is better. Look for:
- Consistent results (small range min→max)
- No regressions vs previous runs
- Throughput increasing with optimization

### Comparison Report (JSON)

```json
{
  "discovery": {
    "alpha_100k": {
      "rust_time_ms": 123.45,
      "python_time_ms": 456.78,
      "speedup": 3.7
    }
  }
}
```

- `speedup < 1` = Python is faster (unusual)
- `speedup = 2-5` = Expected Rust advantage
- `speedup > 10` = Something might be wrong

### Markdown Report

See tables:
- **Speedup:** Should be 2-5x for most algorithms
- **Events:** Increasing should show linear scaling
- **Throughput:** Should increase proportionally

## Troubleshooting

### "command not found: ./scripts/run_benchmarks.sh"

```bash
chmod +x ./scripts/run_benchmarks.sh
./scripts/run_benchmarks.sh
```

### "pm4py not installed"

```bash
pip install pm4py psutil pandas
# Then re-run benchmarks
```

### Benchmarks running too slow

Option 1 - Use smaller scale:
```bash
# Modify benches/*.rs, change event sizes
# 100K → 10K for quick testing
```

Option 2 - Run specific benchmarks:
```bash
cargo bench --bench scale_benchmarks
# Takes ~2 minutes instead of 10
```

### Python benchmarks different every run

Normal! Python has:
- Garbage collection pauses
- JIT compilation overhead
- Thread scheduling variance

Expected variance: ±5-15%

### Huge memory usage during benchmarking

Normal for 10M event tests. Expected peaks:
- Rust: 2-4 GB
- Python: 4-8 GB

Use `--rust-only` if memory constrained.

## Next Steps

### For Development

- Review HTML charts in `target/criterion/report/`
- Compare against baseline
- Profile hot paths with `perf` or `flamegraph`

### For Release

- Run full benchmarks on clean system
- Archive results with version tag
- Include in release notes

### For Optimization

1. Identify slow benchmarks
2. Profile with `cargo flamegraph`
3. Optimize hot paths
4. Re-run benchmarks
5. Verify improvement

## References

- Full details: See `docs/PERFORMANCE_BENCHMARKING.md`
- Criterion guide: https://bheisler.github.io/criterion.rs/book/
- Rust perf book: https://nnethercote.github.io/perf-book/
