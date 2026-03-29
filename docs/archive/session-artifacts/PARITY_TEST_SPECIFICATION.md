# Parity Test Specification

**Technical specification for Python-Rust parity validation framework.**

---

## 1. Overview

### Purpose
Ensure Python bindings expose Rust implementation with semantic equivalence.

### Scope
- **Coverage:** Discovery, Conformance, Statistics, Data Structures
- **Depth:** API, Behavioral, Edge Case, Performance validation
- **Audience:** Developers, CI/CD systems, quality assurance

### Key Definitions

| Term | Definition |
|------|-----------|
| **API Parity** | Same public functions/classes exist in both |
| **Behavioral Parity** | Same inputs produce equivalent outputs |
| **Edge Case Parity** | Error handling consistent across implementations |
| **Performance Parity** | Rust ≤ 3.0x Python time (accounting for bindings overhead) |
| **Semantic Equivalence** | Results equivalent under domain semantics, not necessarily identical |

---

## 2. Test Organization

### Test Hierarchies

```
parity_validation_test.py
├── TestAPIParityDiscovery
│   ├── test_alpha_miner_api_exists
│   ├── test_heuristic_miner_api_exists
│   ├── test_inductive_miner_api_exists
│   └── test_dfg_miner_api_exists
├── TestAPIParityConformance
│   ├── test_footprints_conformance_api_exists
│   └── test_token_replay_conformance_api_exists
├── TestAPIParityStatistics
│   └── test_log_statistics_api_exists
├── TestAPIParityDataStructures
│   ├── test_event_log_api_exists
│   ├── test_trace_api_exists
│   └── test_petri_net_api_exists
├── TestBehavioralParityDiscovery
│   ├── test_alpha_miner_behavioral_parity
│   ├── test_heuristic_miner_behavioral_parity
│   └── test_inductive_miner_behavioral_parity
├── TestBehavioralParityStatistics
│   ├── test_basic_stats_behavioral_parity
│   ├── test_activity_frequency_behavioral_parity
│   └── test_variant_behavioral_parity
├── TestEdgeCaseParityDataStructures
│   ├── test_empty_log_handling
│   ├── test_single_event_trace
│   ├── test_duplicate_activities_in_trace
│   ├── test_special_characters_in_activity_names
│   └── test_large_timestamp_range
├── TestEdgeCaseParityDiscovery
│   ├── test_single_trace_discovery
│   └── test_uniform_trace_discovery
├── TestPerformanceParityDiscovery
│   ├── test_alpha_miner_performance
│   └── test_statistics_performance
└── TestFullPipelineParityIntegration
    ├── test_discover_and_conform_pipeline
    └── test_statistics_workflow
```

### Test Count by Category

| Category | Tests | Coverage |
|----------|-------|----------|
| API Parity | 10 | All public APIs |
| Behavioral Parity | 5 | Core algorithms |
| Edge Cases | 7 | Boundary conditions |
| Performance | 2 | Speed validation |
| Integration | 2 | End-to-end workflows |
| **Total** | **27** | **Comprehensive** |

---

## 3. Test Specifications

### 3.1 API Parity Tests

**Purpose:** Verify same functions/classes exposed in both implementations.

**Test Template:**
```python
def test_[component]_api_exists(self):
    """[Description of API feature]."""
    rust_instance = [RustClass]()
    assert rust_instance is not None
    assert hasattr(rust_instance, '[method_name]')
```

**Acceptance Criteria:**
- Class/function instantiable without errors
- All expected methods present
- Methods callable with appropriate signature

**Example: AlphaMiner API Test**
```python
def test_alpha_miner_api_exists(self):
    """AlphaMiner exists in both."""
    rust_miner = AlphaMiner()
    assert rust_miner is not None
    assert hasattr(rust_miner, 'apply')
```

**Assertion Tolerance:** Exact match (0% tolerance)

---

### 3.2 Behavioral Parity Tests

**Purpose:** Verify algorithms produce equivalent outputs.

**Test Template:**
```python
def test_[algorithm]_behavioral_parity(self):
    """[Algorithm] produces equivalent models."""
    # Create identical logs
    rust_log = create_sample_log_rust()
    py_log = create_sample_log_python()

    # Run algorithm
    rust_result = [RustAlgorithm]().apply(rust_log)
    py_result = [PyAlgorithm].apply(py_log)

    # Compare metrics
    assert rust_result.metric_a ≈ py_result.metric_a
```

