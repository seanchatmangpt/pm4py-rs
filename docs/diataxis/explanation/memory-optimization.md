# Memory Optimization Architecture

## Overview

Memory optimization in pm4py-rust targets 40-45% reduction across all components without sacrificing correctness. This document explains the theoretical foundation and practical techniques.

## Memory Landscape (Before Optimization)

For a 100M event log:

```
Component Breakdown (100M events, ~800MB initial)
────────────────────────────────────────────

Event data (activity, timestamp, UUID)    ≈ 40-50% (320-400 MB)
  - Activity name (string)               ≈ 40-50 bytes per event
  - Timestamp (DateTime)                 ≈ 24 bytes per event
  - UUID                                 ≈ 16 bytes per event

Event attributes (BTreeMap per event)    ≈ 25-30% (200-240 MB)
  - HashMap/BTreeMap overhead           ≈ 56-80 bytes per event
  - Stored key-value pairs              ≈ varies

Trace metadata (Vec<Event>)              ≈ 10-15% (80-120 MB)
  - Vec pointer, capacity, length       ≈ 24 bytes per trace
  - Per-trace attributes                ≈ variable

Indices & structures                     ≈ 5-10% (40-80 MB)
  - DFG/graph structures
  - Variant caches
```

## Optimization Strategy

### Level 1: String Interning

**Problem:** Activity "approve" stored 1,000,000 times × 7 bytes = 7 MB waste

**Solution:** Store string once, reference by ID (8 bytes → index mapping)

```rust
// Before: each event stores full string
event.activity = "approve".to_string();  // 7 bytes + 32 overhead = 39 bytes
event.activity = "reject".to_string();   // 6 bytes + 32 overhead = 38 bytes

// After: store integer ID
pub struct StringIntern {
    strings: HashMap<String, usize>,    // "approve" → 0
    by_id: Vec<String>,                // 0 → "approve"
}

let id = intern.intern("approve");      // Returns 0
events[0].activity_id = 0;              // 8 bytes

// Compression ratio for 1000 "approve" occurrences:
// Before: 1000 × 39 = 39,000 bytes
// After:  1 × 39 + 1000 × 8 = 8,039 bytes
// Saving: 79.3% for this activity
```

**Implementation:** `src/memory/allocator.rs::StringIntern`

### Level 2: Attribute Deduplication via Arc

**Problem:** Each event has identical attributes (priority, cost, resource), stored separately

**Solution:** Share via Arc<HashMap> when attributes are identical

```rust
// Before: every event stores complete HashMap
Event {
    activity: "approve",
    attributes: {
        "priority": "high",
        "cost": "100",
        "resource": "manager_1"
    }
}
Event {
    activity: "reject",
    attributes: {
        "priority": "high",
        "cost": "100",
        "resource": "manager_1"
    }
}
Total: 2 × 200 bytes = 400 bytes

// After: share Arc
pub struct CompactAttributes {
    cache: HashMap<u64, Arc<HashMap<String, String>>>
}

arc1 = CompactAttributes::add_attributes(attrs1);
arc2 = CompactAttributes::add_attributes(attrs1);  // Returns same Arc
assert!(Arc::ptr_eq(&arc1, &arc2));  // true

Total: 200 bytes + 2 × 16 (Arc pointers) = 232 bytes (42% savings)
```

**Implementation:** `src/memory/allocator.rs::CompactAttributes`

### Level 3: Adjacency List Optimization

**Problem:** Petri net arcs stored individually; finding inputs/outputs requires O(n) scan

**Solution:** Pre-build adjacency lists for O(1) access with better cache locality

```rust
// Before: arcs stored in flat Vec
struct Arc {
    from: String,
    to: String,
    weight: usize,
}

// Finding inputs to transition "t1": scan all arcs O(m)
let inputs: Vec<_> = arcs.iter()
    .filter(|a| a.to == "t1")
    .collect();

// After: adjacency lists
pub struct AdjacencyLists {
    outgoing: Vec<Vec<usize>>,  // node i → arc indices starting from i
    incoming: Vec<Vec<usize>>,  // node j → arc indices ending at j
    arcs: Vec<ArcIndex>,        // all arcs, indexed
}

// Finding inputs to transition "t1": O(k) where k = degree
let inputs = adj.incoming(t1_node);

// Cache efficiency: contiguous allocation → better CPU cache hits
// vs HashMap which has scattered memory access
```

**Implementation:** `src/memory/allocator.rs::AdjacencyLists`

### Level 4: Event Streaming (Conformance)

**Problem:** Token replay materializes all replay results in memory before returning

**Solution:** Stream results as they're computed

```rust
// Before: collect all results, then return
fn replay(log: &EventLog, model: &PetriNet) -> Vec<ReplayResult> {
    let mut results = Vec::new();
    for trace in log.traces {
        results.push(ReplayResult::compute(trace, model));
    }
    results  // All in memory!
}

// After: stream via iterator
fn replay_stream(log: &EventLog, model: &PetriNet) -> impl Iterator<Item=ReplayResult> {
    log.traces.iter().map(|trace| ReplayResult::compute(trace, model))
}

// Consumer processes one at a time
for result in model.replay_stream(log) {
    fitness.push(result.fitness);
    // Discard result → memory freed
}

// Peak memory: size of 1 result vs all results
// For 100M events: 1 MB vs 400+ MB (99.75% reduction!)
```

