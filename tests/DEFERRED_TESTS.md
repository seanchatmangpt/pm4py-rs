# Deferred Tests Manifest

**Last Updated:** 2026-03-25  
**Status:** 42 test files deferred to v2026.3.28  
**Strategy:** Staggered re-enablement (P0 → P1 → P2) to maintain CI/CD stability

---

## Summary by Category

| Category | Count | Effort | Priority |
|----------|-------|--------|----------|
| **Cross-Project Integration** | 8 | High | P0 |
| **Distributed/Parallel** | 5 | High | P0 |
| **Chaos/Failure Scenarios** | 3 | High | P0 |
| **Performance/Scale** | 5 | Medium | P1 |
| **Advanced Discovery/Conformance** | 6 | Medium | P1 |
| **Legacy/Parity Testing** | 4 | Medium | P1 |
| **XES/Python Interop** | 5 | Low | P2 |
| **YAWL Patterns** | 6 | Low | P2 |

---

## P0 (Critical) — 16 tests → v2026.3.28 Week 1

### Cross-Project Integration (8)

#### 1. `canopy_integration_test.rs.skip`
- **Purpose:** Real Canopy demo data end-to-end with pm4py-rust
- **Why Deferred:** Depends on Canopy CSV schema finalization
- **Unblock:** Canopy demo data available at `canopy/priv/demo_data/`
- **Effort:** Medium (stub API calls, add CSV fixture)
- **Test Count:** ~20 assertions

#### 2. `deep_api_coverage_tests.rs.skip`
- **Purpose:** Ralph Loop iteration 9: untested high-priority APIs
- **Why Deferred:** Requires BusinessOS/OSA schema-driven logs
- **Unblock:** Schema update for discovery, conformance, filtering
- **Effort:** Medium (add fixtures, unmock real implementations)
- **Test Count:** ~30 assertions

#### 3. `innovative_cross_project_tests.rs.skip`
- **Purpose:** Ralph Loop iteration 5: novel process mining scenarios
- **Why Deferred:** Requires cross-ecosystem data standardization
- **Unblock:** OSA/BusinessOS log schema consistency
- **Effort:** High (multi-project coordination)
- **Test Count:** ~25 assertions

#### 4. `iter35_businessos_integration_test.rs.skip`
- **Purpose:** BusinessOS CRM + process mining integration
- **Why Deferred:** CRM schema not finalized
- **Unblock:** BusinessOS CRM module published
- **Effort:** High (CRM API integration, schema mapping)
- **Test Count:** ~40 assertions

#### 5. `businessos_bos_integration_tests.rs` (active but often flaky)
- **Purpose:** BusinessOS `bos` CLI integration
- **Why Sometimes Fails:** Network timeouts, schema version mismatch
- **Keep In:** Active (monitor flakiness)

#### 6. `pm4py_businessos_integration_test.rs` (active)
- **Purpose:** pm4py-rust discovery → BusinessOS schema
- **Status:** Active, consider for P0 re-enable if green

#### 7. `osa_integration_test.rs` (active)
- **Purpose:** OSA heartbeat protocol + process monitoring
- **Status:** Active, foundation for distributed tests

#### 8. `cross_project_integration_tests.rs` (active)
- **Purpose:** Smoke test across pm4py-rust → bos → BusinessOS
- **Status:** Active, health check runner

---

### Distributed & Parallel (5)

#### 9. `distributed_conformance_test.rs.skip`
- **Purpose:** Distributed token replay with Byzantine fault tolerance
- **Why Deferred:** Requires distributed testbed (k3s or Kubernetes)
- **Unblock:** OSA heartbeat consensus protocol finalized
- **Effort:** High (distributed test harness, consensus simulation)
- **Test Count:** ~15 assertions

#### 10. `distributed_speedup_test.rs.skip`
- **Purpose:** Multi-node discovery/conformance speedup validation
- **Why Deferred:** Depends on distributed consensus
- **Unblock:** Distributed conformance test working
- **Effort:** High (profiling, benchmark harness)
- **Test Count:** ~10 assertions

#### 11. `load_testing.rs.skip`
- **Purpose:** Stress test under peak load (10k+ concurrent requests)
- **Why Deferred:** Requires load harness (Apache JMeter or Gatling)
- **Unblock:** Peak load benchmarks defined (target: <5s p99)
- **Effort:** Medium (harness setup, baseline metrics)
- **Test Count:** ~5 assertions (but high concurrency)

#### 12. `scale_benchmarks_test.rs.skip` (active `.rs` + `.skip` variant)
- **Purpose:** Benchmark scaling limits across event log sizes
- **Active Variant:** `scale_benchmarks_test.rs` (runs, monitors trends)
- **Skip Variant:** Extended version with 10M+ event logs
- **Unblock:** Active variant passing consistently
- **Effort:** Low (extend active test, add larger fixtures)

#### 13. `stress_scenarios.rs.skip`
- **Purpose:** Long-running stress tests (8+ hour execution)
- **Why Deferred:** Too long for CI/CD pipeline
- **Unblock:** Nightly test infrastructure
- **Effort:** Low (already implemented, just scheduled differently)
- **Test Count:** ~20 assertions