**Acceptance Criteria:**
- Results structurally equivalent
- Metrics within tolerance
- No crashes or exceptions

**Metrics Compared by Algorithm:**

| Algorithm | Metrics |
|-----------|---------|
| **Discovery** | Places, transitions, arcs |
| **Conformance** | Fitness score, traces_fit |
| **Statistics** | Trace count, event count, variants |

**Assertion Tolerance:**
- Discovery metrics: ±2 nodes (internal representation variance)
- Statistics: Exact match (0% tolerance)
- Conformance: ±0.05 fitness (5% tolerance)

**Example: Alpha Miner Behavioral Parity**
```python
def test_alpha_miner_behavioral_parity(self, simple_log_rust, simple_log_python):
    """Alpha Miner produces equivalent models."""
    rust_miner = AlphaMiner()
    rust_net = rust_miner.apply(simple_log_rust)

    py_net, _, _ = alpha_miner.apply(simple_log_python)

    rust_places = rust_net.places_count()
    py_places = len(py_net.places)

    assert abs(rust_places - py_places) <= 2
```

---

### 3.3 Edge Case Parity Tests

**Purpose:** Verify consistent error handling and boundary conditions.

**Test Template:**
```python
def test_[edge_case]_handling(self):
    """[Edge case description]."""
    log = [create_edge_case_log]()

    try:
        result = [operation](log)
        assert [validation](result)
    except [ExpectedException]:
        # Both should fail same way
        pass
```

**Acceptance Criteria:**
- No crashes
- Error messages consistent
- Graceful failure or success

**Edge Cases Tested:**

| Case | Input | Expected Behavior |
|------|-------|-------------------|
| **Empty Log** | 0 traces | Explicit error or empty result |
| **Single Event** | 1 event | Processed correctly |
| **Duplicate Activities** | A,A,A in trace | Counted correctly |
| **Special Characters** | A-1_special.act | Preserved correctly |
| **Large Time Range** | 2020-2024 | Handled correctly |
| **Single Trace** | 1 trace | Model or explicit error |
| **Uniform Traces** | All identical | Simplified model |

**Example: Empty Log Handling**
```python
def test_empty_log_handling(self):
    """Empty logs handled gracefully."""
    log = EventLog()
    assert len(log) == 0

    stats = LogStatistics()
    result = stats.basic_stats(log)
    # Either returns zeros or raises with clear message
    assert result["num_traces"] == 0 or "empty" in str(e).lower()
```

**Assertion Tolerance:** Exact behavior match (0% tolerance on error type)

---

### 3.4 Performance Parity Tests

**Purpose:** Verify Rust implementation meets performance expectations.

**Test Template:**
```python
def test_[operation]_performance(self):
    """[Operation] timing within acceptable range."""
    log = create_test_log()

    # Rust timing
    start = time.perf_counter()
    rust_result = [RustOp]().apply(log)
    rust_time = time.perf_counter() - start

    # Python timing
    start = time.perf_counter()
    py_result = [PyOp].apply(log)
    py_time = time.perf_counter() - start

    ratio = rust_time / py_time
    assert ratio < 3.0  # Acceptable range
```

**Acceptance Criteria:**
- Rust ≤ 3.0x Python time
- Ideal: 0.5x–2.0x (faster or comparable)
- No timeouts

**Performance Baselines:**

| Operation | Python Time | Rust Target |
|-----------|-------------|------------|
| Create 100-event log | ~5ms | <10ms |
| AlphaMiner 100 events | ~150ms | <450ms |
| Statistics (100 events) | ~5ms | <15ms |

**Assertion Tolerance:**
- Rust ≤ 3.0x Python time acceptable
- Ideal: 0.5x–2.0x ratio

**Example: Alpha Miner Performance**
```python
def test_alpha_miner_performance(self, complex_log_rust, complex_log_python):
    """Rust Alpha Miner ≤ 3x Python time."""
    rust_miner = AlphaMiner()
    start = time.perf_counter()
    rust_net = rust_miner.apply(complex_log_rust)
    rust_time = time.perf_counter() - start

    start = time.perf_counter()
    py_net, _, _ = alpha_miner.apply(complex_log_python)
    py_time = time.perf_counter() - start

    ratio = rust_time / py_time if py_time > 0 else 0
    assert ratio < 3.0
```

---

