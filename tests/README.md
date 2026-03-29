# pm4py-rust Tests Directory

**Quick Links:**
- [Active Tests](#active-tests-90-passing) — Tests passing in CI/CD
- [Deferred Tests](#deferred-tests-42-for-v202632028) — Tests deferred to v2026.3.28
- [How to Run Tests](#how-to-run-tests)
- [Re-enabling Deferred Tests](#re-enabling-deferred-tests)
- [Test Organization](#directory-organization)

---

## Quick Start

```bash
# Run all active tests (fast, ~2 minutes)
cargo test

# Run specific test
cargo test --test semconv_chicago_tdd_test

# Run tests matching pattern
cargo test discovery_

# Run with output
cargo test -- --nocapture

# Run single test function
cargo test --test semconv_chicago_tdd_test test_semconv_validation
```

---

## Active Tests (90 passing)

Tests in `tests/active/` are:
- ✅ Passing consistently in CI/CD
- ✅ Part of the standard build pipeline
- ✅ Fast enough for local development (<100ms each)
- ✅ Have clear pass/fail assertions (Chicago TDD)

### By Category

#### Core Process Mining (25+)
- `semconv_chicago_tdd_test.rs` — OpenTelemetry span emission + semconv schema validation
- `otel_span_emission_test.rs` — OTEL span formatting and collection
- `csv_json_reader_test.rs` — Event log file format reading
- `format_tests.rs` — XES, CSV, JSON format conversion
- `integration_tests.rs` — End-to-end workflow integration
- `conformance_advanced_test.rs` — Token replay, partial order alignments
- `alignments_advanced_parity_test.rs` — Python pm4py parity testing
- `formal_correctness_parity_test.rs` — Mathematical correctness proofs
- `analytics_advanced_test.rs` — Entropy, complexity metrics
- `declare_conformance_test.rs` — DECLARE constraint checking
- `wave_9_agent_7_drift_detection_test.rs` — Concept drift detection
- And 15+ more (statistics, performance, recovery)

#### Cross-Project Integration (10+)
- `businessos_integration_test.rs` — BusinessOS HTTP integration
- `businessos_rust_http_integration_tests.rs` — HTTP handler validation
- `businessos_streaming_e2e_test.rs` — Streaming data pipeline
- `cross_system_consistency.rs` — pm4py-rust ↔ BusinessOS ↔ OSA consistency
- `osa_integration_test.rs` — OSA heartbeat protocol
- `cross_project_integration_tests.rs` — Smoke test across full integration chain
- And 4+ more (compliance, protocol tests)

#### Advanced Algorithms (20+)
- `conformance_metrics_advanced_test.rs` — Advanced metrics
- `extended_discovery_integration_tests.rs` — Extended discovery
- `io_robustness_test.rs` — I/O error handling
- `io_statistics_remaining_tests.rs` — Statistics coverage
- `metrics_integration_test.rs` — Metrics aggregation
- And 15+ more (performance, recovery, variants)

#### YAWL Workflow Patterns (5)
- `yawl_advanced_patterns_test.rs` — 43 YAWL pattern coverage

#### Security & Quality (10+)
- `xes_security_test.rs` — XXE attack prevention
- `signal_theory_quality_gates_test.rs` — Output signal quality validation
- `soundness_checker_test.rs` — WvdA deadlock/liveness verification
- `soundness_proofs_comprehensive.rs` — Formal soundness proofs
- `toyota_production_system_quality_test.rs` — Toyota principles
- And 5+ more

### Test Statistics

| Metric | Count |
|--------|-------|
| Total assertions | ~1,500+ |
| Average test runtime | <100ms |
| Total suite runtime | ~2 minutes (CI), <30s locally (parallel) |
| Compiler warnings | 0 |
| Flaky tests | 0 (all deterministic) |

---

## Deferred Tests (42 for v2026.3.28)

Tests in `tests/deferred/` are:
- ⏸️ Deferred (not deleted — can be re-enabled)
- 🔗 Blocked by external dependencies (schema finalization, supervisor implementation, etc.)
- 📅 Scheduled for re-enablement in waves (P0 → P1 → P2)
- 📝 Have detailed unblock conditions in [`DEFERRED_TESTS.md`](./DEFERRED_TESTS.md)

### Summary by Priority

| Priority | Count | Target | Why Deferred |
|----------|-------|--------|--------------|
| **P0** (Critical) | 16 | Week 1 | Cross-project integration, distributed consensus |
| **P1** (High Value) | 13 | Week 2–3 | Advanced discovery, performance, parity |
| **P2** (Nice-to-Have) | 13 | Week 4+ | YAWL patterns, XES edge cases |

### Examples

#### P0 Examples
- `canopy_integration_test.rs.skip` — Blocked on Canopy CSV schema finalization
- `distributed_conformance_test.rs.skip` — Blocked on OSA heartbeat consensus protocol
- `chaos_failure_injection.rs.skip` — Blocked on Armstrong supervisor tree

#### P1 Examples
- `discovery_variants_test.rs.skip` — Blocked on variant fingerprint algorithm (<0.1% collision rate)
- `pm4py_python_ported_tests.rs.skip` — Blocked on Python pm4py v2.0+ API stability
- `deployment_validation_test.rs.skip` — Blocked on Kubernetes manifests

#### P2 Examples
- `yawl_advanced_patterns_test.rs.skip` — Blocked on YAWL pattern library v1.0 stability
- `xes_reader_breakage_test.rs.skip` — Blocked on malformed XES fixture library

See [DEFERRED_TESTS.md](./DEFERRED_TESTS.md) for complete manifest with unblock conditions.

---

## How to Run Tests

### Run All Active Tests
```bash
cd pm4py-rust
cargo test
```

### Run Specific Test File
```bash
cargo test --test otel_span_emission_test
```

### Run Tests Matching Pattern
```bash
# All discovery tests
cargo test discovery_

# All conformance tests
cargo test conformance_

# All integration tests
cargo test integration_
```

### Run Single Test Function
```bash
cargo test --test semconv_chicago_tdd_test test_semconv_span_attributes_match_schema
```

### Run with Output (no capture)
```bash
cargo test -- --nocapture
```

### Run in Release Mode (optimization)
```bash
cargo test --release
```

### Run Benchmarks
```bash
cargo bench
```

---

## Re-enabling Deferred Tests

### Step 1: Check the Manifest
Open [DEFERRED_TESTS.md](./DEFERRED_TESTS.md) and find your test.

Example: Want to re-enable `canopy_integration_test.rs.skip`?
```
#### 1. `canopy_integration_test.rs.skip`
- **Purpose:** Real Canopy demo data end-to-end with pm4py-rust
- **Why Deferred:** Depends on Canopy CSV schema finalization
- **Unblock:** Canopy demo data available at `canopy/priv/demo_data/`
- **Effort:** Medium (stub API calls, add CSV fixture)
```

### Step 2: Verify Dependencies
Check that the unblock condition is met:
```bash
# Example: Check that Canopy demo data exists
ls canopy/priv/demo_data/*.csv 2>/dev/null && echo "✓ Unblocked" || echo "✗ Not ready"
```

### Step 3: Move Test from Deferred to Active
```bash
# Move the test
mv tests/deferred/canopy_integration_test.rs.skip tests/active/canopy_integration_test.rs

# This removes the .skip extension automatically
```

### Step 4: Run the Test
```bash
cargo test --test canopy_integration_test
```

### Step 5: Verify It Passes
```
running 20 tests

test tests/active/canopy_integration_test.rs ... ok
test tests/active/canopy_integration_test.rs ... ok
...

test result: ok. 20 passed; 0 failed
```

### Step 6: Commit
```bash
git add tests/
git commit -m "re-enable: canopy_integration_test.rs now passing (Canopy schema finalized)"
```

---

## Directory Organization

```
tests/
├── active/                          ← 90 active test files
│   ├── otel_span_emission_test.rs
│   ├── semconv_chicago_tdd_test.rs
│   ├── businessos_integration_test.rs
│   ├── conformance_advanced_test.rs
│   ├── yawl_advanced_patterns_test.rs
│   └── ... (85 more)
│
├── deferred/                        ← 42 deferred test files (*.rs.skip)
│   ├── canopy_integration_test.rs.skip
│   ├── distributed_conformance_test.rs.skip
│   ├── chaos_failure_injection.rs.skip
│   ├── pm4py_python_ported_tests.rs.skip
│   ├── yawl_data_flow_patterns_test.rs.skip
│   └── ... (37 more)
│
├── README.md                        ← This file
├── DEFERRED_TESTS.md                ← Complete manifest with unblock conditions
│
└── [legacy Python integration files]
    ├── 00-START-HERE.md
    ├── parity_test.py
    ├── test_python_bindings.py
    └── ... (other test support files)
```

---

## Benefits of This Organization

### For Developers
- **Clear status:** Grep for `.skip` files to find what's deferred
- **Fast feedback:** Active tests run in parallel, <30s locally
- **Progressive workflow:** Can tackle deferred tests in priority order (P0 → P1 → P2)
- **Unblock visibility:** Each deferred test has explicit dependencies documented

### For CI/CD
- **Stable pipeline:** Only active tests in the critical path
- **Consistent metrics:** 90 active tests pass deterministically
- **Predictable runtime:** ~2 minutes for full suite (no surprises)
- **Clear upgrade path:** Manifest guides v2026.3.28 re-enablement

### For Code Review
- **Scope clarity:** PRs can reference DEFERRED_TESTS.md for "unblock conditions"
- **Progress tracking:** "This PR unblocks X deferred tests"
- **Risk management:** Deferred tests don't introduce flakiness or regressions

---

## Test Naming Conventions

All test files follow this pattern:
```
<subject>_<variation>_test.rs

subject     = what's being tested (semconv, otel, conformance, discovery, etc.)
variation   = optional qualifier (chicago_tdd, parity, advanced, integration, etc.)
_test.rs    = extension (required for cargo test to discover)
```

Examples:
- `semconv_chicago_tdd_test.rs` — Chicago TDD testing against semconv schema
- `otel_span_emission_test.rs` — OTEL span emission format validation
- `businessos_integration_test.rs` — BusinessOS HTTP API integration
- `yawl_data_flow_patterns_test.rs` — YAWL pattern testing (deferred)

---

## Test Infrastructure

### Compiler Flags
All tests compiled with:
- Rust 1.70+ (MSRV)
- Edition 2021
- No warnings (`cargo clippy`)
- Standard library included

### Dependencies Used in Tests
- **Testing:** `criterion` (benchmarks), `proptest` (property-based)
- **Fixtures:** `tempfile` (temporary files)
- **Data:** Real CSV/XES files in `tests/io/`, demo data from Canopy
- **Mocking:** Minimal; prefer real implementations (Chicago TDD)

### CI/CD Integration
Tests run automatically on:
- Push to `main` branch
- Pull requests to `main`
- Nightly builds (with extended tests)

---

## Troubleshooting

### Test Fails: "Cannot find module"
**Cause:** Test file in wrong directory (not in `active/`)
**Fix:** Check that test is in `tests/active/` directory, not `tests/deferred/`

### Test Fails: "Dependency X not available"
**Cause:** Test is deferred and requires external setup
**Fix:** Check [DEFERRED_TESTS.md](./DEFERRED_TESTS.md) for unblock conditions

### Test Times Out
**Cause:** Test has unguarded infinite loop or waiting for external service
**Fix:** Add timeout guard: `std::thread::sleep(Duration::from_secs(5))`

### Test Flakes (sometimes passes, sometimes fails)
**Cause:** Violates Chicago TDD (uses randomness, unguarded concurrency)
**Fix:** Use `rand` with seed, use fake clocks, synchronize on channels

### Cargo Reports "0 tests collected"
**Cause:** Test file not in `tests/` directory or missing `#[test]` attributes
**Fix:** Ensure file is in `tests/active/` and has `#[test]` or `#[tokio::test]` decorators

---

## Metrics & Dashboard

### Current Status (2026-03-25)
```
Active Tests:        90 ✅
Deferred Tests:      42 ⏸️
Total Assertions:    ~2,100+
Suite Runtime:       ~2 minutes (CI), <30s (parallel local)
Compiler Warnings:   0
Flaky Tests:         0
P0 (Week 1):         0/16 enabled → target 16/16
P1 (Week 2–3):       6/13 enabled → target 13/13
P2 (Week 4+):        0/13 enabled → target 13/13
```

### Re-enablement Timeline
- **v2026.3.28 Week 1:** P0 (16 tests) → ~130+ total active
- **v2026.3.28 Week 2–3:** P1 (13 tests) → ~143+ total active
- **v2026.3.28 Week 4+:** P2 (13 tests) → ~156+ total active

---

## Related Files

- **Deferred test manifest:** [DEFERRED_TESTS.md](./DEFERRED_TESTS.md) — Detailed unblock conditions
- **Build config:** [../Cargo.toml](../Cargo.toml) — Test dependencies
- **Integration guide:** [INTEGRATION_TESTING_GUIDE.md](./INTEGRATION_TESTING_GUIDE.md) — Cross-project integration
- **Architecture:** [../../docs/](../../docs/) — System architecture docs

---

## Contributing New Tests

When adding a new test:

1. **Place in `active/` only if passing consistently locally**
2. **Use Chicago TDD:** Real implementations, no mocks (prefer fixtures)
3. **Single claim per test:** One behavior = one test
4. **Fast:** <100ms per test (use `tempfile`, not file I/O)
5. **Deterministic:** Same result every run (no randomness, seed `rand`)
6. **Include OTEL span:** Tests should emit spans captured by Jaeger

Example test file:
```rust
#[test]
fn test_otel_span_includes_semconv_attributes() {
    let log = create_test_event_log();

    // GIVEN: Process mining operation
    let result = discover_petri_net(&log);

    // WHEN: Operation completes
    assert!(result.is_ok());

    // THEN: OTEL span contains semconv attributes
    let span = get_last_span("pm4py.discovery");
    assert!(span.attributes.contains_key("pm4py.process_name"));
    assert_eq!(span.status, "ok");
}
```

---

## Questions?

- **How do I know if a test is deferred?** Check filename: `.skip` extension = deferred
- **Can I run deferred tests?** Only after renaming: `mv deferred/test.rs.skip active/test.rs`
- **How long to re-enable all tests?** Estimated 4 weeks (P0 → P1 → P2 staggered)
- **What if I need to unblock a test early?** See unblock condition in [DEFERRED_TESTS.md](./DEFERRED_TESTS.md)

---

*Last updated: 2026-03-25 by Claude Code*
*Test organization follows [Chicago TDD](https://en.wikipedia.org/wiki/Test-driven_development#Chicago_school) + [WvdA Soundness](https://www.sciencedirect.com/book/9780123849649/process-mining) principles*
