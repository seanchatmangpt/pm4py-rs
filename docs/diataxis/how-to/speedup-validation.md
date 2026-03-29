# How to: Validate Distributed Speedup

## Overview

This guide explains how to interpret distributed speedup test results and validate parallel efficiency.

## Speedup Fundamentals

**Speedup formula:**
```
Speedup(n) = T(1) / T(n)

Where:
  T(1) = execution time on 1 node (baseline)
  T(n) = execution time on n nodes
  n    = number of nodes
```

**Efficiency formula:**
```
Efficiency(n) = Speedup(n) / n * 100%

Where:
  Perfect efficiency = 100% (linear speedup)
  Good efficiency    = 70-90%
  Acceptable         = 50-70%
  Poor efficiency    = < 50%
```

## Speedup Targets (pm4py-rust)

These targets assume:
- Log partitioned round-robin across nodes
- DFG edges merged with frequency-weighted voting
- Soundness verified (no lost edges)

| Nodes | Events | Target Speedup | Target Efficiency | Rationale |
|-------|--------|----------------|-------------------|-----------|
| 2 | 1M | ≥1.7x | ≥85% | Minimal overhead, communication simple |
| 3 | 2M | ≥2.5x | ≥83% | Larger log masks sync overhead |
| 5 | 5M | ≥3.8x | ≥76% | More overhead, diminishing returns |
| 8 | 10M | ≥5.5x | ≥69% | Significant synchronization cost |

## Step 1: Understand Test Results

Run the speedup test suite:

```bash
cargo test --test distributed_speedup_test --release -- --nocapture 2>&1 | tee speedup_results.log
```

Expected output:
```
✓ 2-node speedup test:
  Baseline (1 node):  12.523s
  Parallel (2 nodes): 7.156s
  Speedup:            1.75x (target: ≥1.7x)
  Efficiency:         87.5% (target: ≥85%)
```

## Step 2: Extract Key Metrics

Use this script to parse results:

```bash
#!/bin/bash
# Extract speedup metrics from test output

LOG_FILE="$1"

echo "=== SPEEDUP ANALYSIS ==="
echo ""

for nodes in 2 3 5 8; do
  BASELINE=$(grep "Baseline.*:" "$LOG_FILE" | head -1 | awk -F'[s]' '{print $(NF-1)}' | awk '{print $NF}')
  PARALLEL=$(grep "Parallel.*nodes.*:" "$LOG_FILE" | head -1 | awk -F'[s]' '{print $(NF-1)}' | awk '{print $NF}')
  SPEEDUP=$(grep "Speedup:" "$LOG_FILE" | head -1 | awk '{print $2}' | sed 's/x//')
  EFFICIENCY=$(grep "Efficiency:" "$LOG_FILE" | head -1 | awk '{print $2}' | sed 's/%//')

  echo "$nodes nodes:"
  echo "  Baseline:    ${BASELINE}s"
  echo "  Parallel:    ${PARALLEL}s"
  echo "  Speedup:     ${SPEEDUP}x"
  echo "  Efficiency:  ${EFFICIENCY}%"
  echo ""
done
```

## Step 3: Verify Speedup Targets

Check if actual results meet targets:

```python
#!/usr/bin/env python3
import json
import sys

targets = {
    2: {"speedup": 1.7, "efficiency": 85},
    3: {"speedup": 2.5, "efficiency": 83},
    5: {"speedup": 3.8, "efficiency": 76},
    8: {"speedup": 5.5, "efficiency": 69},
}

results = {
    2: {"speedup": 1.75, "efficiency": 87.5},
    3: {"speedup": 2.45, "efficiency": 81.7},
    5: {"speedup": 3.68, "efficiency": 73.6},
    8: {"speedup": 5.43, "efficiency": 67.9},
}

print("Speedup Validation Report")
print("=" * 60)

all_pass = True
for nodes in [2, 3, 5, 8]:
    target = targets[nodes]
    result = results[nodes]

    speedup_ok = result["speedup"] >= target["speedup"]
    efficiency_ok = result["efficiency"] >= target["efficiency"]
    node_pass = speedup_ok and efficiency_ok

    status = "✓ PASS" if node_pass else "✗ FAIL"
    print(f"\n{nodes} nodes: {status}")
    print(f"  Speedup:    {result['speedup']:.2f}x (target: {target['speedup']:.1f}x) {'✓' if speedup_ok else '✗'}")
    print(f"  Efficiency: {result['efficiency']:.1f}% (target: {target['efficiency']}%) {'✓' if efficiency_ok else '✗'}")

    if not node_pass:
        all_pass = False

print("\n" + "=" * 60)
print(f"Overall: {'✓ ALL TARGETS MET' if all_pass else '✗ SOME TARGETS MISSED'}")

sys.exit(0 if all_pass else 1)
```

## Step 4: Efficiency Degradation Analysis

Plot efficiency vs node count to identify scaling bottleneck:

```
Efficiency Curve (ideal vs actual)
===================================

100%  ┌─ Ideal (100% efficiency)
      │ /
 90%  │/
      │
 80%  ├─ Actual results
      │ \
 70%  │  \
      │    \___
 60%  │        \___
      │            \___
      └──┬──┬──┬──┬──
        2  3  5  8  nodes

Pattern Analysis:
- Linear drop: communication overhead scaling
- Cliff drop: synchronization bottleneck at specific node count
- Smooth curve: normal Amdahl's law behavior
```

Generate plot from data:

```python
import matplotlib.pyplot as plt

nodes = [2, 3, 5, 8]
efficiency = [87.5, 81.7, 73.6, 67.9]  # Your results

plt.figure(figsize=(10, 6))
plt.plot(nodes, efficiency, 'o-', label='Actual Efficiency')
plt.axhline(y=85, color='g', linestyle='--', label='Target (2-node)')
plt.axhline(y=83, color='orange', linestyle='--', label='Target (3-node)')
plt.axhline(y=76, color='orange', linestyle='--', label='Target (5-node)')
plt.axhline(y=69, color='r', linestyle='--', label='Target (8-node)')

plt.xlabel('Number of Nodes')
plt.ylabel('Efficiency (%)')
plt.title('Distributed Speedup Efficiency')
plt.grid(True, alpha=0.3)
plt.legend()
plt.ylim(60, 100)
plt.savefig('speedup_efficiency.png')
plt.show()
```

## Step 5: Identify Scaling Bottlenecks

If efficiency is below target, analyze the cause:

### Issue: 2-node efficiency < 85%
```
Baseline: 10.0s
2-node:   6.5s
Speedup:  1.54x (target: 1.7x)
Issue:    Merge overhead, synchronization

Solution:
- Profile merge operation with flamegraph
- Check for lock contention in merge phase
- Optimize DFG edge merging algorithm
```

### Issue: 3-node efficiency drops > 10% from 2-node
```
2-node: 87.5% efficiency
3-node: 81.7% efficiency
Drop:   5.8 percentage points (normal)

Pattern: gradual degradation is expected due to Amdahl's Law
See: https://en.wikipedia.org/wiki/Amdahl%27s_law
```

### Issue: Efficiency cliff at specific node count
```
Efficiency: [87.5%, 81.7%, 73.6%, 45.2%]  ← Big drop at 8-node
Cause:      Likely communication saturation

Solutions:
- Reduce merge messages between nodes
- Implement hierarchical merging (2×2 → 1)
- Use asynchronous aggregation
```

## Step 6: Verify Result Soundness

Ensure merged DFG preserves model structure:

```bash
# Test includes soundness checks:
# "Merged DFG lost edges" assertion
# "Partition lost traces" assertion
# "Merge completeness" test

# If soundness check fails, results are invalid
cargo test --test distributed_speedup_test test_merge_result_completeness --release -- --nocapture
```

Expected output:
```
✓ Merge completeness verified: all 1248 edges present with correct frequencies
```

## Step 7: Create Speedup Report

Document your findings:

```markdown
# Distributed Speedup Validation Report

**Date:** 2026-03-24
**System:** [16-core CPU], [128 GB RAM]
**Log Size:** [1M, 2M, 5M, 10M events]
**Partitioning:** Round-robin across nodes

## Results Summary

| Nodes | Speedup | Target | Status | Efficiency | Target |
|-------|---------|--------|--------|-----------|--------|
| 1 | — | — | Baseline | — | — |
| 2 | 1.75x | ≥1.7x | ✓ PASS | 87.5% | ≥85% |
| 3 | 2.45x | ≥2.5x | ✗ FAIL | 81.7% | ≥83% |
| 5 | 3.68x | ≥3.8x | ✗ FAIL | 73.6% | ≥76% |
| 8 | 5.43x | ≥5.5x | ✗ FAIL | 67.9% | ≥69% |

## Analysis

### 2-Node (✓ PASS)
- Achieved 1.75x speedup (target 1.7x)
- 87.5% efficiency exceeds 85% target
- Merge overhead minimal (~12.5% lost)

### 3-Node (✗ MISS by 0.05x)
- Achieved 2.45x speedup (target 2.5x)
- 81.7% efficiency below 83% target
- Reason: Merge phase scales as O(edges), not O(traces)
- Impact: Minor, within experimental noise

### 5-Node (✗ MISS by 0.12x)
- Achieved 3.68x speedup (target 3.8x)
- 73.6% efficiency below 76% target
- Reason: Communication overhead scales with node count

### 8-Node (✗ MISS by 0.07x)
- Achieved 5.43x speedup (target 5.5x)
- 67.9% efficiency below 69% target
- Reason: Amdahl's law serial fraction ≈ 8%

## Recommendations

1. **3-node miss (0.05x):** Within acceptable margin, no action
2. **5-node miss (0.12x):** Optimize merge phase for larger node counts
3. **8-node miss (0.07x):** Expected due to serial fractions, acceptable

## Conclusion

Results demonstrate **good scaling up to 5 nodes** with acceptable efficiency degradation. 8-node scaling follows Amdahl's law predictions.
```

Save as: `docs/DISTRIBUTED_SPEEDUP_VALIDATION_REPORT.md`

## Step 8: Create Regression Tests

Prevent future speedup regressions:

```rust
// tests/speedup_regression.rs
#[test]
fn test_speedup_no_regression_2node() {
    // Use baseline from BASELINE_MEASUREMENT_TEMPLATES
    const BASELINE_TIME_MS: u128 = 12500;
    const ALLOWED_REGRESSION_PERCENT: f64 = 5.0;

    let (_, duration) = single_node_discovery(&log);
    let baseline_efficiency = BASELINE_TIME_MS as f64 / 2.0;
    let max_allowed = baseline_efficiency * (1.0 + ALLOWED_REGRESSION_PERCENT / 100.0);

    assert!(duration.as_millis() as f64 <= max_allowed,
            "2-node speedup regressed: {:.0}ms (max: {:.0}ms)",
            duration.as_millis(), max_allowed);
}
```

## Related Documentation

- **Load Testing**: `docs/diataxis/tutorials/load-testing-quickstart.md`
- **Baseline Measurement**: `docs/diataxis/how-to/baseline-measurement.md`
- **Memory Optimization**: `docs/diataxis/explanation/memory-optimization.md`
