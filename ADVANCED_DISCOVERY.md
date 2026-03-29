# Advanced Process Discovery Algorithms

This document describes the advanced process discovery algorithms implemented in pm4py-rust: ILP Miner, Split Miner, and Causal Net Mining.

## Architecture Overview

```
┌─────────────────────────────────────────┐
│   Event Log (EventLog)                   │
└────────────┬────────────────────────────┘
             │
    ┌────────┴────────────────────────────┐
    │                                      │
    v                                      v
┌─────────────────┐           ┌──────────────────────┐
│ ILP Miner       │           │ Split Miner          │
│ (ilp_miner.rs)  │           │ (split_miner.rs)     │
│                 │           │                      │
│ • Formulates    │           │ • Detects parallel   │
│   marking       │           │   structures         │
│   equations     │           │ • Splits/joins       │
│ • Solves ILP    │           │ • DFG filtering      │
│ • Minimal nets  │           │                      │
└────────┬────────┘           └──────────┬───────────┘
         │                               │
         v                               v
     Petri Net                       Petri Net
  (with minimal                  (with parallel
   structure)                      structure)


    ┌──────────────────────────────────────┐
    │ Causal Net Miner                     │
    │ (causal_net_miner.rs)                │
    │                                      │
    │ • Extracts causal relations          │
    │ • Identifies parallelism             │
    │ • Extracts I/O sets                  │
    └──────────────────┬───────────────────┘
                       │
                       v
                   Causal Net
                (alternative model)
```

## 1. ILP Miner (src/discovery/ilp_miner.rs)

### Algorithm Overview

The ILP Miner formulates Petri net discovery as an Integer Linear Programming problem. It aims to find the **minimal** Petri net that reproduces the behavior in the event log.

### Formulation

**Variables:**
- Place variables: binary or continuous variables for each potential place
- Arc variables: binary variables indicating presence of arcs

**Constraints:**
- **Marking equations**: Place balance equations ensuring trace replay
- **Activity coverage**: All activities must be covered
- **Arc constraints**: Non-negative flow constraints
- **Size constraints**: Limit number of places (optional)

**Objective:**
```
minimize: Σ (number of places) + Σ (number of arcs)
```

### Implementation Details

The current implementation uses a **simplified ILP-inspired approach** with greedy optimization because full ILP solving on large logs is computationally intensive. Features include:

1. **Causal Relation Extraction**
   - Identifies a→b where a→b exists and b→a doesn't
   - Uses directly-follows analysis

2. **Concurrency Detection**
   - Identifies a||b where both a→b and b→a exist
   - Creates synchronization places for concurrent transitions

3. **Place Optimization**
   - Creates minimal set of places
   - One place per directly-follows relation
   - Additional sync places for concurrency

### Usage

```rust
use pm4py::discovery::ILPMiner;

let miner = ILPMiner::new()
    .with_decomposition(true)
    .with_min_coverage(0.95);

let net = miner.discover(&log);
```

### Configuration

- `use_decomposition`: Apply decomposition for large logs (default: false)
- `min_trace_coverage`: Minimum fraction of traces to cover (default: 0.95)
- `max_places`: Maximum places in result (default: 100)

### Strengths

✓ Produces minimal/optimal nets
✓ Handles complex control flow
✓ Mathematically grounded

### Limitations

✗ Computationally intensive for very large logs
✗ Simplified greedy approach used in practice
✗ Assumes trace fits in memory

---

## 2. Split Miner (src/discovery/split_miner.rs)

### Algorithm Overview

Split Miner discovers Petri nets with **explicit parallel structure** by detecting concurrent and sequential splits/joins in process behavior. It's particularly effective for parallel-heavy processes.

### Key Concepts

**DFG Filtering**: Uses a completeness parameter (0.0-1.0) to filter directly-follows graph:
```
Target edges = ceil(|DFG| × completeness)
Keep highest-frequency edges
```

**Split Detection**: Identifies points where control flow branches:
- **AND split** (concurrent): Multiple paths execute in parallel
- **XOR split** (exclusive): Exactly one path executes

