# PM4PY-Rust Parity Testing - Quick Index

## 📋 Quick Navigation

### Start Here
1. **[PARITY_TESTING_README.md](PARITY_TESTING_README.md)** - Overview & quick start (5 min read)
2. **[PARITY_VALIDATION_SUMMARY.md](PARITY_VALIDATION_SUMMARY.md)** - Executive summary (2 min read)
3. **[tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md)** - Complete reference (30 min read)

### Test Code
- **[tests/parity_validation_test.py](tests/parity_validation_test.py)** - 100+ tests, 600 lines
- **[tests/parity_matrix_generator.py](tests/parity_matrix_generator.py)** - Report builder, 400 lines
- **[tests/run-parity-tests.sh](tests/run-parity-tests.sh)** - Automated runner

### Reports
- **[PARITY_VALIDATION_MATRIX.md](tests/PARITY_VALIDATION_MATRIX.md)** - Detailed analysis (generated)
- **[parity_validation_results.json](tests/parity_validation_results.json)** - Machine-readable (generated)

### Existing Docs
- **[PARITY_MATRIX.md](PARITY_MATRIX.md)** - Capability comparison (606 lines)
- **[PARITY_SUMMARY.txt](PARITY_SUMMARY.txt)** - Gap analysis

## 🚀 Quick Start Commands

```bash
# 1. Build bindings (first time)
cd pm4py-rust
maturin develop

# 2. Run all tests
pytest tests/parity_validation_test.py -v

# 3. Generate reports
python3 tests/parity_matrix_generator.py \
  --output PARITY_VALIDATION_MATRIX.md \
  --json parity_validation_results.json

# 4. Or use the full runner
./tests/run-parity-tests.sh
```

## 📊 Current Status

```
Overall Parity: 60.4%
├── Data Structures:  100% ✅ (8/8)
├── Statistics:        71% ⚠️  (5/7)
├── I/O Formats:       60% ⚠️  (6/10)
├── Discovery:         50% ⚠️  (4/8)
├── Filtering:         50% ⚠️  (2/6)
├── Conformance:       33% ❌  (2/6)
└── Analysis:          33% ❌  (1/3)

Tests:     100+
Functions: 48
Performance: 0.80x (Rust 25% faster)
```

## 📚 Documentation Map

### For Getting Started
- **5 min**: [PARITY_TESTING_README.md](PARITY_TESTING_README.md) → What & Why
- **2 min**: [PARITY_VALIDATION_SUMMARY.md](PARITY_VALIDATION_SUMMARY.md) → Status & Gaps
- **10 min**: [tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md) → How to run

### For Understanding Results
- **10 min**: [PARITY_VALIDATION_MATRIX.md](tests/PARITY_VALIDATION_MATRIX.md) → Detailed analysis
- **5 min**: [parity_validation_results.json](tests/parity_validation_results.json) → Raw data
- **20 min**: [PARITY_MATRIX.md](PARITY_MATRIX.md) → Complete capability matrix

### For Running Tests
- **1 min**: `pytest tests/parity_validation_test.py -v`
- **5 min**: `python3 tests/parity_matrix_generator.py --output report.md`
- **10 min**: `./tests/run-parity-tests.sh`

### For Developing
- **30 min**: [tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md) → Testing patterns
- **15 min**: [tests/parity_validation_test.py](tests/parity_validation_test.py) → Code examples
- **10 min**: [tests/parity_matrix_generator.py](tests/parity_matrix_generator.py) → Adding functions

## 🎯 Key Questions Answered

### "What is parity testing?"
→ See [PARITY_TESTING_README.md](PARITY_TESTING_README.md) → "What Is Parity Testing?"

### "How do I run the tests?"
→ See [PARITY_TESTING_README.md](PARITY_TESTING_README.md) → "Running Tests"

### "What's the current parity score?"
→ See [PARITY_VALIDATION_SUMMARY.md](PARITY_VALIDATION_SUMMARY.md) → "Test Results"

### "Which features are missing?"
→ See [PARITY_VALIDATION_SUMMARY.md](PARITY_VALIDATION_SUMMARY.md) → "Critical Gaps"

### "Is this ready for production?"
→ See [PARITY_TESTING_README.md](PARITY_TESTING_README.md) → "Current Status"

### "How do I add a new test?"
→ See [tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md) → "Adding New Tests"

### "What does PERFECT vs PARTIAL mean?"
→ See [PARITY_TESTING_README.md](PARITY_TESTING_README.md) → "Interpreting Results"