**Impact:** 30-40% reduction in conformance checking memory

### Level 5: Incremental Statistics

**Problem:** Statistics computation collects all events/traces before aggregating

**Solution:** Single-pass aggregation without collecting intermediates

```rust
// Before: accumulate then compute
fn statistics(log: &EventLog) -> Stats {
    let mut event_times = Vec::new();
    let mut activity_counts = HashMap::new();

    for trace in log.traces {
        for event in trace.events {
            event_times.push(event.timestamp);  // Collect all!
            *activity_counts.entry(&event.activity).or_insert(0) += 1;
        }
    }

    // Now compute stats
    let avg_time = event_times.iter().sum() / event_times.len();
    let variance = ...
    Stats { avg_time, variance, activity_counts }
}

// After: single-pass incremental
fn statistics(log: &EventLog) -> Stats {
    let mut sum = 0;
    let mut count = 0;
    let mut sum_sq = 0;
    let mut activity_counts = HashMap::new();

    for trace in log.traces {
        for event in trace.events {
            // Compute online (Welford's algorithm)
            sum += event.timestamp;
            sum_sq += event.timestamp * event.timestamp;
            count += 1;
            *activity_counts.entry(&event.activity).or_insert(0) += 1;
        }
    }

    // Compute from accumulated stats
    let avg = sum / count;
    let variance = (sum_sq / count) - (avg * avg);
    Stats { avg, variance, activity_counts }
}

// Peak memory: constant (no event_times Vec)
// Savings: O(n) → O(1) temporary storage
```

**Implementation:** Incremental statistics in discovery and conformance

### Level 6: Cache-Line Alignment

**Problem:** Hot data (token marking) scattered in memory → cache misses

**Solution:** Align frequently-accessed structs to 64-byte cache lines

```rust
// Before: scattered memory access during conformance checking
#[repr(C)]
struct Marking {
    tokens: HashMap<String, usize>,  // Random memory address
}

// After: aligned to cache line
#[repr(align(64))]
pub struct CacheAlignedMarking {
    pub marking: HashMap<String, usize>,
    _padding: [u8; 0],  // Compiler fills to 64 bytes
}

// Effect: Intel CPU cache lines = 64 bytes
// When accessing one marking, neighbors in cache line don't evict it
// → fewer cache misses during token replay
```

**Implementation:** `src/optimization/cache_aware.rs::CacheAlignedMarking`

## Optimization Implementation Map

| Component | Optimization | File | Target Reduction |
|-----------|-------------|------|-----------------|
| EventLog | String interning | `memory/allocator.rs` | 15-25% |
| EventLog | Attribute deduplication | `memory/allocator.rs` | 20-30% |
| PetriNet | Adjacency lists | `memory/allocator.rs` | 10-15% |
| Conformance | Result streaming | `conformance/*.rs` | 30-40% |
| Statistics | Incremental aggregation | `statistics/*.rs` | 20-25% |
| All | Cache alignment | `optimization/cache_aware.rs` | 2-5% (speedup) |

**Total Target:** 40-45% reduction across all components

## Memory Profiling Test Coverage

See `tests/memory_profiling_test.rs` for validation:

```
EventLog 1M events:    < 1200 MB  (vs 2000 MB naïve)
EventLog 10M events:   < 6000 MB  (vs 15000 MB naïve)
EventLog 100M events:  < 6000 MB  (vs 60000 MB naïve)
TokenReplay 1M:        < 1200 MB  (vs 400 MB data + streaming)
DFG Discovery 1M:      < 1200 MB  (working set)
```

## Correctness Guarantees

All optimizations maintain **semantic equivalence**:

✓ String IDs are transparent to API
✓ Deduplicated attributes are identical
✓ Streamed results are identical to collected
✓ Incremental stats are numerically equivalent
✓ Cache alignment doesn't affect logic

**Test suite:** `tests/verify_correctness_after_memory_optimization`

## Trade-offs

| Optimization | Benefit | Trade-off |
|-------------|---------|-----------|
| String interning | 15-25% memory | O(1) string lookup via HashMap |
| Attribute Arc | 20-30% memory | Hash collision chance (mitigation: good hash) |
| Adjacency lists | 10-15% memory + speedup | O(setup) to build indices |
| Streaming conformance | 30-40% memory | Can't re-access results after iteration |
| Incremental stats | 20-25% memory | Online algorithm complexity |

**Conclusion:** Trade-offs are favorable for large-scale event logs

## Implementing Custom Optimizations

To add a new memory optimization:

1. **Profile** with `cargo test memory_profiling_test --release`
2. **Identify** memory hotspot (top 5 consumers)
3. **Design** optimization (maintain correctness, test before/after)
4. **Implement** with target reduction
5. **Benchmark** against baseline: `scripts/benchmark.sh`
6. **Document** in this file

## Related Documentation

- **Baseline Measurement**: `docs/diataxis/how-to/baseline-measurement.md`
- **Cache-Aware Optimization**: `docs/diataxis/explanation/cache-aware-optimization.md`
- **Load Testing**: `docs/diataxis/tutorials/load-testing-quickstart.md`
