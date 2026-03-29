# Baseline Measurement Templates

Reference templates for capturing and documenting baseline performance metrics.

## 1. System Information Template

```markdown
# Baseline Report: [VERSION/COMMIT]

**Baseline ID:** baseline_[YYYYMMDD]_[COMMIT_SHORT]
**Date:** [YYYY-MM-DD HH:MM:SS UTC]
**Completed By:** [Name/CI Pipeline]

## System Configuration

| Parameter | Value |
|-----------|-------|
| **Hostname** | [hostname] |
| **CPU Model** | [Intel Core i9-13900KS / etc] |
| **CPU Cores** | [e.g., 8 P-cores + 12 E-cores = 20 cores] |
| **CPU Frequency** | [3.0 GHz base / 5.8 GHz boost] |
| **RAM** | [e.g., 128 GB DDR5] |
| **L1 Cache** | [32 KB per core] |
| **L2 Cache** | [256 KB per core] |
| **L3 Cache** | [36 MB shared] |
| **OS** | [Ubuntu 22.04.3 / macOS 13.6 / etc] |
| **Kernel** | [6.2.0-26-generic / etc] |
| **Rust Toolchain** | [rustc 1.74.0] |
| **pm4py-rust Commit** | [git rev-parse HEAD] |

## Environment Notes

- [ ] CPU frequency scaling disabled (performance governor)
- [ ] Background processes minimized (only essential services)
- [ ] Network isolation (no active downloads/transfers)
- [ ] Temperature monitoring enabled (all cores < 65°C)
- [ ] Cache cleared before tests (L1/L2/L3 cold start)

```

## 2. Single-Node Baseline Template

```markdown
## Single-Node Discovery Baseline (1M Events)

**Test Command:**
```bash
cargo test --test load_testing test_large_log_single_thread --release -- --nocapture
```

**Configuration:**
- Log size: 1,000,000 events
- Traces: 10,000
- Events per trace: 100
- Activities: 7 (start, verify, process, check, approve, execute, complete)
- Pattern: Linear sequence with 20% rework loops

**Results:**

| Metric | Value | Unit | Notes |
|--------|-------|------|-------|
| Discovery Time | 12.5 | seconds | Wall-clock time |
| Places Discovered | 1,250 | count | Model size |
| Transitions Discovered | 980 | count | Model complexity |
| Memory Peak | 450 | MB | Max resident set |
| Memory Avg | 350 | MB | Average during execution |
| Cache Misses | 8.5% | percent | L3 miss rate (via perf) |

**Model Quality:**

| Metric | Value |
|--------|-------|
| Traces correctly represented | 10,000/10,000 |
| DFG edges discovered | 2,100 |
| Parallelism detected | 45 pairs |
| Loops correctly handled | Yes |

```

## 3. Memory Profiling Baseline Template

```markdown
## Memory Profiling Baseline

**Test Command:**
```bash
cargo test --test memory_profiling_test --release -- --nocapture
```

### EventLog Memory (Scaled Analysis)

| Scale | Traces | Events | Est. Memory | Per-Event | Status |
|-------|--------|--------|------------|-----------|--------|
| 1M | 10k | 1,000,000 | 450 MB | 450 B | ✓ PASS |
| 10M | 100k | 10,000,000 | 4.2 GB | 420 B | ✓ PASS |
| 100M | 1M | 100,000,000 | 41 GB | 410 B | ✓ PASS |

**Optimization Impact:**
- String interning: 15-25% reduction
- Attribute deduplication: 20-30% reduction
- Adjacency lists: 10-15% reduction
- **Total achievable: 40-45% reduction**

### PetriNet Memory

| Complexity | Places | Transitions | Arcs | Memory | Status |
|-----------|--------|-------------|------|--------|--------|
| Medium | 100 | 100 | 200 | 8.5 MB | ✓ PASS |
| High | 500 | 500 | 1500 | 42 MB | ✓ PASS |

### Conformance Memory

| Operation | Events | Memory | Status |
|-----------|--------|--------|--------|
| TokenReplay | 1M | 450 MB | ✓ PASS |
| Streaming | 1M | 120 MB | ✓ PASS |
| Peak vs Baseline | — | -73% | ✓ GOOD |

```

## 4. Distributed Speedup Baseline Template