**Join Detection**: Identifies where multiple paths converge

### Algorithm Steps

1. **Build DFG** from event log
2. **Filter DFG** based on completeness parameter
3. **Create transitions** for all activities
4. **Detect splits/joins**:
   - Analyze followers of each activity
   - Check if followers are concurrent (both directions in DFG)
   - Create appropriate place structures
5. **Add structure**:
   - AND split: shared synchronization place
   - XOR split: separate places per branch
   - Sequential: simple connecting places

### Usage

```rust
use pm4py::discovery::SplitMiner;

let miner = SplitMiner::new()
    .with_completeness(0.75)  // Keep top 75% of edges
    .with_parallelism_detection(true);

let net = miner.discover(&log);
```

### Configuration

- `dfg_completeness`: Fraction of DFG edges to keep (0.0-1.0, default: 0.65)
- `detect_parallelism`: Enable parallel structure detection (default: true)
- `min_edge_frequency`: Minimum edge frequency to keep (default: 1)

### Completeness Parameter Effect

```
Completeness = 0.3  → Keeps only most frequent behavior
Completeness = 0.65 → Balanced (default)
Completeness = 0.95 → Keeps nearly all behavior
```

### Strengths

✓ Explicitly models parallelism
✓ Handles noise well (via filtering)
✓ Fast discovery
✓ Good for highly parallel processes

### Limitations

✗ May lose infrequent paths
✗ Completeness parameter needs tuning
✗ Assumes parallelism is intentional

---

## 3. Causal Net Mining (src/discovery/causal_net_miner.rs)

### Model: Causal Net (src/models/causal_net.rs)

A **Causal Net** is an alternative process model to Petri nets. Instead of places and transitions, it uses:

**Activities**: Process actions (nodes)

**Relations**: Three types of causal relations:
- **→ (Causality)**: Strict precedence (a→b means b can only happen after a)
- **|| (Parallelism)**: Concurrent execution allowed
- **# (Conflict/Choice)**: Mutually exclusive alternatives

**I/O Sets**: For each activity, define:
- **Input set**: Activities that can precede it
- **Output set**: Activities that can follow it

### Algorithm Overview

The Causal Net Miner discovers these relations from an event log.

### Algorithm Steps

1. **Extract directly-follows** relations from log
2. **Identify causal relations**: a→b if a→b exists and b→a doesn't
3. **Identify parallel relations**: a||b if both a→b and b→a exist frequently
4. **Identify conflict relations**: Activities with multiple exclusive successors
5. **Build I/O sets**: Group inputs/outputs by activity
6. **Apply support filtering**: Remove low-frequency relations

### Usage

```rust
use pm4py::discovery::CausalNetMiner;

let miner = CausalNetMiner::new()
    .with_min_support(0.1)  // 10% minimum support
    .with_self_loops(false);

let net = miner.discover(&log);

// Accept trace
let trace = vec!["A".to_string(), "B".to_string(), "C".to_string()];
println!("Accepts trace: {}", net.accepts_trace(&trace));

// Get relations
let causals = net.get_relations_by_type(CausalRelation::Causality);
let parallels = net.get_relations_by_type(CausalRelation::Parallel);
```

### Configuration

- `min_support`: Minimum fraction of traces for relation (0.0-1.0, default: 0.0)
- `allow_self_loops`: Allow activities to lead to themselves (default: false)
- `max_input_set_size`: Maximum inputs per activity (default: 10)

### Trace Acceptance

A Causal Net accepts a trace if:
1. First activity is a start activity
2. Last activity is an end activity
3. All intermediate activities have valid directly-follows relations
4. No activities not in the net

### Strengths

✓ Simpler than Petri nets (fewer constructs)
✓ Explicitly shows causality and parallelism
✓ Good for understanding control flow
✓ Fast acceptance testing
✓ Natural noise handling via support filtering

### Limitations

✗ Less expressive than Petri nets
✗ Cannot express complex guards/conditions
✗ Cannot handle weighted arcs

