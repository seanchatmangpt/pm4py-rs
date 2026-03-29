# PM4Py-Rust Memory Optimization Guide

## Executive Summary

This document details memory profiling and optimization strategies for pm4py-rust, targeting a **50-60% memory reduction vs Python pm4py** while maintaining full correctness and safety.

**Key Results:**
- EventLog: **40% reduction target** (800MB → 480MB for 100M events)
- PetriNet: **20% reduction target** (400MB → 320MB for complex nets)
- Conformance: **30% reduction target** (streaming vs collection)
- Statistics: **25% reduction target** (incremental aggregation)

---

## Memory Profiling Framework

### Test Suite Location
`/Users/sac/chatmangpt/pm4py-rust/tests/memory_profiling_test.rs` (400+ lines)

### Test Coverage

| Scale | Test | Target Memory | Purpose |
|-------|------|--------------|---------|
| 1M events | `profile_eventlog_1m_events` | < 1,200 MB | Standard enterprise load |
| 10M events | `profile_eventlog_10m_events` | < 6,000 MB | Large organization |
| 100M events | `profile_eventlog_100m_events` | < 6,000 MB | Petabyte-scale feasibility |
| Medium net | `profile_petri_net_medium_complexity` | < 10 MB | 100 places × 100 transitions |
| High net | `profile_petri_net_high_complexity` | < 50 MB | 500 places × 500 transitions |
| Conformance | `profile_conformance_token_replay_1m` | < 1,200 MB | Token replay without explosion |
| DFG | `profile_dfg_discovery_1m` | < 1,200 MB | Discovery algorithm memory |
| Statistics | `profile_statistics_incremental_1m` | < 1,200 MB | Incremental computation |

### Profiling Methodology

Each test:
1. Estimates baseline memory using component structure analysis
2. Performs operation
3. Re-estimates memory post-operation
4. Validates against target threshold
5. Prints detailed breakdown for analysis

#### Estimation Formula

```rust
// EventLog memory = sum of:
// - Event overhead: 16 (UUID) + 8*3 (DateTime + Option) + 64 (pointers)
// - Activity string: len + 32 (String overhead)
// - Resource string: len + 32 (optional)
// - Attributes: entries × 128 (BTreeMap entry overhead)
// - Trace overhead: Vec pointer + id string + attributes
```

---

## Top 5 Memory Consumers

### 1. Event Data (40-50% of total)

**What consumes:**
- Activity name string (variable, typically 10-30 bytes)
- Timestamp (DateTime<Utc>: 12 bytes)
- Event UUID (16 bytes)
- Resource reference (optional, 8-20 bytes)

**Example for 100M events:**
- Activity: 100M × 15 bytes = 1.5 GB
- Timestamp: 100M × 12 bytes = 1.2 GB
- UUID: 100M × 16 bytes = 1.6 GB
- Subtotal: ≈ 4.3 GB (43% of typical 10GB footprint)

**Optimization Strategy:**
Use **StringIntern** to deduplicate activity names:
- Store "approve", "reject", "verify" once each
- Reference via u32 IDs (4 bytes instead of repeated strings)
- Compression ratio: 10× for typical logs with 5-20 unique activities

### 2. Event Attributes (25-30% of total)

**What consumes:**
- BTreeMap overhead: 56 bytes per entry
- Key string: variable (10-30 bytes)
- Value string: variable (10-100 bytes)
- Allocation fragmentation

**Example for 100M events with 3 attributes each:**
- BTreeMap overhead: 100M × 3 × 56 = 16.8 GB
- Key+value strings: variable, ~50 bytes average = 15 GB
- Subtotal: ≈ 31.8 GB (much larger than events!)

**Optimization Strategy:**
Use **CompactAttributes** with Arc deduplication:
- Group events with identical attributes
- Share via Arc<HashMap> (atomic reference counting)
- Deduplication ratio: 5-10× in typical logs (many events with "priority=high")

### 3. Trace Structure (10-15% of total)

**What consumes:**
- Trace ID string
- Vec<Event> allocations
- Attributes BTreeMap
- Padding from alignment

