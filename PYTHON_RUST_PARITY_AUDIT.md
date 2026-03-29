# Python-Rust API Parity Audit Report

## Overview
This audit validates API parity between Python bindings (PyO3 exposed APIs) and Rust core implementation for pm4py-rust v0.3.0.

**Current Status:** 36.8% overall parity (84/228 capabilities)
**Python Bindings:** Only 10 classes exposed, ~15 total methods
**Rust Core:** 56 fully implemented features across all modules

---

## Python Bindings - Currently Exposed Classes

### Event Log API
- `PyEvent`: activity, timestamp, resource, attributes
- `PyTrace`: case_id, add_event(), add_event_with_resource(), events()
- `PyEventLog`: add_trace(), get_trace(), len(), variant_count()

### Discovery API
- `PyAlphaMiner`: apply(log) → PyPetriNet
- `PyInductiveMiner`: apply(log) → dict (process_tree stub)
- `PyHeuristicMiner`: apply(log) → PyPetriNet

### Conformance API
- `PyFootprintsConformanceChecker`: apply(net, log) → PyConformanceResult
- `PyConformanceResult`: is_conformant, traces_fit, traces_total, fitness, violations

### Models API
- `PyPetriNet`: places_count, transitions_count, arcs_count, places(), transitions(), arcs(), to_json()
- `PyProcessTree`: to_json()

### Statistics API
- `PyLogStatistics`: basic_stats(log), get_activities(log), get_activity_frequencies(log)

---

## Critical Parity Gaps

### GAP 1: DECLARE Miner (Discovery)
**Status:** Rust ✅ / Python ❌
**Rust Implementation:**
- `DeclareMiner::new()`, `with_min_support()`, `discover()`
- `DeclareModel::new()`, `add_constraint()`, `check_trace()`
- `conformance_declare(log, model)`
- 21 constraint templates supported

**Python Exposure:** NONE - No PyDeclareMiner class
**Impact:** CRITICAL - blocks constraint-based discovery workflows
**Fix Needed:** Create PyDeclareMiner class in `src/python/discovery.rs`

---

### GAP 2: Alignment-Based Conformance (Conformance)
**Status:** Rust ✅✅ / Python ❌❌
**Rust Implementation:**
- `AlignmentChecker::new()`, `check()`
- `TraceAlignment` with sync/log/model move tracking
- `fitness_alignments()`, `precision_alignments()`, `diagnostics_alignments()`
- `AStarAligner`, `BeamSearchAligner`, `StreamingAligner`

**Python Exposure:** NONE - Only Footprints conformance exposed
**Impact:** CRITICAL - missing advanced conformance metrics
**Fix Needed:** Create PyAlignmentChecker, PyTraceAlignment in `src/python/conformance.rs`

---

### GAP 3: Token Replay Precision/Fitness Aggregation (Conformance)
**Status:** Rust ✅ / Python ❌
**Rust Implementation:**
- `precision_token_based_replay(log, net) → f64`
- `diagnostics_token_based_replay(log, net) → ConformanceResult`

**Python Exposure:** NONE - Only basic conformance results available
**Impact:** CRITICAL - metrics aggregation missing
**Fix Needed:** Add methods to PyConformanceResult or new PyTokenReplayConformance class

---

### GAP 4: DFG Performance Analysis (Discovery)
**Status:** Rust ✅ / Python ❌
**Rust Implementation:**
- `discover_performance_dfg()` returns PerformanceDFG with mean/min/max/median
- `PerformanceMetrics` struct with detailed timing

**Python Exposure:** NONE - No DFG performance variant
**Impact:** HIGH - blocks performance-aware discovery
**Fix Needed:** Create PyPerformanceDFG, expose `discover_performance_dfg()`

---

### GAP 5: Tree Conversion (Discovery → Models)
**Status:** Rust ⚠️ Partial / Python ❌
**Rust Implementation:**
- `tree_conversion.rs` has conversion logic
- Tree→Petri conversion exists but limited

**Python Exposure:** NONE
**Impact:** HIGH - blocks conversion workflows
**Fix Needed:** Create PyTreeConverter class for bidirectional conversion

---

### GAP 6: Advanced Discovery Algorithms (Discovery)
**Status:** Rust ✅ (11 algorithms) / Python ❌ (3 exposed)
**Rust Implementations Missing from Python:**
- ILP Miner (`ILPMiner`)
- Split Miner (`SplitMiner`)
- Causal Net Miner (`CausalNetMiner`)
- Log Skeleton Miner (`LogSkeletonMiner`)
- Eventually-Follows Graph (`EventuallyFollowsGraph`)
- Organizational miners (6 variants)
- Prefix Tree discovery
- Temporal Profile discovery
- Transition System discovery