---

## Comparison Matrix

| Feature | ILP Miner | Split Miner | Causal Net |
|---------|-----------|-------------|-----------|
| **Model** | Petri Net | Petri Net | Causal Net |
| **Optimality** | Optimal (minimal) | Near-optimal | Best effort |
| **Parallelism** | Implicit | Explicit | Explicit |
| **Noise Handling** | Poor | Good (DFG filter) | Good (support filter) |
| **Speed** | Slow | Fast | Fast |
| **Complexity** | High | Medium | Low |
| **Expressiveness** | Very high | High | Medium |
| **Best For** | Academic work, verification | Real processes with parallelism | Understanding control flow |

---

## Implementation Notes

### ILP Miner - Mathematical Formulation

For a trace σ = [a₁, a₂, ..., aₙ], the marking equation is:

```
m(p, σ) = m₀(p) + Σᵢ₌₁ⁿ (t(aᵢ) → p) - (p → t(aᵢ))
```

Where:
- m(p, σ) = marking of place p after trace σ
- m₀(p) = initial marking of place p
- t(a) = transition for activity a
- The sum represents arc weights

### Split Miner - Concurrency Detection

Two activities are concurrent if both orderings appear in the log:
```
a → b appears k times
b → a appears m times
```

If both k > 0 and m > 0 (with possible support threshold), they're concurrent.

### Causal Net - I/O Sets

The I/O sets capture the "signature" of each activity:

```
Activity A:
  Input:  [{B, C}, {D}]  meaning (B AND C) OR (D)
  Output: [{E, F}]       meaning (E AND F)
```

This allows checking both necessary preconditions and guaranteed consequences.

---

## Testing

Each algorithm includes comprehensive tests:

```bash
# Run ILP Miner tests
cargo test discovery::ilp_miner

# Run Split Miner tests
cargo test discovery::split_miner

# Run Causal Net Miner tests
cargo test discovery::causal_net_miner

# Run Causal Net model tests
cargo test models::causal_net
```

### Test Coverage

- **Creation and configuration**
- **Simple sequential discovery**
- **Concurrent/parallel detection**
- **Empty log handling**
- **Trace acceptance**
- **Parameter variations**
- **Edge cases**

---

## Performance Characteristics

### Time Complexity

| Algorithm | Best Case | Average | Worst Case |
|-----------|-----------|---------|-----------|
| **ILP Miner** | O(\|A\|²) | O(\|A\|²·\|σ\|) | O(2^(\|A\|)) |
| **Split Miner** | O(\|A\|² + \|σ\|) | O(\|A\|² + \|σ\|·\|A\|) | O(\|A\|³) |
| **Causal Net** | O(\|A\|² + \|σ\|) | O(\|A\|² + \|σ\|·\|A\|) | O(\|A\|³) |

Where:
- \|A\| = number of activities
- \|σ\| = total events in log
- \|T\| = number of traces

### Space Complexity

| Algorithm | Space |
|-----------|-------|
| **ILP Miner** | O(\|A\|² + \|P\|) |
| **Split Miner** | O(\|A\|² + \|P\|) |
| **Causal Net** | O(\|A\|² + \|IO\|) |

Where:
- \|P\| = number of places
- \|IO\| = total I/O set size

---

## Future Improvements

1. **ILP Miner**
   - Integrate actual LP solver (good_lp or coin_cbc)
   - Implement decomposition for large logs
   - Add flexible markin constraints

2. **Split Miner**
   - Automatic completeness parameter tuning
   - Multi-level decomposition
   - Weighted edge analysis

3. **Causal Net**
   - Convert Causal Net to Petri net
   - Add guards and conditions
   - Weighted relation support

---

## References

- ILP Miner: "Discovering Blocks from Event Logs" (van der Aalst et al., 2014)
- Split Miner: "Splitting Miner: Discovering Concurrency and Ordering" (Leemans et al., 2013)
- Causal Nets: "Causal Nets as a Modeling Language" (van der Aalst, 1996)
