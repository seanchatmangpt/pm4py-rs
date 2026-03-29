# How to: Capture and Use Baseline Measurements

## Overview

Baseline measurements establish reference performance metrics to detect regressions and validate optimizations.

## What is a Baseline?

A **baseline** is a set of performance metrics from a known, stable code version:

```
Single-node discovery (1M events):
  Time: 12.5 seconds
  Memory: 450 MB
  Model size: 1250 places, 980 transitions

2-node distributed (1M events):
  Time: 7.2 seconds
  Speedup: 1.74x
  Efficiency: 87%
```

## Step 1: Prepare Your Environment

Ensure consistent test conditions:

```bash
# Close unnecessary applications
killall chrome firefox Slack

# Check system load
top -b -n 1 | head -20

# Disable CPU frequency scaling (optional, for precision)
# On Linux: echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Use release builds
cargo build --release
```

## Step 2: Run Single-Node Baseline

Start with a single-node baseline for all operations:

```bash
# Discovery baseline (1M events)
cargo test --test load_testing test_large_log_single_thread --release -- --nocapture 2>&1 | tee baseline_discovery_1m.log

# Extract metrics from output:
# - Completed in: [TIME]
# - Places: [N], Transitions: [M]

# Conformance baseline
cargo test --test load_testing test_concurrent_conformance_10_simultaneous --release -- --nocapture 2>&1 | tee baseline_conformance_1m.log

# Statistics baseline
cargo test --test load_testing test_concurrent_statistics_50_simultaneous --release -- --nocapture 2>&1 | tee baseline_stats_1m.log
```

## Step 3: Capture Memory Baseline

Profile memory usage at key scales:

```bash
# 1M events (standard enterprise)
cargo test --test memory_profiling_test profile_eventlog_1m_events --release -- --nocapture 2>&1 | tee baseline_mem_1m.log

# 10M events (high-volume)
cargo test --test memory_profiling_test profile_eventlog_10m_events --release -- --nocapture 2>&1 | tee baseline_mem_10m.log

# Extract metrics:
# - Estimated memory: [MB]
# - Creation time: [SECONDS]
```

## Step 4: Run Distributed Speedup Baseline

Establish speedup expectations for different node counts:

```bash
# Run all speedup tests and capture in a report
cargo test --test distributed_speedup_test --release -- --nocapture 2>&1 | tee baseline_speedup.log

# Expected output includes:
# - Single-node baseline time
# - 2-node speedup: X.XXx (target: ≥1.7x)
# - 3-node speedup: X.XXx (target: ≥2.5x)
# - 5-node speedup: X.XXx (target: ≥3.8x)
# - 8-node speedup: X.XXx (target: ≥5.5x)
```

## Step 5: Create Baseline Report

Use this template to document your baseline:

```markdown
# Baseline Measurement Report

**Date:** 2026-03-24
**System:** [CPU model], [RAM GB], [OS]
**Rust Version:** [rustc --version]
**pm4py-rust Commit:** [git rev-parse --short HEAD]

## Single-Node Discovery (1M events)

| Metric | Value |
|--------|-------|
| Time | 12.5s |
| Memory | 450 MB |
| Places | 1250 |
| Transitions | 980 |

## Memory Profiling

| Scale | Memory | Creation Time |
|-------|--------|---------------|
| 1M events | 450 MB | 2.34s |
| 10M events | 4.2 GB | 23.4s |
| 100M events | 41 GB | 234s |

## Distributed Speedup

| Nodes | Time (s) | Speedup | Efficiency |
|-------|----------|---------|-----------|
| 1 | 12.5 | — | — |
| 2 | 7.2 | 1.74x | 87% |
| 3 | 5.1 | 2.45x | 82% |
| 5 | 3.4 | 3.68x | 74% |
| 8 | 2.3 | 5.43x | 68% |

## Conformance Checking (1M events)

| Test | Time | Fitness | Success Rate |
|------|------|---------|--------------|
| TokenReplay | 8.2s | 0.85 | 100% |

## Notes

- All tests run in `--release` mode
- System was idle during testing
- Cold cache (first run, no repetition)
```

Save as: `docs/BASELINE_MEASUREMENT_TEMPLATES.md`