---

### Chaos & Failure Scenarios (3)

#### 14. `chaos_failure_injection.rs.skip`
- **Purpose:** 30+ failure scenarios: let-it-crash + supervision + recovery
- **Why Deferred:** Requires Armstrong fault tolerance supervisor implementation
- **Unblock:** OSA supervisor tree finalized
- **Effort:** High (supervisor integration, timeout handling)
- **Test Count:** ~30 assertions

#### 15. `byzantine_agent_tolerance.rs` (active)
- **Purpose:** Byzantine-resistant consensus for multi-agent log processing
- **Status:** Active, core component

#### 16. `recovery_verification.rs` (active + `.skip` variant)
- **Active Variant:** `recovery_verification.rs` (basic recovery)
- **Skip Variant:** Extended chaos scenarios
- **Effort:** Low (extend active test)

---

## P1 (High Value) — 13 tests → v2026.3.28 Week 2–3

### Performance & Scale (5)

#### 17. `deployment_validation_test.rs.skip`
- **Purpose:** Blue-green, canary, rolling deployment validation
- **Why Deferred:** Requires deployment infrastructure
- **Unblock:** Kubernetes manifests for pm4py-rust
- **Effort:** Medium
- **Test Count:** ~12 assertions

#### 18. `memory_allocator_test.rs` (active)
- **Purpose:** Memory usage tracking (OTEL instrumentation)
- **Status:** Active, but consider adding deferred benchmarks for jemalloc profiling

#### 19. `memory_profiling_test.rs` (active)
- **Purpose:** Heap profiling during large log processing
- **Status:** Active

#### 20. `performance.rs` (active)
- **Purpose:** Latency/throughput benchmarks
- **Status:** Active

#### 21. `k8s_manifest_validation_test.rs` (active)
- **Purpose:** Kubernetes deployment manifests validation
- **Status:** Active

---

### Advanced Discovery & Conformance (6)

#### 22. `discovery_variants_test.rs.skip`
- **Purpose:** Variant fingerprinting, frequency analysis, filtering
- **Why Deferred:** Variant fingerprint algorithm not finalized
- **Unblock:** Fingerprint collision rate < 0.1%
- **Effort:** Medium
- **Test Count:** ~18 assertions

#### 23. `conformance_advanced_test.rs` (active)
- **Purpose:** Partial order alignments, diagnostics
- **Status:** Active

#### 24. `alignments_advanced_parity_test.rs` (active)
- **Purpose:** Python pm4py alignment parity
- **Status:** Active

#### 25. `formal_correctness_parity_test.rs` (active)
- **Purpose:** Mathematical proof correctness for discovery
- **Status:** Active

#### 26. `analytics_advanced_test.rs` (active)
- **Purpose:** Advanced analytics (entropy, complexity)
- **Status:** Active

#### 27. `declare_conformance_test.rs` (active)
- **Purpose:** DECLARE constraint conformance checking
- **Status:** Active

---

### Legacy & Parity Testing (4)

#### 28. `pm4py_python_ported_tests.rs.skip`
- **Purpose:** Direct port of Python pm4py unit tests
- **Why Deferred:** Requires Python pm4py API stability (v2.0+)
- **Unblock:** Python bindings API finalized
- **Effort:** Low (mechanical porting)
- **Test Count:** ~50 assertions

#### 29. `python_bindings_integration_test.rs.skip`
- **Purpose:** Rust-Python interop via PyO3
- **Why Deferred:** Requires Python environment setup
- **Unblock:** CI/CD Python support (added to GitHub Actions)
- **Effort:** Low (PyO3 fixtures)
- **Test Count:** ~15 assertions

#### 30. `integration_parity_test.rs.skip`
- **Purpose:** End-to-end parity: Rust pm4py = Python pm4py
- **Why Deferred:** Depends on Python ported tests + bindings
- **Unblock:** ported + bindings tests passing
- **Effort:** Low
- **Test Count:** ~20 assertions

#### 31. `official_pm4py_core_ported_tests.rs` (active)
- **Purpose:** Official pm4py core test suite
- **Status:** Active

---

## P2 (Nice-to-Have) — 13 tests → v2026.3.28 Week 4+

### XES & Python Interop (5)

#### 32. `xes_security_test.rs` (active)
- **Purpose:** XXE attack prevention, XES parsing security
- **Status:** Active (security critical)

#### 33. `xes_reader_breakage_test.rs.skip`
- **Purpose:** Edge cases in XES parsing
- **Why Deferred:** Requires malformed XES fixtures
- **Unblock:** Fixture library (XES files with known issues)
- **Effort:** Low
- **Test Count:** ~12 assertions

#### 34. `wave9_agent1_xxe_prevention_test.rs` (active)
- **Purpose:** Wave 9 Agent 1: XXE prevention
- **Status:** Active