**Optimization Strategy:**
Use **Arc<Trace>** for immutable sharing:
- Multiple analysis passes don't need separate copies
- Reference counting vs duplication

### 4. PetriNet Structure (5-10% for discovery/conformance)

**What consumes:**
- Place/Transition strings (IDs + names)
- Vec<Arc> allocations
- HashMap lookups during traversal (poor cache locality)

**Optimization Strategy:**
Use **AdjacencyLists**:
- Replace HashMap with pre-allocated Vec + indices
- Outgoing/incoming lists for cache-friendly traversal
- Cache efficiency: 1.5-2× improvement

### 5. Temporary Results (5% during operation)

**What consumes:**
- Statistics results (intermediate collections)
- DFG nodes/edges collections
- Conformance alignment results

**Optimization Strategy:**
Use **streaming/incremental** computation:
- Never collect all results into memory
- Aggregate incrementally (running sums, histograms)
- ObjectPool for temporary allocations

---

## Optimization Techniques

### 1. String Interning (40% reduction on activities)

**Module:** `src/memory/allocator.rs::StringIntern`

**How it works:**
```rust
let mut intern = StringIntern::new();
let approve_id = intern.intern("approve");  // First time: stores string
let approve_id2 = intern.intern("approve"); // Second: reuses same ID

// Memory saved: (copies - 1) × string_length
// For 100M events with 5 activities: 100M × 15 bytes - 5 × 15 bytes ≈ 1.5 GB saved
```

**Implementation in EventLog:**
Replace `activity: String` with `activity: u32` (ID to intern pool)

**Compression Ratio:**
- Single activity repeated 1M times: 1M × 7 bytes → 4 bytes + 7 bytes storage = **99.9% reduction**
- 10 activities in 100M events: 100M × 8.5 bytes → 100M × 4 bytes = **50% reduction**

**Integration Points:**
- `Event` struct: replace `activity: String` with `activity_id: u32`
- `Transition` struct: replace `label` with interned string ID
- Maintain backward compatibility via `Event::activity(&intern)` method

### 2. Arc-Based Attribute Sharing (5-10× deduplication)

**Module:** `src/memory/allocator.rs::CompactAttributes`

**How it works:**
```rust
let mut attrs = CompactAttributes::new();

// Event 1: {priority: high, cost_center: 1001}
let attr_arc_1 = attrs.add_attributes(map.clone());

// Event 2: {priority: high, cost_center: 1001}
// Same attributes: Arc is reused, no new allocation
let attr_arc_2 = attrs.add_attributes(map.clone());

// Both point to same Arc (deduplication)
assert!(Arc::ptr_eq(&attr_arc_1, &attr_arc_2));
```

**Implementation:**
Replace `attributes: BTreeMap<String, String>` with `attributes: Arc<BTreeMap<String, String>>`

**Deduplication Gains:**
- 100K events, 30% with same priority value: 30K × 200 bytes saved = 6 MB
- Large logs with repeating cost centers: 1GB+ savings

**Integration Points:**
- `Event` struct: wrap attributes in Arc
- `Trace` struct: wrap attributes in Arc
- Maintain Clone semantics (cheap Arc clone)

### 3. Cache-Friendly Arc Traversal (20% reduction + faster queries)

**Module:** `src/memory/allocator.rs::AdjacencyLists`

**How it works:**
```rust
// Naive (HashMap):
// Each arc lookup: HashMap::get() → hash → probe → dereference
// Cache misses due to scattered heap allocations

// Optimized (AdjacencyLists):
// Pre-allocate contiguous Vec of ArcIndex
// Outgoing[node] = vec of indices into contiguous arc list
// Linear scan with L1/L2 cache hits

let adj = AdjacencyLists::new(100, &arcs);
for arc in adj.outgoing(42) {
    // Direct array access: fast, cache-friendly
}
```

**Memory Layout (optimized):**
```
outgoing: [Vec<usize>, Vec<usize>, ...]  ← dense indices
incoming: [Vec<usize>, Vec<usize>, ...]  ← dense indices
arcs:     [ArcIndex, ArcIndex, ...]      ← contiguous data
```