```markdown
## Distributed Speedup Baseline

**Test Command:**
```bash
cargo test --test distributed_speedup_test --release -- --nocapture
```

### Speedup Results

| Nodes | Events | Time (s) | Speedup | Efficiency | Status |
|-------|--------|----------|---------|-----------|--------|
| 1 | 1M | 12.5 | — | — | Baseline |
| 2 | 1M | 7.2 | 1.75x | 87.5% | ✓ PASS |
| 3 | 2M | 5.1 | 2.45x | 81.7% | ⚠ MISS |
| 5 | 5M | 3.4 | 3.68x | 73.6% | ⚠ MISS |
| 8 | 10M | 2.3 | 5.43x | 67.9% | ⚠ MISS |

**Status Legend:**
- ✓ PASS = Target met (≥target speedup AND ≥target efficiency)
- ⚠ MISS = Below target but within acceptable margin (< 10%)
- ✗ FAIL = Significant regression (> 10% below target)

### Performance Breakdown by Phase

| Phase | 2-node | 3-node | 5-node | 8-node |
|-------|--------|--------|--------|--------|
| Log Partition | 0.2s | 0.3s | 0.5s | 0.8s |
| Parallel Discovery | 4.0s | 3.2s | 2.4s | 1.6s |
| Result Merge | 2.8s | 1.4s | 0.4s | 0.1s |
| Soundness Check | 0.2s | 0.2s | 0.1s | 0.1s |
| **Total** | **7.2s** | **5.1s** | **3.4s** | **2.3s** |

**Analysis:**
- Partition overhead: ~1.5% per node
- Merge overhead: ~39% (2-node), ~27% (3-node), ~12% (5-node)
- Serial fraction: ~8% (estimated from 8-node: 1 - speedup/8)

```

## 5. Conformance Baseline Template

```markdown
## Conformance Checking Baseline

**Test Command:**
```bash
cargo test --test load_testing test_concurrent_conformance_50_simultaneous --release -- --nocapture
```

### TokenReplay Performance

| Scale | Events | Time | Fitness | Success Rate | Memory |
|-------|--------|------|---------|--------------|--------|
| 50 concurrent | 1M | 8.2s | 0.85 | 100% | 450 MB |

**Detailed Results:**

| Metric | Value | Notes |
|--------|-------|-------|
| Total traces replayed | 10,000 | All successful |
| Fitness (average) | 0.85 | 85% match |
| Fitness (min) | 0.72 | Worst case |
| Fitness (max) | 1.00 | Perfect match |
| Execution time variance | ±2% | Stable across runs |

```

## 6. Stress Test Baseline Template

```markdown
## Stress Scenario Baseline

**Test Commands:**
```bash
cargo test --test stress_scenarios scenario --release -- --nocapture
```

### Summary

| Scenario | Status | Time | Notes |
|----------|--------|------|-------|
| Rapid-fire (5k events) | ✓ PASS | 8.2s | Event creation + discovery |
| Memory pressure (50k events) | ✓ PASS | 12.5s | Allocation verified |
| Deep sequence (500 depth) | ✓ PASS | 18.3s | Pathological pattern |
| High branching (100 branches) | ✓ PASS | 15.7s | Many unique paths |
| Loop intensive (50 loops) | ✓ PASS | 19.2s | Rework handling |
| Fragmented (5k unique) | ✓ PASS | 6.3s | Single-event traces |
| Complex structure (1000 events) | ✓ PASS | 22.1s | Nested patterns |
| Cascading failures (30×3 threads) | ✓ PASS | 45.2s | Multiple errors |
| Sustained load (1000 ops) | ✓ PASS | 180.5s | Sequential operations |
| 24-hr simulation (100 concurrent) | ✓ PASS | 31.2s | Concurrent load |

```

## 7. JSON Export Format (for tracking)

