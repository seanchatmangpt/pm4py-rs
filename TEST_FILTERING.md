# pm4py-rust Test Filtering Guide

Fast feedback loops for test development in pm4py-rust.

## Quick Commands

### Run all tests
```bash
cargo test
```

### Run only library tests (pure logic, no integration)
```bash
cargo test --lib
```

### Run only integration tests
```bash
cargo test --test '*'
```

### Run all tests with specific name
```bash
cargo test process_mining
cargo test algorithm::dijkstra
```

### Run only benchmarks
```bash
cargo bench
```

### Run fast tests (<100ms per test)
```bash
cargo test -- --test-threads=4
```

### Run with output on success
```bash
cargo test -- --nocapture
```

### Run single test with full output
```bash
cargo test test_name -- --nocapture --test-threads=1
```

## Test Organization

pm4py-rust follows Cargo's standard test structure:

```
pm4py-rust/
├── src/
│   └── lib.rs          # Library tests via #[cfg(test)] modules
├── tests/
│   ├── integration_test.rs
│   ├── conformance_test.rs
│   └── ...             # Integration tests (run with `cargo test --test '*'`)
└── benches/
    └── perf.rs        # Benchmarks (run with `cargo bench`)
```

## Test Filters by Category

### Library Tests (Fast Feedback)
**Run:** `cargo test --lib`
- Location: `src/lib.rs` with `#[cfg(test)]` modules
- Speed: <500ms total
- Use for: Unit tests, algorithm verification, quick iteration

### Integration Tests (Thorough Validation)
**Run:** `cargo test --test '*'`
- Location: `tests/` directory
- Speed: 1-5s total
- Use for: End-to-end flows, API validation, cross-component testing

### Benchmarks (Performance Verification)
**Run:** `cargo bench`
- Location: `benches/` directory
- Speed: Variable (profiling)
- Use for: Performance regression detection, optimization validation

## Common Test Scenarios

### Working on a specific algorithm
```bash
# Run tests for dijkstra algorithm
cargo test dijkstra -- --nocapture

# Run tests + see debug output
cargo test dijkstra -- --nocapture --test-threads=1
```

### Before committing code
```bash
# Run all library tests (fast)
cargo test --lib

# If passing, run all tests including integration
cargo test
```

### After optimization work
```bash
# Verify no regressions
cargo test

# Check performance improvements
cargo bench algorithm::dijkstra
```

### Debugging a failing test
```bash
# Run with full output
cargo test test_name -- --nocapture --test-threads=1

# Run with RUST_BACKTRACE
RUST_BACKTRACE=1 cargo test test_name
```

## CI/CD Test Matrix

- **Fast feedback:** `cargo test --lib` (5 sec)
- **Full validation:** `cargo test` (30 sec)
- **Performance baseline:** `cargo bench` (2 min)

## Cargo.toml Test Configuration

Tests can be configured with:

```toml
[[test]]
name = "integration_test"
path = "tests/integration_test.rs"

[[bench]]
name = "perf"
path = "benches/perf.rs"
harness = true
```

## Test Tagging Pattern

For tests that should only run in certain conditions:

```rust
#[test]
#[ignore]  // Skip by default; run with: cargo test -- --ignored
fn slow_algorithm_test() {
    // Expensive test
}

// Or conditionally compile:
#[cfg(test)]
mod tests {
    #[cfg(not(feature = "slow_tests"))]
    mod fast {
        // Fast unit tests
    }

    #[cfg(feature = "slow_tests")]
    mod slow {
        // Slow integration tests
    }
}
```

## Best Practices

1. **Unit tests in lib.rs** — Keep fast (<100ms each)
2. **Integration tests in tests/** — Can be slower, test cross-module flows
3. **Benchmarks in benches/** — Track performance over time
4. **Use --nocapture for debugging** — See println! output during test runs
5. **Test-thread control** — Use `--test-threads=1` for tests that need isolation
6. **Feature gates for optional tests** — Use `#[cfg(feature = "...")]` for slow tests

## Troubleshooting

### Tests hang or timeout
```bash
# Run with timeout (requires test crate)
cargo test -- --test-threads=1 --timeout 30
```

### Need to debug a test
```bash
# Print all output and run sequentially
RUST_LOG=debug cargo test test_name -- --nocapture --test-threads=1
```

### Run only tests matching a pattern
```bash
cargo test algorithm
cargo test dijkstra::test
```

---

**Quick Reference:**
- Library tests: `cargo test --lib` (fast, for iteration)
- All tests: `cargo test` (thorough, for validation)
- Benchmarks: `cargo bench` (performance, for profiling)
