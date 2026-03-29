# Python-Rust Parity Validation Guide

## Overview

This comprehensive guide covers the parity validation testing framework for pm4py-rust. The goal is to ensure Python bindings produce identical results to the Rust implementation for the same inputs across all major functional categories.

## Quick Start

### 1. Build Python Bindings

```bash
cd pm4py-rust
maturin develop  # Builds extension in-place
```

### 2. Run Parity Tests

```bash
# All parity tests
pytest tests/parity_validation_test.py -v

# Specific category
pytest tests/parity_validation_test.py::TestAPIParityDiscovery -v

# With coverage
pytest tests/parity_validation_test.py -v --cov=pm4py_rust

# Generate parity matrix
python tests/parity_matrix_generator.py --output parity_matrix.md --json parity_results.json
```

## Test Categories

### 1. API Parity Tests

Verify that the same functions are available in both implementations.

**Test Classes:**
- `TestAPIParityDiscovery` - Discovery algorithms (Alpha, Heuristic, Inductive, DFG)
- `TestAPIParityConformance` - Conformance checkers (Token Replay, Footprints)
- `TestAPIParityStatistics` - Statistics functions (basic stats, frequencies, variants)
- `TestAPIParityDataStructures` - Core models (EventLog, Trace, Event, PetriNet)

**What Tests:**
- Function existence
- Method availability
- Constructor behavior

**Success Criteria:**
- Both implementations provide the same API surface

### 2. Behavioral Parity Tests

Verify that algorithms produce equivalent results for the same inputs.

**Test Classes:**
- `TestBehavioralParityDiscovery` - Discovery algorithm outputs
- `TestBehavioralParityStatistics` - Statistics correctness
- `TestIOFormatParity` - I/O serialization formats

**What Tests:**
- Model structure equivalence (place/transition counts)
- Activity frequency matching
- Variant extraction accuracy
- JSON serialization consistency

**Success Criteria:**
- Output structures match (±2% variance allowed)
- Statistics values identical
- Serialized formats valid

### 3. Edge Case Parity Tests

Verify that edge cases are handled consistently.

**Test Classes:**
- `TestEdgeCaseParityDataStructures` - Data structure edge cases
- `TestEdgeCaseParityDiscovery` - Discovery algorithm edge cases

**Test Cases:**
- Empty logs
- Single event traces
- Duplicate activities in sequence
- Special characters in activity names
- Large timestamp ranges
- Single trace discovery
- Uniform traces (all identical)
- Highly branching paths
- Loop patterns (repeated activities)

**Success Criteria:**
- Both fail gracefully on invalid inputs
- Both succeed on edge case inputs
- Error messages are informative

### 4. Performance Parity Tests

Verify that Rust implementation meets performance expectations.

**Test Classes:**
- `TestPerformanceParityDiscovery` - Discovery algorithm timing
- `TestStatisticsParityAdvanced` - Statistics calculation timing

**What Tests:**
- Alpha Miner execution time
- Statistics calculation speed
- Memory efficiency (when applicable)

**Success Criteria:**
- Rust not >3x slower than Python (accounting for binding overhead)
- Statistics calculations complete in <1s
- Memory usage reasonable

## Test Architecture

### Fixtures

#### Simple Logs
- **`simple_log_rust`**: 10 traces of A→B→C pattern
- **`simple_log_python`**: Equivalent in Python pm4py format

#### Complex Logs
- **`complex_log_rust`**: 20 traces with 3 branching patterns
- **`complex_log_python`**: Equivalent Python version

### Data Classes

#### `ParityCheckResult`
Represents test result for a single parity check:
- `function`: Function name
- `category`: Test category
- `api_parity`: API match status
- `behavior_parity`: Behavioral equivalence
- `edge_case_parity`: Edge case handling
- `performance_ratio`: Rust time / Python time
- `error_message`: Any error encountered
- `details`: Additional metadata

#### `ParityStatus`
- `PERFECT`: All checks passed (✓)
- `GOOD`: Minor differences (⚠️)
- `PARTIAL`: Partial implementation (≈)
- `MISMATCH`: Major differences (✗)
- `SKIPPED`: Test skipped (○)
- `UNAVAILABLE`: Feature not available (—)

### Reporter

The `ParityMatrixReporter` class generates comprehensive reports:

```python
reporter = ParityMatrixReporter()
reporter.add_result(result)
report = reporter.generate_matrix()
reporter.save_report(Path("report.md"))
reporter.save_json(Path("results.json"))
```

## Parity Matrix Generator

Standalone utility for generating comprehensive parity matrices.

### Usage

```bash
# Generate markdown report
python tests/parity_matrix_generator.py --output parity_matrix.md

# Generate JSON results
python tests/parity_matrix_generator.py --json results.json

# Both
python tests/parity_matrix_generator.py --output matrix.md --json results.json
```

### Database

The generator includes a comprehensive function database with:
- **Data Structures** (8 functions)
- **Discovery Algorithms** (8 functions)
- **Conformance Checking** (6 functions)
- **Statistics** (7 functions)
- **I/O Formats** (10 functions)
- **Filtering** (6 functions)
- **Analysis** (3 functions)

Total: **48 core functions** analyzed

### Output Format