## Step 6: Store Baseline for Comparison

Create a version-specific baseline directory:

```bash
mkdir -p .benchmarks/baselines/

# Create baseline file
cat > .benchmarks/baselines/v$(git rev-parse --short HEAD).json << 'EOF'
{
  "commit": "$(git rev-parse HEAD)",
  "date": "$(date -Iseconds)",
  "discovery_1m_time_ms": 12500,
  "discovery_1m_memory_mb": 450,
  "conformance_1m_time_ms": 8200,
  "memory_1m_mb": 450,
  "memory_10m_mb": 4200,
  "speedup_2node": 1.74,
  "speedup_3node": 2.45,
  "speedup_5node": 3.68,
  "speedup_8node": 5.43
}
EOF
```

## Step 7: Compare Against Baseline

After code changes, run tests and compare:

```bash
# Run tests with new code
cargo test --test load_testing test_large_log_single_thread --release -- --nocapture 2>&1 | tee current_discovery.log

# Extract time: grep "Completed in:"
BASELINE_TIME=12.5
CURRENT_TIME=$(grep "Completed in:" current_discovery.log | awk '{print $3}' | sed 's/s//')

# Calculate regression
REGRESSION=$(echo "scale=2; ($CURRENT_TIME - $BASELINE_TIME) / $BASELINE_TIME * 100" | bc)
echo "Performance change: ${REGRESSION}%"
```

## Interpreting Results

### Green Light (≤ 5% regression)
```
Baseline: 12.5s
Current:  13.0s
Δ:        +4.0%  ✓ ACCEPTABLE
```
No action needed; regression within noise margin.

### Yellow Light (5-10% regression)
```
Baseline: 12.5s
Current:  13.8s
Δ:        +10.4%  ⚠ INVESTIGATE
```
Check for:
- System load during test
- Unintended code changes
- New debug logging

### Red Light (> 10% regression)
```
Baseline: 12.5s
Current:  14.5s
Δ:        +16.0%  ✗ FAILURE
```
Action required:
1. Profile with flamegraph: `cargo flamegraph --test load_testing`
2. Revert recent commits
3. Optimize identified hotspots

## Benchmark Script Example

Create `scripts/benchmark.sh`:

```bash
#!/bin/bash
set -e

BASELINE_FILE=".benchmarks/baseline.json"
RESULTS_FILE=".benchmarks/results_$(date +%s).json"

echo "Running baseline measurements..."

# Extract all metrics
DISCOVERY_TIME=$(cargo test --test load_testing test_large_log_single_thread --release -- --nocapture 2>&1 | grep "Completed in:" | awk '{print $3}' | sed 's/s//')
MEMORY_1M=$(cargo test --test memory_profiling_test profile_eventlog_1m_events --release -- --nocapture 2>&1 | grep "Estimated memory:" | awk '{print $3}' | sed 's/MB//')

# Store results
cat > "$RESULTS_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "discovery_1m_time_s": $DISCOVERY_TIME,
  "memory_1m_mb": $MEMORY_1M
}
EOF

echo "✓ Baseline saved to $RESULTS_FILE"

# Compare against previous baseline if it exists
if [ -f "$BASELINE_FILE" ]; then
  BASELINE_DISCOVERY=$(jq '.discovery_1m_time_s' "$BASELINE_FILE")
  REGRESSION=$(echo "scale=2; ($DISCOVERY_TIME - $BASELINE_DISCOVERY) / $BASELINE_DISCOVERY * 100" | bc)
  echo "Performance change: ${REGRESSION}%"

  if (( $(echo "$REGRESSION > 10" | bc -l) )); then
    echo "⚠ WARNING: > 10% regression detected!"
    exit 1
  fi
fi

# Update baseline
cp "$RESULTS_FILE" "$BASELINE_FILE"
```

Run with: `bash scripts/benchmark.sh`

## Related Documentation

- **Speedup Validation**: `docs/diataxis/how-to/speedup-validation.md`
- **Memory Optimization**: `docs/diataxis/explanation/memory-optimization.md`
- **Performance Tuning**: `pm4py-rust/docs/PERFORMANCE_TUNING_GUIDE.md`