### 3.5 Integration Tests

**Purpose:** Verify complete workflows work end-to-end.

**Test Template:**
```python
def test_[workflow]_pipeline(self):
    """[Workflow] works end-to-end."""
    log = create_test_log()

    # Step 1
    result1 = [Operation1](log)
    assert result1 is not None

    # Step 2
    result2 = [Operation2](result1)
    assert result2 is not None

    # Verify final state
    assert [validation](result2)
```

**Acceptance Criteria:**
- No crashes
- All steps complete
- Final result valid

**Workflows Tested:**

1. **Discovery → Conformance**
   ```
   EventLog → AlphaMiner → PetriNet → Conformance → Fitness
   ```

2. **Full Statistics Analysis**
   ```
   EventLog → LogStatistics → Metrics (traces, events, variants)
   ```

**Example: Discovery & Conformance Pipeline**
```python
def test_discover_and_conform_pipeline(self, simple_log_rust):
    """Discovery → Conformance workflow."""
    # Discover
    rust_miner = AlphaMiner()
    rust_net = rust_miner.apply(simple_log_rust)
    assert rust_net is not None

    # Conform
    rust_conformer = FootprintsConformanceChecker()
    rust_result = rust_conformer.apply(rust_net, simple_log_rust)

    # Validate
    if hasattr(rust_result, 'fitness'):
        assert 0.0 <= rust_result.fitness <= 1.0
```

---

## 4. Test Data & Fixtures

### 4.1 Simple Log

**Purpose:** Minimal test case for basic validation.

**Specification:**
- **Traces:** 10
- **Pattern:** A → B → C (linear)
- **Events:** 30 (10 × 3)
- **Variants:** 1
- **Case names:** case_0 through case_9
- **Timestamps:** Sequential within traces

**Creation Code:**
```python
log = EventLog()
for i in range(10):
    trace = Trace(f"case_{i}")
    trace.add_event("A", f"2024-01-01T{i:02d}:00:00Z")
    trace.add_event("B", f"2024-01-01T{i:02d}:01:00Z")
    trace.add_event("C", f"2024-01-01T{i:02d}:02:00Z")
    log.add_trace_obj(trace)
```

**Expected Metrics:**
- Num traces: 10
- Num events: 30
- Num activities: 3 (A, B, C)
- Avg trace length: 3.0
- Variants: {A,B,C: 10}

---

### 4.2 Complex Log

**Purpose:** Multiple patterns for variant testing.

**Specification:**
- **Traces:** 20
- **Patterns:** 3 variants (A→B→C→D, A→B→D→C, A→C→B→D)
- **Distribution:** 7, 7, 6 traces per variant
- **Events:** 80 (20 × 4)
- **Activities:** 4 (A, B, C, D)

**Expected Metrics:**
- Num traces: 20
- Num events: 80
- Num variants: 3
- Avg trace length: 4.0

---

### 4.3 Fixtures

**Setup:**
```python
@pytest.fixture
def simple_log_rust():
    """Create simple Rust event log."""
    return [EventLog]()  # See above

@pytest.fixture
def complex_log_rust():
    """Create complex Rust event log."""
    return [EventLog]()  # 20 traces, 3 patterns
```

**Teardown:** Automatic (fixtures destroyed after test)

---

## 5. Assertion Specifications

### 5.1 Assertion Types

| Type | Syntax | Tolerance | Use Case |
|------|--------|-----------|----------|
| **Exact** | `assert a == b` | 0% | Counts, activities |
| **Range** | `assert abs(a - b) <= n` | ±n | Place/transition counts |
| **Ratio** | `assert a/b < threshold` | <3.0x | Performance |
| **Contains** | `assert x in y` | 0% | Set membership |
| **Type** | `assert isinstance(a, T)` | 0% | Type checking |

### 5.2 Error Messages

All assertions include descriptive messages:

```python
# Bad
assert rust_places == py_places

# Good
assert rust_places == py_places, \
    f"Place count mismatch: Rust={rust_places}, Python={py_places}"
```

---

## 6. Test Execution

### 6.1 Execution Order

Tests execute in class order (declared order):
1. API Parity (10 tests)
2. Behavioral Parity (5 tests)
3. Edge Case Parity (7 tests)
4. Performance Parity (2 tests)
5. Integration (2 tests)

**Rationale:** API → Behavior → Edge Cases → Performance ensures logical flow.