### "How do I use this in CI/CD?"
→ See [tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md) → "Integration with CI/CD"

## 📁 File Structure

```
pm4py-rust/
├── tests/
│   ├── parity_validation_test.py           [600 lines]  Test suite
│   ├── parity_matrix_generator.py          [400 lines]  Report builder
│   ├── run-parity-tests.sh                 [150 lines]  Runner script
│   ├── PARITY_VALIDATION_GUIDE.md          [500 lines]  Reference
│   ├── PARITY_VALIDATION_MATRIX.md         [200 lines]  Generated report
│   ├── parity_validation_results.json      [300 lines]  Generated data
│   └── parity_reports/                     [Archive]    Historical reports
│
├── PARITY_TESTING_README.md                [700 lines]  Overview
├── PARITY_TESTING_INDEX.md                 [This file]  Navigation
├── PARITY_VALIDATION_SUMMARY.md            [400 lines]  Executive summary
├── PARITY_MATRIX.md                        [606 lines]  Capability matrix
└── PARITY_SUMMARY.txt                      [210 lines]  Gap analysis
```

## 🔍 Test Categories

### API Parity (25 tests)
- Function existence
- Method availability
- Constructor behavior
→ See `TestAPIParity*` classes

### Behavioral Parity (35 tests)
- Model equivalence
- Output matching
- Statistics correctness
→ See `TestBehavioralParity*` classes

### Edge Cases (22 tests)
- Empty logs
- Special characters
- Large timestamps
- Loop patterns
→ See `TestEdgeCase*` classes

### Performance (8 tests)
- Speed ratios
- Memory efficiency
→ See `TestPerformance*` classes

### Integration (12 tests)
- Full workflows
- End-to-end scenarios
→ See `TestFullPipeline*` classes

## ✨ Key Metrics

```
100+ Test Methods
15 Test Classes
48 Functions Analyzed
1000+ Lines of Test Code
2000+ Lines of Documentation
60.4% Overall Parity
100% Data Structure Parity
```

## 🎓 Learning Path

### Beginner (30 minutes)
1. Read [PARITY_TESTING_README.md](PARITY_TESTING_README.md)
2. Review [PARITY_VALIDATION_SUMMARY.md](PARITY_VALIDATION_SUMMARY.md)
3. Run: `pytest tests/parity_validation_test.py -v -k "APIParity"`

### Intermediate (2 hours)
1. Read [tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md)
2. Study [PARITY_VALIDATION_MATRIX.md](tests/PARITY_VALIDATION_MATRIX.md)
3. Run full suite: `./tests/run-parity-tests.sh`
4. Explore test code: [tests/parity_validation_test.py](tests/parity_validation_test.py)

### Advanced (Full day)
1. Understand report generation: [tests/parity_matrix_generator.py](tests/parity_matrix_generator.py)
2. Add new tests to [tests/parity_validation_test.py](tests/parity_validation_test.py)
3. Update matrix database in [tests/parity_matrix_generator.py](tests/parity_matrix_generator.py)
4. Run with coverage: `pytest tests/parity_validation_test.py --cov`

## 🔗 Cross References

### Signal Theory Integration
- See: [../../docs/diataxis/explanation/signal-theory-complete.md](../../docs/diataxis/explanation/signal-theory-complete.md)
- Mode: `code` | Genre: `spec` | Type: `commit` | Format: `python` | Structure: `test-suite`

### Chatman Equation
- See: [../../docs/diataxis/explanation/chatman-equation.md](../../docs/diataxis/explanation/chatman-equation.md)
- A = μ(O): Parity tests are transformation functions verifying semantic equivalence

## 📞 Support

### Questions?
- Check [tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md) → "Troubleshooting"
- Search parity_validation_results.json for your function
- Review test patterns in test code

### Issues?
- File GitHub issue with label `parity-test`
- Attach generated report from `tests/parity_reports/`
- Reference [PARITY_VALIDATION_MATRIX.md](tests/PARITY_VALIDATION_MATRIX.md)

### Contributing?
1. Read [tests/PARITY_VALIDATION_GUIDE.md](tests/PARITY_VALIDATION_GUIDE.md) → "Adding New Tests"
2. Create test in [tests/parity_validation_test.py](tests/parity_validation_test.py)
3. Add function to [tests/parity_matrix_generator.py](tests/parity_matrix_generator.py)
4. Regenerate report and include in PR

---

**Version:** 1.0
**Last Updated:** 2026-03-24
**Status:** Production Ready
**Maintainer:** ChatmanGPT Team

**[START HERE →](PARITY_TESTING_README.md)**
