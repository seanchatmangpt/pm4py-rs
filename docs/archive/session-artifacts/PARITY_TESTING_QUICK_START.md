# Parity Testing Quick Start

**Get started validating Python-Rust sync in 5 minutes.**

---

## 1. Build Python Bindings

```bash
cd /Users/sac/chatmangpt/pm4py-rust
pip install maturin
maturin develop
```

✓ This creates `pm4py_rust` Python package from Rust code.

---

## 2. Install Test Dependencies

```bash
pip install pytest pm4py
```

✓ `pytest` runs tests, `pm4py` provides comparison baseline.

---

## 3. Run Full Parity Test Suite

```bash
pytest tests/parity_validation_test.py -v
```

**Expected output (first run):**
```
collected 27 items

tests/parity_validation_test.py::TestAPIParityDiscovery::test_alpha_miner_api_exists PASSED
tests/parity_validation_test.py::TestAPIParityDiscovery::test_heuristic_miner_api_exists PASSED
...
======================== 27 passed in 1.23s ========================
```

---

## 4. Run Specific Test Class

```bash
# Test API parity only
pytest tests/parity_validation_test.py::TestAPIParityDiscovery -v

# Test behavioral parity only
pytest tests/parity_validation_test.py::TestBehavioralParityDiscovery -v

# Test edge cases only
pytest tests/parity_validation_test.py::TestEdgeCaseParityDataStructures -v

# Test performance only
pytest tests/parity_validation_test.py::TestPerformanceParityDiscovery -v
```

---

## 5. Generate Parity Matrix Report

```bash
# Run tests and save detailed output
pytest tests/parity_validation_test.py -v --tb=short > parity_results.txt

# Show summary
tail parity_results.txt
```

**Output sample:**
```
======================== PARITY VALIDATION RESULTS ========================

## Discovery Algorithms

| Function | API | Behavior | Edge Cases | Performance | Status |
|----------|-----|----------|-----------|-------------|--------|
| AlphaMiner() | ✓ | ✓ | ✓ | 1.2x | ✓ PERFECT |
| HeuristicMiner() | ✓ | ✓ | ✓ | 1.5x | ✓ PERFECT |
| InductiveMiner() | ✓ | ✓ | ✓ | 2.1x | ⚠️ GOOD |

Overall Parity Score: 95%
```

---

## 6. Troubleshooting

### Bindings Not Found

```
ImportError: cannot import name 'AlphaMiner' from pm4py_rust
```

**Fix:**
```bash
maturin develop --release
```

### pm4py Not Installed

```
WARNING: pm4py not available
```

**Fix:**
```bash
pip install pm4py>=2.7.0
```

### Tests Skip

```
SKIPPED: Rust bindings not available
```

**Reason:** Bindings not built. Run step 1 first.

---

## Test Structure at a Glance

| Category | What | Tests |
|----------|------|-------|
| **API Parity** | Same functions exposed | 10 tests |
| **Behavioral Parity** | Same outputs for same inputs | 5 tests |
| **Edge Cases** | Consistent error handling | 7 tests |
| **Performance** | Rust ≤ 3x Python time | 2 tests |
| **Integration** | Full workflows work | 2 tests |

**Total:** 27 comprehensive tests

---

## Key Test Scenarios

### API Parity
```python
# Verify miners exist and are callable
miner = AlphaMiner()
assert hasattr(miner, 'apply')
```

### Behavioral Parity
```python
# Run same algorithm in both, compare structure
rust_net = AlphaMiner().apply(rust_log)
py_net, _, _ = alpha_miner.apply(py_log)
assert rust_net.places_count() ≈ py_net.places
```

### Edge Case Parity
```python
# Test special characters, empty logs, large timestamps
log = EventLog()
trace.add_event("A-1_special.act", "2024-01-01T00:00:00Z")
assert "A-1_special.act" in stats.get_activities(log)
```

### Performance Parity
```python
# Rust should complete in reasonable time
ratio = rust_time / python_time
assert ratio < 3.0  # Rust ≤ 3x Python acceptable
```

---

## CI/CD Integration

Add to your GitHub Actions:

```yaml
- name: Run parity tests
  run: |
    pip install maturin pytest pm4py
    cd pm4py-rust
    maturin develop
    pytest tests/parity_validation_test.py -v
```

---

## Next Steps

1. ✓ Run `pytest tests/parity_validation_test.py -v`
2. Review any failures with `pytest ... --tb=long`
3. Check `PARITY_VALIDATION_REPORT.md` for detailed methodology
4. Add to pre-commit hook for continuous validation

---

## Files

- **`tests/parity_validation_test.py`** — Full test suite (500+ lines)
- **`PARITY_VALIDATION_REPORT.md`** — Detailed methodology & expected results
- **`PARITY_MATRIX.md`** — Generated parity validation results

---

**Total Time to Full Parity Check:** ~5-10 minutes
