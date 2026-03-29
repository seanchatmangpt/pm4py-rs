# PM4Py-Rust Benchmarking and Scale Testing Index

## Overview

Complete benchmarking infrastructure for performance validation at enterprise scales (100K to 100M events).

## 🚀 Quick Links

| Document | Purpose | Audience |
|----------|---------|----------|
| [SCALE_BENCHMARKING_README.md](../SCALE_BENCHMARKING_README.md) | Quick start guide | Everyone |
| [SCALE_BENCHMARKING_GUIDE.md](./SCALE_BENCHMARKING_GUIDE.md) | Complete reference | Developers, DevOps |
| [PERFORMANCE_RESULTS_TEMPLATE.md](./PERFORMANCE_RESULTS_TEMPLATE.md) | Results reporting | QA, Performance analysts |

## 📁 File Structure

```
pm4py-rust/
├── SCALE_BENCHMARKING_README.md          Main guide (this project)
├── tests/
│   └── scale_benchmarks_test.rs          15 scale tests (419 lines)
├── benches/
│   ├── scale_benchmarks.rs               13 Criterion benchmarks (267 lines)
│   ├── discovery.rs                      Existing discovery benchmarks
│   └── ...
├── docs/
│   ├── SCALE_BENCHMARKING_GUIDE.md       Complete reference (340 lines)
│   ├── PERFORMANCE_RESULTS_TEMPLATE.md   Results template (200 lines)
│   └── BENCHMARKING_INDEX.md             This file
└── Cargo.toml                            Benchmark registration
```

## 📊 Test Coverage

### 15 Scale Tests Across 4 Categories

#### 1. Baseline Tests (100K events) - ~1 minute
- Alpha Miner discovery (<1s)
- DFG Miner discovery (<500ms)
- Token Replay conformance (<2s)

#### 2. Enterprise Tests (1M events) - ~10 minutes
- Alpha Miner
- Inductive Miner
- DFG Miner
- Token Replay conformance
- Scalability validation

#### 3. Large Organization Tests (10M events) - ~30+ minutes
- Alpha Miner (<30s target)
- DFG Miner (<15s target)
- Scalability from 1M

#### 4. Stress Tests
- High activity diversity (20 activities)
- Many traces (50K traces)
- Long traces (10 events each)

### 13 Criterion Benchmarks

**Discovery Algorithms:**
- Alpha Miner: 100K, 1M, 10M
- Inductive Miner: 100K, 1M
- DFG Miner: 100K, 1M, 10M

**Conformance:**
- Token Replay: 100K, 1M

**Analysis:**
- Scalability: Multi-scale comparisons
- Throughput: Events/second metrics

## ⚡ Quick Commands

### Run Baseline Tests Only (1 minute)
```bash
cargo test --test scale_benchmarks_test test_discovery_alpha_100k_baseline --release -- --nocapture
```

### Run All Scale Tests (30+ minutes)
```bash
cargo test --test scale_benchmarks_test --release -- --nocapture
```

### Run Enterprise Tests (10 minutes)
```bash
cargo test --test scale_benchmarks_test test_discovery_alpha_1m_enterprise --release -- --nocapture
```

### Run Benchmarks (1-2 hours)
```bash
cargo bench --bench scale_benchmarks
```

### View Benchmark Reports
```bash
open target/criterion/report/index.html
```

## 📈 Performance Targets

| Size | Algorithm | Target | Status |
|------|-----------|--------|--------|
| 100K | Baseline (all) | <1s | ✓ |
| 1M | Alpha | <5s | ✓ |
| 1M | DFG | <3s | ✓ |
| 1M | Token Replay | <5s | ✓ |
| 10M | Alpha | <30s | ? |
| 10M | DFG | <15s | ? |

## 🔍 Key Metrics Collected

### Timing
- CPU seconds per algorithm per size
- Wall-clock time from Criterion
- Throughput (events/second)

### Accuracy
- Fitness (token replay conformance)
- Model quality (places, transitions discovered)
- Pattern recognition (rework, loops)

### Scalability
- Linear vs quadratic growth
- Scaling ratio (1M/100K)
- Memory efficiency

### Stress
- High-activity logs (20+ unique activities)
- Many traces (50K+)
- Long traces (1000+ events per trace)

## 📚 Documentation Structure

### For Different Audiences

**New Users:**
1. Start with [SCALE_BENCHMARKING_README.md](../SCALE_BENCHMARKING_README.md)
2. Run quick baseline: `cargo test test_discovery_alpha_100k_baseline --release`
3. Check quick commands section

**Developers:**
1. Read [SCALE_BENCHMARKING_GUIDE.md](./SCALE_BENCHMARKING_GUIDE.md) completely
2. Study test file: `tests/scale_benchmarks_test.rs`
3. Study benchmark file: `benches/scale_benchmarks.rs`
4. Understand test data generation

**Performance Analysts:**
1. Use [PERFORMANCE_RESULTS_TEMPLATE.md](./PERFORMANCE_RESULTS_TEMPLATE.md)
2. Collect metrics from benchmark runs
3. Analyze scalability growth
4. Identify bottlenecks

**DevOps/CI:**
1. Integrate tests into CI pipeline
2. Set baseline comparisons in Criterion
3. Configure regression detection
4. Set up artifact upload

## 🏗️ Implementation Details

### Test Data Generation

Two realistic log generators:

**Synthetic Log (standard process)**
```rust
generate_synthetic_log(num_events, num_traces, num_activities)
  → Rotating activities: A→B→C→D→E
  → Evenly distributed traces
  → 1-second event intervals
```

