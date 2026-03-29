# Performance Baseline Measurement Templates

**Reference Implementation for Benchmark Execution & Data Capture**

---

## Expected Output Format

When running the test suite, expect output in this format:

### Test Suite Output Template

```
running 24 tests

[100K Alpha Miner] Time: XXXms, Memory: 25.60 MB, Throughput: 330000 events/sec
[100K DFG Miner] Time: XXms, Memory: 25.60 MB, Throughput: 3333333 events/sec
[100K Token Replay] Time: XXXms, Memory: 25.60 MB, Throughput: 250000 events/sec

[1M Alpha Miner] Time: Xs, Memory: 256.00 MB, Throughput: 330000 events/sec
[1M Inductive Miner] Time: Xs, Memory: 256.00 MB, Throughput: 200000 events/sec
[1M DFG Miner] Time: Xs, Memory: 256.00 MB, Throughput: 2000000 events/sec
[1M Token Replay] Time: Xs, Memory: 256.00 MB, Throughput: 300000 events/sec

[10M Alpha Miner] Time: XXs, Memory: 2560.00 MB, Throughput: 330000 events/sec
[10M Inductive Miner] Time: XXs, Memory: 2560.00 MB, Throughput: 200000 events/sec
[10M DFG Miner] Time: XXs, Memory: 2560.00 MB, Throughput: 2000000 events/sec
[10M Token Replay] Time: XXs, Memory: 2560.00 MB, Throughput: 300000 events/sec

Alpha Miner Scalability:
  100K: XXXms (25.60 MB)
  1M:   Xs (256.00 MB)
  10M:  XXs (2560.00 MB)
  Scale factor: 100.0x events → X.XXx time (ideal linear: 100.0x)

DFG Miner Scalability:
  100K:  XXms (25.60 MB)
  1M:    Xs (256.00 MB)
  10M:   XXs (2560.00 MB)
  Scale factor: 100.0x events → X.XXx time (ideal linear: 100.0x)
  Scale factor: 100.0x events → X.XXx time (ideal linear: 100.0x)

Token Replay Scalability:
  100K: XXXms (25.60 MB)
  1M:   Xs (256.00 MB)
  10M:  XXs (2560.00 MB)
  Scale factor: 100.0x events → X.XXx time (ideal linear: 100.0x)

Accuracy Test (Synthetic): Fitness=0.7854, Expected >0.7
Accuracy Test (Complex): Fitness=0.5324, Expected >0.5
Fitness Preservation Test:
  100K fitness: 0.7854
  1M fitness:   0.7743
  Difference: 0.0111 (target <0.15)

[Stress: 500K, 20 activities] Time: XXXms, Memory: 128.00 MB, Throughput: 2000000 events/sec
[Stress: 100K across 50K traces] Time: XXXms, Memory: 25.60 MB, Throughput: 2000000 events/sec
[Stress: 100K across 10 long traces] Time: XXXms, Memory: 25.60 MB, Throughput: 2000000 events/sec

[Stress: 1M events, variant distributions]
  Balanced (10K traces, 5 activities): XXXms
  Diverse  (10K traces, 15 activities): XXXms
  Few Long (100 traces, 5 activities): XXXms

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured
```

---

## Baseline Numeric Estimates

### Memory Calculations
```
100K events:
  Events memory:  100,000 * 256 bytes = 25,600 KB = 25.0 MB
  Traces memory:  2,000 * 128 bytes = 256 KB
  Total:          ~25.6 MB

1M events:
  Events memory:  1,000,000 * 256 bytes = 256,000 KB = 250 MB
  Traces memory:  10,000 * 128 bytes = 1.3 MB
  Total:          ~251-256 MB

10M events:
  Events memory:  10,000,000 * 256 bytes = 2,560 MB = 2.5 GB
  Traces memory:  20,000 * 128 bytes = 2.6 MB
  Total:          ~2,560 MB (2.5 GB)

100M events:
  Events memory:  100,000,000 * 256 bytes = 25,600 MB = 25 GB
  Traces memory:  100,000 * 128 bytes = 12.8 MB
  Total:          ~25,600 MB (25 GB)
```

### Throughput Estimates
```
DFG Miner (fastest):
  100K in 30ms   → 3.3M events/sec
  1M in 300ms    → 3.3M events/sec
  10M in 3s      → 3.3M events/sec

Alpha Miner (moderate):
  100K in 300ms  → 330K events/sec
  1M in 3s       → 330K events/sec
  10M in 30s     → 330K events/sec

Token Replay (slowest):
  100K in 400ms  → 250K events/sec
  1M in 4s       → 250K events/sec
  10M in 40s     → 250K events/sec
```