**Markdown Report** (`PARITY_VALIDATION_MATRIX.md`):
- Overall statistics
- Category breakdown with scores
- Detailed parity analysis per category
- Critical gaps identification
- Recommendations
- Performance characteristics

**JSON Output** (`results.json`):
- Machine-readable results
- Timestamp and metadata
- Individual function parity details
- Category statistics

## Interpreting Results

### Overall Parity Score

- **90%+ PERFECT**: Production ready ✅
- **75-90% GOOD**: Suitable for core workflows ⚠️
- **50-75% PARTIAL**: Limited production use
- **<50% LIMITED**: Experimental only ❌

### Category Scores

| Category | 90%+ | 75-90% | 50-75% | <50% |
|----------|------|--------|--------|------|
| **Data Structures** | ✅ | | | |
| **Discovery** | ⚠️ | | | |
| **Conformance** | ⚠️ | | | |
| **Statistics** | ✅ | | | |
| **I/O** | ⚠️ | | | |
| **Filtering** | ⚠️ | | | |
| **Analysis** | ❌ | | | |

### Function-Level Status

```
✓ PERFECT     - 100% parity, all tests pass
⚠️ GOOD       - 85%+ parity, minor differences in edge cases
≈ PARTIAL     - 60%+ parity, missing some functionality
✗ MISSING     - 0% parity, not implemented
○ SKIPPED     - Test skipped (test infrastructure issue)
— UNAVAILABLE - Feature not available (optional)
```

## Common Test Patterns

### Testing Discovery Algorithm

```python
def test_algorithm_parity(self, simple_log_rust, simple_log_python):
    # Rust
    rust_miner = AlphaMiner()
    rust_net = rust_miner.apply(simple_log_rust)

    # Python
    py_net, _, _ = alpha_miner.apply(simple_log_python)

    # Compare structure
    assert abs(rust_net.places_count() - len(py_net.places)) <= 2
    assert abs(rust_net.transitions_count() - len(py_net.transitions)) <= 2
```

### Testing Statistics

```python
def test_stats_parity(self, simple_log_rust, simple_log_python):
    # Rust
    rust_stats = LogStatistics()
    rust_result = rust_stats.basic_stats(simple_log_rust)

    # Python (manual)
    py_trace_count = len(simple_log_python)
    py_event_count = sum(len(trace) for trace in simple_log_python)

    # Compare
    assert rust_result["num_traces"] == py_trace_count
    assert rust_result["num_events"] == py_event_count
```

### Testing Edge Cases

```python
def test_empty_log_parity(self):
    log = EventLog()
    stats = LogStatistics()

    try:
        result = stats.basic_stats(log)
        assert result["num_traces"] == 0
    except Exception as e:
        # Both should fail the same way
        assert "empty" in str(e).lower()
```

## Troubleshooting

### Bindings Not Available

```
ERROR: pm4py_rust bindings not available
```

**Solution**: Rebuild bindings
```bash
cd pm4py-rust
maturin develop --release
```

### Python pm4py Not Available

```
ERROR: pm4py not available
```

**Solution**: Install pm4py
```bash
pip install pm4py
```

### Performance Tests Failing

If Rust is too slow (>3x slower than Python):
1. Check for binding overhead
2. Verify test data size is reasonable
3. Check for blocking I/O operations
4. Profile with `py-spy` or `perf`

### Behavioral Tests Failing

If outputs don't match:
1. Check variance thresholds (±2% for model counts)
2. Verify test data is identical between Rust and Python
3. Check timestamp format compatibility
4. Verify activity name handling (special characters)

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Parity Tests
on: [push, pull_request]

jobs:
  parity:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-python@v2
        with:
          python-version: '3.10'

      - name: Build bindings
        run: pip install maturin && maturin develop

      - name: Install dependencies
        run: pip install pytest pm4py

      - name: Run parity tests
        run: pytest tests/parity_validation_test.py -v

      - name: Generate matrix
        run: python tests/parity_matrix_generator.py --output parity_matrix.md

      - name: Upload report
        uses: actions/upload-artifact@v2
        with:
          name: parity-report
          path: parity_matrix.md
```

## Maintenance

### Adding New Tests

1. Identify category (Discovery, Conformance, etc.)
2. Create test class: `TestXxxParity`
3. Use existing fixtures or create new ones
4. Follow naming convention: `test_xxx_parity`
5. Document expected behavior

### Updating Parity Matrix

1. Edit `tests/parity_matrix_generator.py`
2. Update `_initialize_functions()` method
3. Regenerate with: `python tests/parity_matrix_generator.py`
4. Review `PARITY_VALIDATION_MATRIX.md`

## References

- [PM4PY Documentation](https://pm4py.fit.fraunhofer.de/)
- [PM4PY-Rust Repository](https://github.com/your-org/pm4py-rust)
- [Signal Theory S=(M,G,T,F,W)](../../docs/diataxis/explanation/signal-theory-complete.md)

## Contact

For issues or questions about parity testing:
- GitHub Issues: [pm4py-rust/issues](https://github.com/your-org/pm4py-rust/issues)
- Discussions: [pm4py-rust/discussions](https://github.com/your-org/pm4py-rust/discussions)

---

**Last Updated:** 2026-03-24
**Maintained By:** ChatmanGPT Team
**Status:** Production Ready