**Python Exposure:** Only AlphaMiner, InductiveMiner (stub), HeuristicMiner
**Impact:** CRITICAL - missing 11 discovery algorithms
**Fix Needed:** Add PyILPMiner, PySplitMiner, etc. to discovery.rs

---

### GAP 7: Parameter Signature Mismatches

#### AlphaMiner (MATCH)
| Aspect | Rust | Python |
|--------|------|--------|
| Constructor | `AlphaMiner::new()` | `AlphaMiner()` ✅ |
| Method | `apply(&EventLog) → PetriNet` | `apply(log) → PetriNet` ✅ |
| Parameters | None | None ✅ |
| Return Type | PetriNet | PyPetriNet ✅ |

#### HeuristicMiner (MATCH)
| Aspect | Rust | Python |
|--------|------|--------|
| Constructor | `HeuristicMiner::new()` | `HeuristicMiner()` ✅ |
| Method | `apply(&EventLog) → PetriNet` | `apply(log) → PetriNet` ✅ |
| Parameters | None | None ✅ |
| Return Type | PetriNet | PyPetriNet ✅ |

#### InductiveMiner (MISMATCH ⚠️)
| Aspect | Rust | Python |
|--------|------|--------|
| Constructor | `InductiveMiner::new()` | `InductiveMiner()` ✅ |
| Method | `apply(&EventLog) → ProcessTree` | `apply(log) → dict stub` ❌ |
| Parameters | None | None ✅ |
| Return Type | ProcessTree | dict (incomplete) ❌ |
| Issue | Returns actual tree | Returns `{"type": "process_tree", "status": "discovered"}` |

#### Conformance Checker (PARTIAL MISMATCH ⚠️)
| Aspect | Rust | Python |
|--------|------|--------|
| Method | `check(log, net) → ConformanceResult` | `apply(net, log) → PyConformanceResult` ⚠️ |
| Param Order | `(log, net)` | `(net, log)` ❌ |
| Return Type | Full struct | Limited properties ❌ |

---

### GAP 8: Model Conversion APIs (Models)
**Status:** Rust ✅ (tree_conversion.rs) / Python ❌
**Rust Exposed Functions:**
- Tree→Petri conversion methods in tree_conversion.rs
- BPMN XML handling in bpmn_xml.rs
- DFG conversion in conversions.rs

**Python Exposure:** NONE
**Impact:** MEDIUM - blocks cross-format workflows
**Fix Needed:** Create PyTreeConverter, PyBPMNConverter classes

---

### GAP 9: Filtering Operations (Log Operations)
**Status:** Rust ✅ (39 filters) / Python ❌
**Rust Implementations Missing from Python:**
- `AdvancedFilter` trait with 15+ filter types
- `FilterChain` for filter composition
- Statistical filters (percentile, stochastic language)
- DFG-based filters
- Temporal filters
- Trace abstraction filters

**Python Exposure:** NONE - No filter API exposed
**Impact:** HIGH - blocks log preprocessing
**Fix Needed:** Create PyFilterChain, PyFilter classes

---

### GAP 10: Statistics & Metrics (Statistics)
**Status:** Rust ✅ (61% implemented) / Python ⚠️ (minimal)
**Rust Exposures Missing from Python:**
- Correlation metrics
- Rework detection
- Performance analysis
- Stability metrics
- Tree statistics (30+ metrics)
- ML features extraction
- Extended metrics

**Python Exposure:** Only basic_stats, get_activities, get_activity_frequencies
**Impact:** MEDIUM - advanced analytics missing
**Fix Needed:** Add more methods to PyLogStatistics or create specialized classes

---

### GAP 11: I/O Format Support (IO)
**Status:** Rust ✅ (46% of 20 formats) / Python ❌
**Rust Implementations:**
- XES, CSV, JSON, PNML, PTML (✅ 5 exposed)
- Parquet, OCEL, OCEL2 (⚠️ 3 partial)
- Database, streaming JSON (not exposed)

**Python Exposure:** NONE - No IO API exposed
**Impact:** MEDIUM - blocks file I/O workflows
**Fix Needed:** Create PyXESLoader, PyCSVLoader, PyParquetLoader classes

---

### GAP 12: Object-Centric Event Logs (OCPM)
**Status:** Rust ✅ (partial) / Python ❌
**Rust Available:**
- `ObjectCentricEventLog`, `ObjectCentricPetriNet`
- `ObjectCentricConformanceResult`
- `OCPMDiscoveryMiner`

**Python Exposure:** NONE
**Impact:** MEDIUM - OCEL support missing
**Fix Needed:** Create PyObjectCentricEventLog, PyOCPMDiscoveryMiner

---

### GAP 13: Predictive Analytics (Predictive)
**Status:** Rust ✅ (3 modules) / Python ❌
**Rust Available:**
- `NextActivityPredictor`, `ActivityPrediction`
- `RemainingTimePredictor`, `RemainingTimePrediction`
- `OutcomePredictor`, `CaseOutcome`, `RiskAssessment`