**Complex Log (with branches/rework)**
```rust
generate_complex_log(num_events, num_traces)
  → Start→Initialize→[Path_C|Path_D]→Process→[Alt_F|Alt_G]→Complete
  → 50% path variation
  → 33% rework loops
  → 5-second event intervals
```

### Criterion.rs Configuration

- **Measurement time**: 10 seconds per benchmark
- **Sample size**: 10 iterations
- **Output**: HTML reports with statistical analysis
- **Confidence**: 95%

### Black Box Optimization

All benchmarks use `black_box()` to prevent compiler optimizations that would skew results.

## 🎯 Success Criteria

All items must pass:

- [ ] 15/15 scale tests pass
- [ ] Baseline tests <1s
- [ ] Enterprise tests <5s
- [ ] 10M tests <30s
- [ ] Scalability ratio <20x (100K→1M)
- [ ] Fitness >0.7 on synthetic data
- [ ] Fitness >0.5 on complex patterns
- [ ] HTML benchmark reports generated
- [ ] No compile warnings in test files

## 🐛 Troubleshooting

### Tests Time Out
**Problem**: 10M tests exceed timeout
**Solution**:
- Reduce size: test 1M instead
- Use `--release` profile
- Pin CPU with `taskset`

### Out of Memory
**Problem**: OOM on 10M events
**Solution**:
- Reduce to 1M events
- Check for memory leaks with `valgrind`
- Implement streaming mode

### Fitness Too Low
**Problem**: Conformance fitness <0.5 on synthetic
**Solution**:
- Verify test data generation
- Compare with Python pm4py
- Check discovery algorithm

### High Variation in Benchmarks
**Problem**: Criterion reports high variance
**Solution**:
- Run on quiet system
- Disable CPU turbo boost
- Increase sample size

## 📊 Benchmark Results Format

After running benchmarks, results appear in:
```
target/criterion/
├── report/
│   └── index.html          Main dashboard
├── alpha_miner_100k/       Individual results
├── alpha_miner_1m/
└── ...
```

Each report shows:
- Mean time ± margin of error
- Throughput (events/sec)
- Historical trend (if baseline exists)
- Regression detection

## 🔄 Continuous Integration

### Adding to GitHub Actions

```yaml
- name: Run scale tests
  run: cargo test --test scale_benchmarks_test --release

- name: Run benchmarks
  run: cargo bench --bench scale_benchmarks

- name: Save baseline
  if: github.ref == 'refs/heads/main'
  run: cargo bench --bench scale_benchmarks -- --save-baseline main

- name: Compare benchmarks
  if: github.ref != 'refs/heads/main'
  run: cargo bench --bench scale_benchmarks -- --baseline main
```

### Regression Detection

Criterion automatically detects regressions (>5% change) and reports them.

## 📖 Related Files

- `src/discovery/` - Algorithm implementations
- `src/conformance/` - Conformance checking
- `tests/extended_discovery_integration_tests.rs` - Real-world tests
- `benches/discovery.rs` - Existing benchmarks
- `benches/conformance.rs` - Existing conformance benchmarks

## 🎓 Learning Resources

**Criterion.rs:**
- [Official Book](https://bheisler.github.io/criterion.rs/book/)
- [GitHub](https://github.com/bheisler/criterion.rs)

**PM4Py Process Mining:**
- [pm4py Python](https://pm4py.fit.fraunhofer.de/)
- [Process Mining Overview](https://en.wikipedia.org/wiki/Process_mining)

**Performance Analysis:**
- Big-O notation for algorithm complexity
- Memory profiling with valgrind
- CPU profiling with perf (Linux) or Instruments (macOS)

## 📞 Support

For issues or questions:
1. Check [SCALE_BENCHMARKING_GUIDE.md](./SCALE_BENCHMARKING_GUIDE.md) troubleshooting section
2. Review test logs for specific errors
3. Open GitHub issue: https://github.com/seanchatmangpt/pm4py-rust/issues
4. Email: info@chatmangpt.com

## 📝 Changelog

### Version 1.0 (2026-03-24)

**Created:**
- `tests/scale_benchmarks_test.rs` - 15 comprehensive scale tests
- `benches/scale_benchmarks.rs` - 13 Criterion benchmarks
- `docs/SCALE_BENCHMARKING_GUIDE.md` - Complete reference
- `docs/PERFORMANCE_RESULTS_TEMPLATE.md` - Results reporting template
- `SCALE_BENCHMARKING_README.md` - Quick start guide
- `docs/BENCHMARKING_INDEX.md` - This file

**Features:**
- 100K, 1M, 10M, and stress test coverage
- Criterion.rs statistical analysis
- Black box optimization prevention
- Realistic synthetic log generation
- Scalability analysis
- Fitness validation
- Multiple algorithm support

## 📋 Checklist for New Developers

- [ ] Read `SCALE_BENCHMARKING_README.md`
- [ ] Run baseline test: `cargo test test_discovery_alpha_100k_baseline --release`
- [ ] Understand test data generation (lines 10-100 in test file)
- [ ] Run enterprise test: `cargo test test_discovery_alpha_1m_enterprise --release`
- [ ] Read `SCALE_BENCHMARKING_GUIDE.md` sections 1-3
- [ ] Run quick benchmark: `cargo bench alpha_miner_100k`
- [ ] View HTML report: `open target/criterion/report/index.html`
- [ ] Study bottleneck identification section
- [ ] Set up CI integration if needed

---

**Last Updated**: 2026-03-24
**Maintainer**: Sean Chatman (info@chatmangpt.com)