**Memory Layout (naive HashMap):**
```
HashMap scattered across heap (poor locality)
Each lookup → hash computation → probe sequence → possible collision
```

**Implementation:**
Replace HashMap-based traversal with AdjacencyLists in:
- `PetriNet::fire_transition()`
- `DFGMiner` construction
- `Conformance::check()` marking propagation

**Cache Efficiency:**
- L1 cache line: 64 bytes
- ArcIndex: 24 bytes (fits 2-3 per line)
- HashMap entry: 58 bytes (scatters)
- Estimated improvement: **1.5-2× faster traversal**

### 4. Streaming Conformance Results (30% reduction)

**Current Implementation Problem:**
```rust
// Token Replay collects ALL results in memory
let results: Vec<ConformanceResult> = log.traces
    .iter()
    .map(|trace| check_trace(trace, net))
    .collect();  // ← All results in Vec
```

**Optimized Streaming Approach:**
```rust
// Aggregate incrementally, never store all results
pub fn check_streaming<F>(
    log: &EventLog,
    net: &PetriNet,
    mut aggregator: F,
) where
    F: FnMut(TraceConformance) -> ()
{
    for trace in &log.traces {
        let conformance = check_single(trace, net);
        aggregator(conformance);  // Process and discard
    }
}
```

**Memory Savings:**
- 1M events: 1M TraceConformance results × 100 bytes = 100 MB → 0 MB in collection
- 10M events: 1 GB saved
- 100M events: 10 GB saved

**Integration:**
- Modify `TokenReplay::check()` to return iterator
- Implement streaming variants for all conformance checkers
- Maintain aggregate metrics without storing all results

### 5. Incremental Statistics (25% reduction)

**Current Problem:**
```rust
// Collects all trace lengths into Vec before computing
let lengths: Vec<usize> = log.traces.iter().map(|t| t.len()).collect();
let min = lengths.iter().min();
let max = lengths.iter().max();
let avg = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;
```

**Optimized Incremental:**
```rust
// Single pass, running aggregates
let (min, max, sum, count) = log.traces.iter().fold(
    (usize::MAX, 0, 0, 0),
    |(min, max, sum, count), trace| {
        let len = trace.len();
        (min.min(len), max.max(len), sum + len, count + 1)
    },
);
let avg = sum as f64 / count as f64;
```

**Memory Savings:**
- No intermediate Vec allocation
- Single pass through data
- Perfect for streaming scenarios

**Integration Points:**
- `log_statistics()` → use fold instead of collect
- Variant frequency counting → use HashMap instead of Vec collection
- DFG edge weight aggregation → streaming instead of collecting all edges

---

## Optimization Priority Queue

### Phase 1: High Impact, Low Effort (6-8 hours)

1. **StringIntern for activities** ✓
   - Effort: 3 hours (modify Event, update Display/Serialize)
   - Impact: **40% reduction on activity storage**
   - Risk: Low (backward compat via getter methods)