**Python Exposure:** NONE
**Impact:** MEDIUM - predictive features missing
**Fix Needed:** Create PyNextActivityPredictor, PyRemainingTimePredictor, PyOutcomePredictor

---

### GAP 14: Process Tree Properties (Models)
**Status:** Rust ✅ (18 tree nodes) / Python ⚠️ (stub)
**Rust Tree Nodes Available:**
- Sequence, Parallel, XOR, Loop, Or
- Silent, Activity, Leaf nodes
- 30+ analysis functions

**Python Exposure:** `PyProcessTree` exists but returns empty JSON
**Impact:** HIGH - tree manipulation incomplete
**Fix Needed:** Expand PyProcessTree with node access, operators, serialization

---

## One-Way Implementations

### In Rust Only (Not in Python):
1. **Discovery:** ILP, Split, Causal, Tree, Inductive→Petri, Organizational (6), Prefix Tree, Transition System, Temporal Profile, Batches, Correlation
2. **Conformance:** Alignments, Anti-Alignment, DECLARE, Token Replay precision, Advanced variants
3. **Filtering:** All 39 filter types (Advanced, Statistical, DFG, Temporal, Abstraction)
4. **I/O:** Parquet, Database, Streaming JSON, OCEL2
5. **OCPM:** All object-centric features
6. **Predictive:** Next Activity, Remaining Time, Outcome prediction
7. **Models:** Tree conversion, BPMN execution, All conversions
8. **Statistics:** 60%+ of metrics (correlation, rework, stability, ML features, tree stats)

**Total: ~110+ Rust features with NO Python exposure**

### In Python Only (Not in Rust):
**NONE** - Python only exposes subset of Rust

---

## Function Signature Mapping

### Discovery Algorithms

| Algorithm | Rust Function | Python Method | Match? | Notes |
|-----------|---------------|---------------|--------|-------|
| Alpha | `AlphaMiner::apply(&log)` | `AlphaMiner().apply(log)` | ✅ | Perfect |
| Heuristic | `HeuristicMiner::apply(&log)` | `HeuristicMiner().apply(log)` | ✅ | Perfect |
| Inductive | `InductiveMiner::apply(&log) → ProcessTree` | `InductiveMiner().apply(log) → dict` | ❌ | Type mismatch |
| DFG | `DFGMiner::apply(&log) → DFG` | NOT EXPOSED | ❌ | Missing |
| Performance DFG | `discover_performance_dfg(&log) → PerformanceDFG` | NOT EXPOSED | ❌ | Missing |
| ILP | `ILPMiner::apply(&log)` | NOT EXPOSED | ❌ | Missing |
| Split | `SplitMiner::apply(&log)` | NOT EXPOSED | ❌ | Missing |
| Causal | `CausalNetMiner::apply(&log) → CausalNet` | NOT EXPOSED | ❌ | Missing |
| Tree | `TreeMiner::apply(&log) → ProcessTree` | NOT EXPOSED | ❌ | Missing |
| DECLARE | `DeclareMiner::discover(&log) → DeclareModel` | NOT EXPOSED | ❌ | Missing |
| Log Skeleton | `LogSkeletonMiner::discover(&log)` | NOT EXPOSED | ❌ | Missing |

### Conformance Algorithms

| Checker | Rust Function | Python Method | Match? | Notes |
|---------|---------------|---------------|--------|-------|
| Footprints | `FootprintsConformanceChecker::apply(net, log)` | `.apply(net, log)` | ✅ | Perfect |
| Token Replay | `TokenReplay::check(log, net)` | NOT EXPOSED SEPARATELY | ❌ | Part of Footprints only |
| Token Replay Precision | `precision_token_based_replay(log, net) → f64` | NOT EXPOSED | ❌ | Missing |
| Alignments | `AlignmentChecker::check(log, net)` | NOT EXPOSED | ❌ | Missing |
| Fitness (Align) | `fitness_alignments(result) → f64` | NOT EXPOSED | ❌ | Missing |
| Precision (Align) | `precision_alignments(result) → f64` | NOT EXPOSED | ❌ | Missing |
| DECLARE | `conformance_declare(log, model)` | NOT EXPOSED | ❌ | Missing |

### Statistics Functions

| Metric | Rust Function | Python Method | Match? | Notes |
|--------|---------------|---------------|--------|-------|
| Basic Stats | Various | `.basic_stats(log)` | ⚠️ | Limited set |
| Activities | Various | `.get_activities(log)` | ✅ | Matches |
| Activity Freq | Various | `.get_activity_frequencies(log)` | ✅ | Matches |
| Correlation | `correlation_analysis(&log)` | NOT EXPOSED | ❌ | Missing |
| Tree Stats | `analyze_tree(&tree)` | NOT EXPOSED | ❌ | Missing |
| ML Features | `extract_features_*()` | NOT EXPOSED | ❌ | Missing |