```json
{
  "baseline_id": "baseline_20260324_dcee663b6",
  "timestamp": "2026-03-24T16:30:00Z",
  "system": {
    "hostname": "legion",
    "cpu": "Intel Core i9-13900KS",
    "cores": 20,
    "ram_gb": 128,
    "os": "macOS 13.6"
  },
  "commit": {
    "sha": "dcee663b6e7d28e8c29e5f3a4b8c1d9e2f3g4h5i",
    "short": "dcee663b6",
    "branch": "main"
  },
  "results": {
    "discovery_1m_time_s": 12.5,
    "discovery_1m_memory_mb": 450,
    "discovery_1m_places": 1250,
    "discovery_1m_transitions": 980,
    "conformance_1m_time_s": 8.2,
    "conformance_1m_fitness": 0.85,
    "memory_1m_eventlog_mb": 450,
    "memory_10m_eventlog_mb": 4200,
    "speedup_2node": 1.75,
    "speedup_2node_efficiency": 87.5,
    "speedup_3node": 2.45,
    "speedup_3node_efficiency": 81.7,
    "speedup_5node": 3.68,
    "speedup_5node_efficiency": 73.6,
    "speedup_8node": 5.43,
    "speedup_8node_efficiency": 67.9
  },
  "status": {
    "discovery_1m": "PASS",
    "conformance_1m": "PASS",
    "memory_1m": "PASS",
    "speedup_2node": "PASS",
    "speedup_3node": "MISS",
    "speedup_5node": "MISS",
    "speedup_8node": "MISS",
    "overall": "ACCEPTABLE"
  }
}
```

## 8. Comparison Script

```python
#!/usr/bin/env python3
"""
Compare baseline against current results
"""
import json
import sys
from pathlib import Path

def load_baseline(path):
    with open(path) as f:
        return json.load(f)

def compare(baseline, current):
    print("\n" + "="*70)
    print("BASELINE COMPARISON REPORT")
    print("="*70)

    metrics = [
        ("Discovery Time (1M)", "discovery_1m_time_s", 1.05),
        ("Discovery Memory (1M)", "discovery_1m_memory_mb", 1.10),
        ("Conformance Time (1M)", "conformance_1m_time_s", 1.05),
        ("Speedup (2-node)", "speedup_2node", 0.95),
        ("Efficiency (2-node)", "speedup_2node_efficiency", 0.95),
        ("Memory (1M EventLog)", "memory_1m_eventlog_mb", 1.10),
    ]

    all_pass = True
    for metric_name, key, threshold in metrics:
        baseline_val = baseline["results"][key]
        current_val = current["results"].get(key, baseline_val)

        if key.endswith("_time") or key.endswith("_memory"):
            # Lower is better
            change = current_val / baseline_val
            regression = change > threshold
        else:
            # Higher is better (speedup, efficiency)
            change = baseline_val / current_val if current_val > 0 else 1.0
            regression = change > threshold

        status = "✗ REGRESSION" if regression else "✓ OK"
        change_pct = (change - 1.0) * 100

        print(f"\n{metric_name}")
        print(f"  Baseline:  {baseline_val:.2f}")
        print(f"  Current:   {current_val:.2f}")
        print(f"  Change:    {change_pct:+.1f}% {status}")

        if regression:
            all_pass = False

    print("\n" + "="*70)
    print(f"Overall: {'✓ NO REGRESSIONS' if all_pass else '✗ REGRESSIONS DETECTED'}")
    print("="*70)

    return 0 if all_pass else 1

if __name__ == "__main__":
    baseline = load_baseline(Path(".benchmarks/baseline.json"))
    current = load_baseline(Path(".benchmarks/current.json"))
    sys.exit(compare(baseline, current))
```

## 9. Regression Detection

**Acceptable regressions** (within natural variation):
- ±5% for timing metrics
- ±10% for memory metrics
- ±5% for speedup/efficiency

**Investigate** if:
- Timing regression > 5%
- Memory regression > 10%
- Speedup regression > 5%

**Fail** if:
- Timing regression > 10%
- Memory regression > 20%
- Speedup regression > 10%

## 10. Version Control Strategy

Store baselines in `.benchmarks/` directory:

```bash
.benchmarks/
├── baseline_20260324_dcee663b6.json    # Current baseline
├── baseline_20260323_a1b2c3d4e.json    # Previous version
├── baseline_20260320_9z8y7x6w5.json    # Week old
└── README.md                            # Documentation
```

**Procedure:**
1. Run baseline measurements on tagged release
2. Save as `baseline_[YYYYMMDD]_[COMMIT_SHORT].json`
3. Keep for 1 month (automatic cleanup)
4. Update `baseline_LATEST.json` symlink

## Next Steps

- Capture initial baseline: `scripts/benchmark.sh`
- Compare against changes: `scripts/compare.py`
- Document findings in `DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md`