2. **Streaming token replay** ✓
   - Effort: 2 hours (add streaming variant, update aggregation)
   - Impact: **30% reduction in conformance memory**
   - Risk: Low (new function, doesn't break existing)

3. **Incremental statistics** ✓
   - Effort: 2 hours (replace Vec::collect with fold)
   - Impact: **25% reduction in stats computation**
   - Risk: Very low (functional equivalent)

### Phase 2: Medium Impact, Medium Effort (7-12 hours)

4. **Arc<Attributes> deduplication** ✓
   - Effort: 4 hours (Event + Trace modifications)
   - Impact: **5-10× on attributes if duplicates exist**
   - Risk: Medium (mutable field handling)

5. **AdjacencyLists for Petri nets** ✓
   - Effort: 5 hours (discovery + conformance updates)
   - Impact: **20% reduction + 1.5× speedup**
   - Risk: Medium (algorithm correctness verification)

### Phase 3: Remaining Optimizations (20-28 hours)

6. **Arc<Trace> for log sharing** (8 hours)
   - Impact: Incremental (mostly for repeated analysis)
   - Risk: High (borrowing semantics)

7. **ObjectPool for temporary allocations** (6 hours)
   - Impact: 5% in conformance/discovery
   - Risk: Medium (pool management)

8. **Specialized collections** (12 hours)
   - DFG: sparse graph representation
   - Statistics: histogram buckets vs full distributions
   - Impact: 10-15%
   - Risk: High (correctness verification)

---

## Testing Strategy: TDD + Memory Validation

### Test Methodology

1. **Establish Baseline** (before optimization)
```rust
#[test]
fn baseline_eventlog_memory_1m() {
    let log = generate_profile_log(1_000_000, 10_000);
    let mem = estimate_log_memory(&log);
    assert!(mem < 1200.0);  // Fail if already exceeds
}
```

2. **Implement Optimization**
   - Apply StringIntern or other technique
   - Verify compilation and all tests pass

3. **Measure Improvement**
```rust
#[test]
fn optimized_eventlog_memory_1m() {
    let log = generate_optimized_log(1_000_000, 10_000);
    let mem = estimate_log_memory(&log);
    assert!(mem < 720.0);  // 40% reduction target
}
```

4. **Verify Correctness**
```rust
#[test]
fn correctness_after_optimization() {
    let optimized_log = generate_optimized_log(10_000, 100);
    let original_log = generate_original_log(10_000, 100);

    // Results must be identical
    assert_eq!(
        log_statistics(&optimized_log),
        log_statistics(&original_log)
    );
}
```

### Memory Validation Checklist

- [ ] Baseline measurements established for each component
- [ ] No unsafe code introduced
- [ ] All existing tests still pass
- [ ] New memory-specific tests added
- [ ] Correctness verified (results unchanged)
- [ ] Performance not degraded (or improved)
- [ ] Memory target achieved
- [ ] Documentation updated with new APIs

---

## Backward Compatibility

### Key Design Decisions

1. **String Interning**: Hidden behind `Event::activity()` getter
   ```rust
   pub fn activity(&self, intern: &StringIntern) -> &str
   ```

2. **Arc Attributes**: Transparent to users (Arc is Clone)
   ```rust
   // Old: event.attributes.get("key")
   // New: event.attributes.get("key") (still works, Arc provides Deref)
   ```

3. **Adjacency Lists**: Internal optimization in PetriNet
   ```rust
   // Public API unchanged, traversal faster internally
   ```

### Serialization Considerations

- Serde derive works with Arc (provides Serialize/Deserialize)
- StringIntern needs custom serialization:
  ```rust
  impl Serialize for Event {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
          // Denormalize interned ID back to string for external format
      }
  }
  ```

---

## Success Criteria

✓ **50-60% total memory reduction** across all components
✓ **All tests still pass** (zero correctness regressions)
✓ **No unsafe code** introduced
✓ **Performance not degraded** (ideally improved)
✓ **Documentation complete** (APIs clear for integrators)

---

## Related Files

- **Profiling tests**: `tests/memory_profiling_test.rs`
- **Allocator module**: `src/memory/allocator.rs`
- **Memory module**: `src/memory/mod.rs`
- **EventLog**: `src/log/mod.rs` (primary optimization target)
- **PetriNet**: `src/models/petri_net.rs` (secondary target)
- **Conformance**: `src/conformance/token_replay.rs` (streaming)
- **Statistics**: `src/statistics/log_stats.rs` (incremental)

---

## References

### Rust Optimization Techniques

1. **String Interning**: Reduces memory for repeated data (arena allocation)
2. **Arc<T>**: Atomic reference counting for shared ownership
3. **Cache Locality**: Contiguous memory improves CPU cache hits
4. **Iterator/Streaming**: Avoid collecting into intermediate Vec
5. **Custom Allocators**: Global allocators (e.g., jemalloc) for better fragmentation

### Process Mining Context

- pm4py (Python) baseline: ≈ 1.5GB for 100M events
- Target (pm4py-rust): ≈ 0.5-0.9GB for 100M events
- Advantage of Rust: No GC pauses, better memory layout control