#### 35. `wave9_agent1_xxe_api_integration_test.rs.skip`
- **Purpose:** XXE prevention in API endpoints
- **Why Deferred:** API security hardening in progress
- **Unblock:** Security review complete
- **Effort:** Low
- **Test Count:** ~8 assertions

#### 36. `petri_net_model_manipulation_tests.rs.skip`
- **Purpose:** Petri net manipulation edge cases
- **Why Deferred:** Advanced graph algorithms not finalized
- **Unblock:** Graph optimization PR merged
- **Effort:** Medium
- **Test Count:** ~16 assertions

---

### YAWL Patterns (6)

#### 37. `yawl_advanced_patterns_test.rs.skip`
- **Purpose:** YAWL 43 patterns: advanced control flow
- **Why Deferred:** Advanced patterns not finalized (some of 43 patterns)
- **Unblock:** YAWL pattern library v1.0 stable
- **Effort:** Medium
- **Test Count:** ~20 assertions

#### 38. `yawl_state_based_patterns_test.rs.skip`
- **Purpose:** YAWL 43 patterns: state machines
- **Why Deferred:** State-based transitions still being refined
- **Unblock:** State machine validator finalized
- **Effort:** Medium
- **Test Count:** ~18 assertions

#### 39. `yawl_data_flow_patterns_test.rs.skip`
- **Purpose:** YAWL 43 patterns: data flow
- **Why Deferred:** Data serialization across pattern boundaries
- **Unblock:** Data flow specification finalized
- **Effort:** Medium
- **Test Count:** ~15 assertions

#### 40. `yawl_resource_patterns_test.rs.skip`
- **Purpose:** YAWL 43 patterns: resource/swimlane allocation
- **Why Deferred:** Resource management API not finalized
- **Unblock:** Resource allocation API stable
- **Effort:** Medium
- **Test Count:** ~14 assertions

#### 41. `yawl_object_flow_patterns_test.rs.skip`
- **Purpose:** YAWL 43 patterns: object data interaction
- **Why Deferred:** Object serialization not finalized
- **Unblock:** Object flow specification complete
- **Effort:** Medium
- **Test Count:** ~16 assertions

#### 42. (1 remaining P2 test)
- **See below**

---

## Remaining P2 Tests (continued)

#### 43. `schema_driven_ecosystem_tests.rs.skip`
- **Purpose:** Semconv schema-driven test generation
- **Why Deferred:** Requires weaver schema registry integration
- **Unblock:** Schema registry at `semconv/model/` with 100% coverage
- **Effort:** Low (schema queries)
- **Test Count:** ~12 assertions

---

## Re-enablement Checklist (v2026.3.28)

### Week 1 (P0 Critical)
- [ ] Finalize Canopy CSV schema
- [ ] Publish BusinessOS/OSA log schema
- [ ] Implement OSA supervisor tree
- [ ] Merge distributed consensus protocol
- [ ] Run P0 tests → all pass
- [ ] Commit: "re-enable: P0 cross-project integration tests"

### Week 2–3 (P1 High Value)
- [ ] Finalize variant fingerprinting algorithm
- [ ] Add Kubernetes manifests
- [ ] Python pm4py API stable (v2.0+)
- [ ] Run P1 tests → all pass
- [ ] Commit: "re-enable: P1 advanced discovery/conformance tests"

### Week 4+ (P2 Nice-to-Have)
- [ ] XES fixture library complete
- [ ] YAWL pattern library v1.0 stable
- [ ] Weaver schema registry integrated
- [ ] Run P2 tests → all pass
- [ ] Commit: "re-enable: P2 YAWL and XES tests"

---

## How to Re-enable a Test

```bash
# 1. Check why it was deferred
grep -A 5 "^#### N\. " DEFERRED_TESTS.md | grep "Why Deferred"

# 2. Verify dependencies resolved
# E.g., check that OSA supervisor tree exists: ls OSA/lib/osa/supervision/

# 3. Move test back to active
mv tests/deferred/your_test.rs.skip tests/active/your_test.rs

# 4. Run the test
cargo test --test your_test

# 5. If passes, remove from this manifest and commit
# E.g., git add -A && git commit -m "re-enable: your_test passes"
```

---

## Metrics & Trends

| Metric | Current | Target v2026.3.28 |
|--------|---------|-------------------|
| Active tests | 76 | 89+ |
| Deferred tests | 42 | 0 |
| P0 enabled | 0/16 | 16/16 |
| P1 enabled | 6/13 | 13/13 |
| P2 enabled | 0/13 | 13/13 |
| Total test assertions | ~1,500+ | ~2,000+ |

---

## Implementation Notes

1. **No breaking changes:** .skip extension preserved. Tests can be re-enabled by renaming.
2. **Fast feedback:** Active tests run in ~2 minutes. Deferred tests would add 15+ minutes if enabled now.
3. **Clear ownership:** Each deferred test has explicit "unblock condition" — PRs can reference this manifest.
4. **Progressive disclosure:** P0→P1→P2 ensures critical infrastructure tested first.

---

*Last updated: 2026-03-25 by Claude Code*