### Scalability Ratios
```
Expected for Linear O(n) algorithm:
  1M / 100K = 10x events → 10x time = 10.0x ratio ✓

Expected for O(n log n) algorithm:
  1M / 100K = 10x events → 33.2x time = 33.2x ratio (worse than linear)
  But typically 15-20x due to constant factors

Actual observations (if linear):
  100K→1M:   ratio should be 9-11x (targeting 10)
  1M→10M:    ratio should be 9-11x (targeting 10)

Non-linear warning:
  If ratio > 15x → potential O(n log n) or worse
  If ratio > 30x → likely O(n²) algorithm issue
```

---

## Criterion Benchmark Report Structure

After running `cargo bench --bench scale_benchmarks`, reports appear in:

```
target/criterion/
├── alpha_miner_100k/report/
│   ├── index.html              (individual benchmark results)
│   ├── base/raw.json           (raw timing data)
│   └── pdf/ and plots/         (graphs)
├── alpha_miner_1m/report/
├── alpha_miner_10m/report/
├── alpha_miner_100m/report/
├── inductive_miner_100k/report/
├── inductive_miner_1m/report/
├── inductive_miner_10m/report/
├── dfg_miner_100k/report/
├── dfg_miner_1m/report/
├── dfg_miner_10m/report/
├── dfg_miner_100m/report/
├── token_replay_100k/report/
├── token_replay_1m/report/
├── token_replay_10m/report/
├── token_replay_100m/report/
├── scalability_alpha/report/
├── scalability_dfg/report/
├── throughput_discovery/report/
├── throughput_conformance/report/
└── report/index.html           (master index)
```

Each report contains:
- **Time measurements** (mean, std dev)
- **Throughput** (events/sec)
- **PDF plots** showing time trends
- **Regression analysis** (compared to previous runs)
- **Raw JSON data** for automation

### Sample HTML Report Structure
```html
<!DOCTYPE html>
<html>
<head>
    <title>alpha_miner_1m - Criterion.rs Results</title>
</head>
<body>
    <h1>Benchmarking: alpha_miner_1m</h1>

    <table>
        <tr><td>Time (mean ± std):</td><td>3.45 s ± 0.12 s</td></tr>
        <tr><td>Change (vs baseline):</td><td>+2.1% [slower]</td></tr>
        <tr><td>Throughput:</td><td>289,855 events/sec</td></tr>
    </table>

    <img src="raw.svg" />  <!-- Time trend graph -->
</body>
</html>
```

---

## Automated Data Extraction

### Parse Test Output to CSV
```bash
#!/bin/bash
# Extract baseline numbers from test output

cargo test --test scale_benchmarks_test -- --nocapture 2>&1 | \
grep -E "^\[.*\] Time:" | \
sed 's/\[\(.*\)\] Time: \(.*\), Memory: \(.*\) MB, Throughput: \(.*\) events\/sec/\1,\2,\3,\4/' > baseline.csv

# baseline.csv output:
# 100K Alpha Miner,XXms,25.60,330000
# 100K DFG Miner,XXms,25.60,3333333
# ...
```

### Parse Criterion Reports to JSON
```bash
#!/bin/bash
# Extract Criterion raw data

for bench in target/criterion/*/base/raw.json; do
    scale=$(dirname $(dirname $bench) | xargs basename)
    cat $bench | jq '.measurements | map(.estimate) | add / length' | \
    xargs echo "$scale: mean ="
done
```

---

## Regression Detection Workflow

### Baseline Establishment
```bash
# Run initial baseline
cargo test --test scale_benchmarks_test -- --nocapture 2>&1 | tee baseline_run_1.log

# Verify all tests pass
grep "test result: ok" baseline_run_1.log
```

### Optimization Verification
```bash
# After optimization
cargo test --test scale_benchmarks_test -- --nocapture 2>&1 | tee optimized_run_1.log

# Extract timing comparisons
for metric in "Alpha Miner" "DFG Miner" "Token Replay"; do
    baseline=$(grep "\[$metric\]" baseline_run_1.log | grep "Time:" | head -1)
    optimized=$(grep "\[$metric\]" optimized_run_1.log | grep "Time:" | head -1)
    echo "Metric: $metric"
    echo "  Before: $baseline"
    echo "  After:  $optimized"
done
```

### Regression Alerting
```bash
# If any test takes >110% of baseline, alert

cargo test --test scale_benchmarks_test 2>&1 | \
while read line; do
    if [[ $line =~ Time:\ ([0-9.]+)([ms]+) ]]; then
        time="${BASH_REMATCH[1]}"
        unit="${BASH_REMATCH[2]}"
        if (( $(echo "$time > $BASELINE_TIME * 1.1" | bc -l) )); then
            echo "REGRESSION: $line"
        fi
    fi
done
```

---

## Success Checklist

### Pre-Benchmark
- [ ] System idle (close other apps)
- [ ] No background jobs (docker, CI, etc.)
- [ ] Thermal throttling disabled (if possible)
- [ ] Power management set to "performance"
- [ ] Sufficient disk space (100M tests need 30GB free)