### 6.2 Isolation

Each test:
- Creates fresh fixtures
- No shared state
- Independent assertions
- Can run in any order

### 6.3 Markers

```python
# Skip if bindings unavailable
@pytest.mark.skipif(not RUST_BINDINGS_AVAILABLE, ...)

# Skip if Python pm4py unavailable
@pytest.mark.skipif(not PYTHON_PM4PY_AVAILABLE, ...)

# Skip slow tests
@pytest.mark.slow
```

---

## 7. Reporting

### 7.1 Parity Matrix

**Format:** Markdown table

**Headers:**
- Function
- API (✓/✗)
- Behavior (✓/✗)
- Edge Cases (✓/✗)
- Performance (ratio)
- Status (PERFECT/GOOD/PARTIAL/MISMATCH)

**Example:**
```markdown
| Function | API | Behavior | Edge Cases | Performance | Status |
|----------|-----|----------|-----------|-------------|--------|
| AlphaMiner() | ✓ | ✓ | ✓ | 1.2x | ✓ PERFECT |
```

### 7.2 Summary Metrics

```
Total Functions Tested: 12
Perfect Parity:        10 (83%)
Good Parity:           2  (17%)
Partial Parity:        0  (0%)
Mismatch:              0  (0%)

Overall Parity Score:  100%
```

### 7.3 Output Files

- **`parity_validation_test.py`** — Test source code
- **`PARITY_VALIDATION_REPORT.md`** — Full methodology
- **`PARITY_MATRIX.md`** — Results table (generated)

---

## 8. Failure Handling

### 8.1 Test Failure Categories

| Category | Cause | Action |
|----------|-------|--------|
| **API Mismatch** | Missing function | Update bindings |
| **Behavioral Mismatch** | Different algorithm | Debug algorithm logic |
| **Edge Case Failure** | Inconsistent error handling | Add error handling |
| **Performance Failure** | Too slow | Profile and optimize |
| **Flaky Test** | Non-deterministic | Add wait/retry logic |

### 8.2 Debugging

```bash
# Verbose output
pytest tests/parity_validation_test.py -vv

# Show all assertions
pytest tests/parity_validation_test.py -vv --tb=long

# Stop on first failure
pytest tests/parity_validation_test.py -x

# Show captured logs
pytest tests/parity_validation_test.py -v --log-cli-level=DEBUG
```

### 8.3 Common Issues

| Issue | Fix |
|-------|-----|
| ImportError | Run `maturin develop` |
| Assertion mismatch | Check log creation |
| Timeout | Use smaller test logs |
| Flaky | Add retry logic |

---

## 9. Maintenance

### 9.1 When to Update Tests

- New algorithms added → Add API test
- Algorithm semantics change → Update behavioral test
- New edge cases discovered → Add edge case test
- Performance regresses → Update baseline

### 9.2 Test Stability

**Flaky Test Criteria:**
- Passes/fails unpredictably
- Random test order dependency
- Non-deterministic timing

**Mitigation:**
- Deterministic fixtures
- Explicit waits
- No global state

### 9.3 Review Checklist

Before committing:
- [ ] All tests pass locally
- [ ] Coverage ≥ 90%
- [ ] Performance baseline met
- [ ] Edge cases documented
- [ ] Assertions have messages
- [ ] No flaky tests

---

## 10. Metrics & KPIs

### Test Coverage

```
Target: 90% comprehensive coverage
  - API: 100% (all public methods)
  - Behavioral: 85% (major algorithms)
  - Edge Cases: 80% (documented scenarios)
  - Integration: 70% (common workflows)
```

### Test Quality

| Metric | Target | Method |
|--------|--------|--------|
| **Pass Rate** | >95% | CI/CD runs |
| **Execution Time** | <2 minutes | time.perf_counter() |
| **Flakiness** | <2% | 10 runs × all tests |
| **Coverage** | >90% | pytest-cov |

### Parity Score

```
Overall Parity Score = (perfect + good) / total × 100%
Target: ≥95%
Acceptable: ≥85%
```

---

## References

### Standards
- IEEE 1849: Event Logs
- pm4py specification
- Petri Net Notation

### Tools
- pytest: https://docs.pytest.org/
- PyO3: https://pyo3.rs/
- maturin: https://www.maturin.rs/

---

**Version:** 1.0
**Status:** Final
**Last Updated:** 2026-03-24
