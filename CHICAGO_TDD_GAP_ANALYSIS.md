# Chicago TDD Gap Analysis - pm4py-rust vs Python pm4py

**Generated:** 2026-03-24
**Methodology:** Direct execution tests (NO MOCKS, ONLY REAL)
**Test File:** tests/pm4py_python_parity_chicago_tdd.rs

## Summary

| Category | Missing | Partial | Complete |
|----------|---------|---------|----------|
| Discovery | 8 | 3 | 12 |
| Conformance | 14 | 0 | 4 |
| Statistics | 7 | 2 | 8 |
| Filtering | 2 | 0 | 20 |
| I/O | 1 | 0 | 15 |
| Utils | 4 | 0 | 8 |
| Models | 5 | 0 | 4 |
| OCPM | 0 | 0 | 8 |
| Visualization | 0 | 2 | 10 |
| **TOTAL** | **41** | **7** | **81** |

**Parity Status:** 81/129 (63%) - 41 functions completely missing

---

## Critical Gaps (Priority 1)

### Discovery (8 missing)

1. **`discover_performance_dfg(log)`** - DFG with performance metrics
2. **`directly_follows_graph(log)`** - Basic directly-follows graph
3. **`eventually_follows_graph(log)`** - Eventually-follows graph
4. **`discover_bpmn_inductive(log)`** - BPMN discovery via Inductive Miner
5. **`discover_declare(log)`** - DECLARE constraints discovery
6. **`discover_temporal_profile(log)`** - Temporal profile discovery
7. **`discover_ocdfg(log)`** - Object-Centric DFG discovery
8. **`discover_powl(log)`** - POWL model discovery

### Conformance (14 missing)

9. **`fitness_footprints(log, fp)`** - Fitness via footprints
10. **`precision_token_based_replay(log, net)`** - Precision via TBR
11. **`precision_footprints(log, fp)`** - Precision via footprints
12. **`diagnostics_token_based_replay(log, net)`** - Diagnostics via TBR
13. **`diagnostics_alignments(log, net)`** - Diagnostics via alignments
14. **`diagnostics_footprints(log, fp)`** - Diagnostics via footprints
15. **`conformance_declare(log, declare)`** - DECLARE conformance
16. **`conformance_log_skeleton(log, skeleton)`** - Log skeleton conformance
17. **`conformance_temporal_profile(log, profile)`** - Temporal profile conformance
18. **`conformance_ocdfg(log, ocdfg)`** - OCDFG conformance

### Statistics (7 missing)

19. **`case_arrival_average(log)`** - Average time between case arrivals
20. **`filter_case_performance(log, min, max)`** - Filter by case duration
21. **`extract_features_dataframe(log)`** - ML feature extraction
22. **`extract_ocel_features(ocel)`** - OCEL feature extraction
23. **`embeddings_similarity(log1, log2)`** - Embedding-based similarity
24. **`structural_similarity(net1, net2)`** - Structural similarity of nets
25. **`case_overlap(log)`** - Case overlap analysis

### Utils (4 missing)

26. **`split_train_test(log, ratio)`** - Train/test split
27. **`insert_artificial_start_end(log)`** - Insert artificial start/end events
28. **`play_out(net, num_traces)`** - Generate traces from Petri net
29. **`get_stochastic_language(net, num_traces)`** - Get stochastic language

---

## Medium Priority Gaps (Priority 2)

### Filtering (2 missing)

30. **`filter_directly_follows_relation(log, a, b)`** - Filter by DF relation
31. **`filter_eventually_follows_relation(log, a, b)`** - Filter by EF relation

### Models (5 missing)

32. **`generate_marking(net)`** - Generate initial marking
33. **`solve_marking_equation(net)`** - Solve marking equation
34. **`maximal_decomposition(net)`** - Maximal decomposition
35. **`generate_process_tree()`** - Generate random process tree
36. **`parse_process_tree(str)`** - Parse process tree from string

### I/O (1 missing)

37. **`write_xes(log, path)`** - Write XES file

---

## API Signature Issues (Priority 3)

### Discovery (3 partial)

38. **`PrefixTreeMiner`** - Type not exported
39. **`InductiveMiner::discover_tree(log)`** - Method missing
40. **`DirectlyFollowsGraph.graph`** - Field name should be `edges`

### Conformance (3 partial)

41. **`ConformanceResult`** - Missing `len()`, `iter()` methods
42. **`TokenReplay::check(log, net)`** - Wrong signature
43. **`WeightedTokenReplay::check(log, net)`** - Wrong signature

### Statistics (2 partial)

44. **`LogStats.total_events`** - Field name should be `event_count`
45. **`log_statistics(log)`** - Returns different type

### Visualization (2 partial)

46. **`save_vis_dfg(dfg, path)`** - Wrong signature
47. **`save_vis_process_tree(tree, path)`** - Wrong signature

---

## Implementation Plan

### Phase 1: Core Discovery (8 functions, ~4h)
```rust
// src/discovery/extended_discovery2.rs
pub fn discover_performance_dfg(log: &EventLog) -> PerformanceDFG;
pub fn directly_follows_graph(log: &EventLog) -> DirectlyFollowsGraph;
pub fn eventually_follows_graph(log: &EventLog) -> EventuallyFollowsGraph;
pub fn discover_temporal_profile(log: &EventLog) -> TemporalProfile;
```

### Phase 2: Core Conformance (14 functions, ~8h)
```rust
// src/conformance/extended_conformance.rs
pub fn fitness_footprints(log: &EventLog, fp: &Footprints) -> f64;
pub fn precision_token_based_replay(log: &EventLog, net: &PetriNet) -> f64;
// ... 12 more
```

### Phase 3: Statistics & ML (7 functions, ~4h)
```rust
// src/statistics/ml_features.rs
pub fn extract_features_dataframe(log: &EventLog) -> DataFrame;
pub fn embeddings_similarity(log1: &EventLog, log2: &EventLog) -> f64;
// ... 5 more
```

### Phase 4: Utils & Models (9 functions, ~4h)
```rust
// src/utils/extended_utils2.rs
pub fn split_train_test(log: &EventLog, ratio: f64) -> (EventLog, EventLog);
pub fn insert_artificial_start_end(log: &mut EventLog);
pub fn play_out(net: &PetriNet, num_traces: usize) -> EventLog;
// ... 6 more
```

### Phase 5: API Fixes (7 fixes, ~2h)
- Export `PrefixTreeMiner`
- Add `len()`, `iter()` to `ConformanceResult`
- Fix field names (`total_events` → `event_count`)
- Fix method signatures

---

## Total Effort Estimate

| Phase | Functions | Hours |
|-------|-----------|-------|
| Phase 1 | 8 | 4h |
| Phase 2 | 14 | 8h |
| Phase 3 | 7 | 4h |
| Phase 4 | 9 | 4h |
| Phase 5 | 7 | 2h |
| **TOTAL** | **45** | **22h** |

---

## Test Coverage

**Current tests:** 111 test functions
**Passing:** 71 (64%)
**Failing (missing functions):** 40 (36%)

---

## Next Steps

1. Implement Phase 1 (Core Discovery) - 4h
2. Implement Phase 2 (Core Conformance) - 8h
3. Implement Phase 3 (Statistics & ML) - 4h
4. Implement Phase 4 (Utils & Models) - 4h
5. Fix API signatures (Phase 5) - 2h
6. Re-run Chicago TDD tests to verify 100% parity

---

## Notes

- All tests use REAL data (Canopy CSV files)
- NO MOCKS used per Chicago TDD methodology
- Each function is executed directly to verify it works
- Gap analysis is complete and actionable