### Baseline Run
- [ ] All 24 tests pass
- [ ] No timeouts on 1M/10M tests
- [ ] Memory estimates computed and printed
- [ ] Scalability ratios documented
- [ ] Fitness preservation verified (< 0.15 difference)
- [ ] Log output captured for reference

### Criterion Benchmark
- [ ] 20 benchmark functions complete
- [ ] HTML reports generated in `target/criterion/`
- [ ] No regression warnings on first run
- [ ] Throughput graphs show linear trend
- [ ] Statistical significance confirmed

### Documentation
- [ ] Baseline numbers recorded in spreadsheet
- [ ] Performance targets verified
- [ ] Optimization roadmap identified
- [ ] Regression detection workflow tested
- [ ] CI/CD integration configured

---

## Time Estimates

| Test | Type | Typical Duration | Notes |
|------|------|------------------|-------|
| 100K tests | Unit | <100ms each | Baseline reference |
| 1M tests | Unit | 3-5s each | Standard enterprise |
| 10M tests | Unit | 15-40s each | Large organization |
| 100M tests | Unit | 60-300s each | Marked @ignore, optional |
| Full test suite | Unit | 3-5 minutes | All 24 tests, no 100M |
| Full criterion | Bench | 15-30 minutes | All 20 benchmark functions |
| CI/CD pipeline | Full | 20-40 minutes | Tests + criterion + reports |

---

## Troubleshooting

### Test Times Vary Wildly
**Cause:** System load, CPU throttling
**Solution:** Run with `--test-threads=1` to serialize tests

### Memory Estimates Seem Low
**Cause:** 256 bytes/event is conservative estimate
**Reality:** Actual Rust structs may use 200-350 bytes
**Action:** Profile with `valgrind --tool=massif` for exact numbers

### Criterion Reports Overwrite Each Other
**Cause:** Multiple runs in same session
**Solution:** Move `target/criterion/` before each run or use timestamps:
```bash
mv target/criterion/ baseline_run_$(date +%s)
```

### 100M Tests Cause OOM
**Cause:** 25GB memory requirement
**Solution:**
  1. Reduce to 50M in test code
  2. Use cloud instances with 32GB+ RAM
  3. Stream-process logs (future optimization)

---

## Performance Monitoring During Runs

### Real-time System Metrics
```bash
# In separate terminal
watch -n 1 'top -l 1 | head -20'

# Or use Activity Monitor on macOS
open -a "Activity Monitor"
```

### Memory Profiling
```bash
# Detailed memory usage during test
/usr/bin/time -lv cargo test --test scale_benchmarks_test test_discovery_alpha_1m_enterprise

# Output includes:
# Maximum resident set size (kbytes): 2560000 (2.5 GB for 1M test)
```

### Flamegraph (CPU profiling)
```bash
# Install flamegraph
cargo install flamegraph

# Profile a single test
cargo flamegraph --test scale_benchmarks_test --bench -- test_discovery_dfg_1m_enterprise

# View result
open flamegraph.svg
```

---

## Reference Spreadsheet Template

```
Scale       | Algorithm      | Time    | Memory | Throughput | Fitness | Status
------------|----------------|---------|--------|------------|---------|--------
100K events | Alpha Miner    | XXXms   | 25 MB  | 330K/sec   | 0.78    | PASS
100K events | DFG Miner      | XXms    | 25 MB  | 3.3M/sec   | N/A     | PASS
100K events | Token Replay   | XXXms   | 25 MB  | 250K/sec   | 0.78    | PASS
1M events   | Alpha Miner    | Xs      | 250 MB | 330K/sec   | 0.77    | PASS
1M events   | DFG Miner      | Xs      | 250 MB | 3.3M/sec   | N/A     | PASS
1M events   | Token Replay   | Xs      | 250 MB | 250K/sec   | 0.77    | PASS
10M events  | Alpha Miner    | XXs     | 2.5GB  | 330K/sec   | 0.76    | PASS
10M events  | DFG Miner      | XXs     | 2.5GB  | 3.3M/sec   | N/A     | PASS
10M events  | Token Replay   | XXs     | 2.5GB  | 250K/sec   | 0.75    | PASS
100M events | Alpha Miner    | XXXs    | 25 GB  | 330K/sec   | 0.75    | SKIP
100M events | DFG Miner      | XXXs    | 25 GB  | 3.3M/sec   | N/A     | SKIP
100M events | Token Replay   | XXXs    | 25 GB  | 250K/sec   | 0.74    | SKIP
```

---

## Conclusion

This measurement template provides:
1. **Expected output format** for direct comparison
2. **Numeric estimates** for quick validation
3. **Data extraction scripts** for automation
4. **Regression detection** workflows
5. **Troubleshooting guide** for common issues

Use in conjunction with `/benches/scale_benchmarks.rs` and `/tests/scale_benchmarks_test.rs` to establish definitive baselines.