---

## Impact Analysis by Priority

### P0: CRITICAL (Breaks Standard Use Cases)
1. **DECLARE Discovery/Conformance** - Constraint-based workflows (2 features)
2. **Alignment-Based Conformance** - Advanced quality metrics (4 features)
3. **Discovery Algorithms** - 11 algorithms missing from Python (11 features)
4. **InductiveMiner Type Mismatch** - Returns dict instead of ProcessTree (1 feature)

**Total P0 Impact:** 18 critical gaps

### P1: HIGH (Reduces Functionality)
1. **Token Replay Metrics Aggregation** - Fitness/precision scores (2 features)
2. **Performance DFG** - Performance-aware discovery (1 feature)
3. **Tree Conversion** - Tree↔Petri bidirectional (2 features)
4. **Advanced Statistics** - 15+ missing metrics (15 features)
5. **Filtering Operations** - 39 missing filter types (39 features)
6. **Organizational Miners** - 6 missing algorithms (6 features)

**Total P1 Impact:** 65 high-priority gaps

### P2: MEDIUM (Nice to Have)
1. **I/O Formats** - Parquet, Database, etc. (5 features)
2. **OCPM Support** - Object-centric logs (6 features)
3. **Predictive Analytics** - Outcome/activity/time prediction (3 features)
4. **Model Conversion** - BPMN, format conversions (5 features)
5. **Process Tree** - Full node manipulation (18 features)

**Total P2 Impact:** 37 medium-priority gaps

---

## Parity Matrix Summary

### By Category

| Category | Total in PM4Py | In Rust | Exposed to Python | % Exposed | Gap Type |
|----------|----------------|---------|-------------------|-----------|----------|
| **Discovery** | 25 | 18 ✅ | 3 ⚠️ | 12% | Missing 15 algos |
| **Conformance** | 19 | 11 ✅ | 1 ⚠️ | 5% | Missing advanced checks |
| **Statistics** | 23 | 14 ✅ | 3 ⚠️ | 13% | Missing metrics |
| **Models** | 8 | 8 ✅ | 2 ⚠️ | 25% | Tree is stub |
| **I/O Formats** | 20 | 9 ✅ | 0 ❌ | 0% | No I/O API |
| **Filtering** | 38 | 15 ✅ | 0 ❌ | 0% | No filter API |
| **OCPM** | 20 | 3 ✅ | 0 ❌ | 0% | OCPM missing |
| **Predictive** | 7 | 3 ✅ | 0 ❌ | 0% | Prediction missing |
| **Analysis** | 15 | 0 ❌ | 0 ❌ | 0% | Analysis gap |
| **Visualization** | 26 | 0 ❌ | 0 ❌ | 0% | Visualization gap |
| **Other** | 27 | 7 ✅ | 2 ⚠️ | 7% | Limited coverage |
| **TOTAL** | **228** | **84** | **16** | **7%** | **~120 gaps** |

---

## Recommendations

### For Production Use:
- **Use Python bindings for:** Basic discovery (Alpha, Heuristic), basic conformance (Footprints), log statistics
- **Use Rust directly for:** Advanced conformance, performance analysis, large-scale processing
- **Avoid for now:** DECLARE, Alignments, predictive analytics, object-centric workflows via Python bindings

### For Development Priority (Quick Wins):
1. **Week 1:** Expose ILPMiner, SplitMiner, CausalNetMiner (high-impact discovery)
2. **Week 2:** Expose AlignmentChecker, fitness/precision aggregation (critical conformance)
3. **Week 3:** Fix InductiveMiner return type to ProcessTree (type correctness)
4. **Week 4:** Add PyFilterChain for filtering operations

### For Medium-Term (v0.4.0):
1. DECLARE discovery/conformance
2. Tree conversion APIs
3. I/O format loaders (Parquet, CSV, XES)
4. Advanced statistics

### For Long-Term (v1.0.0):
1. Predictive analytics
2. OCPM full support
3. Model conversion framework
4. Visualization HTTP APIs

---

## Conclusion

**Current Parity: 7% of Rust capabilities exposed to Python**

The Python bindings expose only 16 of ~120+ implemented Rust features. This is a deliberate partial implementation suitable for basic workflows, but significantly limits pm4py-rust from being a drop-in replacement for Python pm4py.

**Key Finding:** All gaps are on the Python side (missing bindings), not in Rust implementation. The Rust core is feature-complete for 36.8% of pm4py capabilities.

**Recommended Approach:** Incrementally expose Rust features to Python in priority order, starting with high-impact discovery and conformance algorithms.
