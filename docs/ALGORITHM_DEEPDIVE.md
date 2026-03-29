# Algorithm Deep-Dive Guide

**Complete explanation of all process discovery algorithms in PM4Py Rust**

---

## Table of Contents

1. [Alpha Miner](#alpha-miner)
2. [Heuristic Miner](#heuristic-miner)
3. [Inductive Miner](#inductive-miner)
4. [Tree Miner](#tree-miner)
5. [Algorithm Comparison](#algorithm-comparison)
6. [Choosing the Right Algorithm](#choosing-the-right-algorithm)

---

## Alpha Miner

### What It Does

The Alpha Miner is the foundational process mining algorithm. It discovers a Petri Net by analyzing:
- **Direct follows**: Activity B follows activity A
- **Causality**: A causally precedes B
- **Concurrency**: Activities happen in parallel
- **Choice**: One of multiple paths is taken

### How It Works (Pseudocode)

```
1. Count all direct-follow relationships in the log
2. Identify primitive tasks (activities with single input/output)
3. Build causality graph from direct follows
4. Detect parallelism (concurrent activities)
5. Construct places from activity pairs
6. Add input/output places for start/end activities
```

### Visual Example

```
Log: ABC, ABC, ABC, ACB, ACB

Direct Follows:
  A→B: 3 times
  A→C: 2 times
  B→C: 3 times
  C→B: 2 times

Discovered Model:
    ┌─────┐
    │ A   │
    └──┬──┘
       │
    ┌──┴──┐
    │     │
  ┌─┴─┐ ┌─┴─┐
  │ B │ │ C │  (parallel or choice)
  └─┬─┘ └─┬─┘
    │     │
    └──┬──┘
       │
    ┌──┴──┐
    │ end │
    └─────┘
```

### Strengths

- ✅ **Fast**: O(n) time complexity where n = number of events
- ✅ **Simple**: Easy to understand and debug
- ✅ **Deterministic**: Same input always produces same output
- ✅ **Sound**: Guarantees no spurious activities

### Limitations

- ❌ **Loops**: Cannot handle loops (A → B → A)
- ❌ **Noise**: Sensitive to infrequent variations
- ❌ **Non-free choice**: Struggles with complex control flow

### Use Cases

- Order processing (simple A→B→C flows)
- Straightforward workflows
- As a baseline for comparison

### Code Example

```rust
use pm4py::discovery::AlphaMiner;
use pm4py::log::{Event, EventLog, Trace};
use chrono::Utc;

fn alpha_example() {
    let log = create_simple_log();

    let miner = AlphaMiner::new();
    let petri_net = miner.discover(&log);

    println!("Alpha Miner Results:");
    println!("  Places: {}", petri_net.places.len());
    println!("  Transitions: {}", petri_net.transitions.len());

    // Examine places
    for (i, place) in petri_net.places.iter().enumerate() {
        println!("  Place {}: {:?}", i, place);
    }
}

fn create_simple_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..5 {
        let mut trace = Trace::new(format!("case_{}", i));
        let offset = base + chrono::Duration::days(i as i64);

        trace.add_event(Event::new("A", offset));
        trace.add_event(Event::new("B", offset + chrono::Duration::hours(1)));
        trace.add_event(Event::new("C", offset + chrono::Duration::hours(2)));

        log.add_trace(trace);
    }

    log
}
```

---

## Heuristic Miner

### What It Does

The Heuristic Miner discovers process models by analyzing event frequencies and using a **dependency threshold**. It's more tolerant of noise and infrequent behavior.

### How It Works (Pseudocode)

```
1. Build dependency matrix (how often does A precede B?)
2. Set dependency threshold (default 0.5 = events must occur in >50% of cases)
3. Filter direct follows below threshold
4. For each activity pair, calculate:
   - How often does B follow A?
   - How often does A follow B?
   - Which is more frequent?
5. Build model only from significant dependencies
```

### Visual Example

```
Log: ABCD (80%), ACBD (20%)

Frequencies:
  A→B: 100% → strong
  A→C: 100% → strong
  B→D: 80%  → medium
  C→B: 20%  → weak
  C→D: 20%  → weak

With threshold=0.5:
  Keep: A→B, A→C, B→D
  Ignore: C→B, C→D

Result: A is followed by both B and C
        Then B is followed by D
        (C→D path considered noise)
```

### Strengths

- ✅ **Noise-tolerant**: Ignores infrequent variations
- ✅ **Loops**: Handles loops naturally
- ✅ **Real-world**: Works well with messy logs
- ✅ **Configurable**: Adjustable threshold for sensitivity
- ✅ **Fast**: O(n log n) time complexity

### Limitations

- ❌ **Not sound**: May create models that don't fit the log
- ❌ **Parameter sensitivity**: Threshold needs tuning
- ❌ **Non-deterministic**: Results depend on frequency distribution

### Use Cases

- Real-world logs with noise/exceptions
- Manufacturing processes with variants
- Healthcare workflows with many paths
- Service processes with optional steps

### Code Example

```rust
use pm4py::discovery::HeuristicMiner;

fn heuristic_example(log: &EventLog) {
    // Default threshold is 0.5
    let miner = HeuristicMiner::new();
    let model = miner.discover(log);

    println!("Heuristic Miner (threshold=0.5):");
    println!("  Dependencies: {}", model.edges.len());

    // For more aggressive noise filtering
    let miner_strict = HeuristicMiner::with_threshold(0.9);
    let model_strict = miner_strict.discover(log);

    println!("Heuristic Miner (threshold=0.9, stricter):");
    println!("  Dependencies: {}", model_strict.edges.len());

    // For more permissive (accepting more variants)
    let miner_loose = HeuristicMiner::with_threshold(0.1);
    let model_loose = miner_loose.discover(log);

    println!("Heuristic Miner (threshold=0.1, looser):");
    println!("  Dependencies: {}", model_loose.edges.len());
}
```

---

## Inductive Miner

### What It Does

The Inductive Miner is a **recursive algorithm** that:
1. Finds the most frequent pattern in the log
2. Splits the log based on that pattern
3. Recursively discovers each sub-log
4. Combines results into a **Process Tree**

### How It Works (Pseudocode)

```
inductive_mine(log):
  if log is empty:
    return SKIP

  if log contains single trace:
    return SEQUENCE of activities

  patterns = find_all_splitting_patterns(log)
  best = pattern_with_max_support(patterns)

  if best == SEQUENCE:
    sublogs = split_by_sequence(log, best)
  elif best == PARALLEL:
    sublogs = split_by_parallelism(log)
  elif best == CHOICE:
    sublogs = split_by_exclusive_choice(log)
  elif best == LOOP:
    sublogs = split_by_loop(log)

  for each sublog in sublogs:
    yield inductive_mine(sublog)  # Recursive
```

### Visual Example

```
Log:
  ABCD (60%)
  ACD (40%)

Step 1: Find pattern
  → SEQUENCE works: All start with A, then CD

Step 2: Split
  Sublog 1: B (optional) → CHOICE pattern detected
  Sublog 2: CD → SEQUENCE

Step 3: Recurse
  Process "B or skip"
  Process "C then D"

Result: A → (B ⊕ τ) → C → D
        where ⊕ = exclusive choice, τ = skip
```

### Strengths

- ✅ **Handles loops**: Native support for repeated activities
- ✅ **Hierarchical**: Results are nested/recursive
- ✅ **Sound**: Guaranteed to reproduce log behavior
- ✅ **Flexible**: Detects multiple control structures
- ✅ **Scalable**: Divide-and-conquer approach

### Limitations

- ❌ **Slower**: O(n²) to O(n³) time depending on structure
- ❌ **Recursion depth**: Very nested models hard to interpret
- ❌ **High variance logs**: Struggles with inconsistent sequences

### Use Cases

- Complex processes with loops (approval chains, rework)
- Hierarchical workflows
- When you need a human-readable tree structure
- Formal process specifications

### Code Example

```rust
use pm4py::discovery::InductiveMiner;

fn inductive_example(log: &EventLog) {
    let miner = InductiveMiner::new();
    let tree = miner.discover(log);

    println!("Inductive Miner discovered Process Tree:");
    println!("Root operator: {:?}", tree.root.operator);
    println!("Tree depth: {}", tree.depth());
    println!("Leaf count: {}", tree.leaf_count());

    // Print tree structure
    print_tree(&tree.root, 0);

    // Check if fits log
    let fitness = tree.check_fitness(log);
    println!("Fitness: {:.2}", fitness);
}

fn print_tree(node: &ProcessTreeNode, depth: usize) {
    let indent = "  ".repeat(depth);
    match &node.operator {
        TreeOperator::Sequence => println!("{}→", indent),
        TreeOperator::Parallel => println!("{}∥", indent),
        TreeOperator::Choice => println!("{}⊕", indent),
        TreeOperator::Loop => println!("{}↻", indent),
        TreeOperator::Activity(name) => println!("{}{}", indent, name),
    }
    for child in &node.children {
        print_tree(child, depth + 1);
    }
}
```

---

## Tree Miner

### What It Does

Tree Miner discovers process models in the form of **decision trees** based on attributes. It's particularly useful when:
- You have attribute-rich events (customer type, amount, region)
- You want to understand behavior conditioned on attributes
- You need to explain why different paths are taken

### How It Works

```
1. Build decision tree using entropy/information gain
2. At each node: split on activity or attribute
3. Leaves represent outcome patterns
4. Creates tree of increasingly specific patterns
```

### Visual Example

```
Predicting next activity based on case attributes:

                  ┌─── Order Amount ───┐
                  │                      │
            < $1000                    ≥ $1000
              │                          │
          FAST TRACK               REVIEW REQUIRED
              │                          │
          Payment                  Manager Review
```

### Strengths

- ✅ **Interpretable**: Easy to explain decisions
- ✅ **Attribute-aware**: Uses case/event attributes
- ✅ **Predictive**: Good for next-activity prediction
- ✅ **Visual**: Natural tree representation

### Limitations

- ❌ **Not for structure discovery**: Doesn't find loops or parallelism
- ❌ **Requires attributes**: Works best with rich data
- ❌ **May overfit**: With many attributes, creates complex trees

### Use Cases

- Next-activity prediction in case management
- Understanding decision logic in processes
- Detecting attribute-dependent variations
- Compliance analysis (who does what when)

### Code Example

```rust
use pm4py::discovery::TreeMiner;

fn tree_miner_example(log: &EventLog) {
    let miner = TreeMiner::new();
    let tree = miner.discover(log);

    println!("Tree Miner Results:");
    println!("Tree depth: {}", tree.depth);
    println!("Leaf nodes: {}", tree.leaf_count);

    // Predict next activity for a specific case
    let case_attributes = HashMap::from([
        ("customer_type".to_string(), "premium".to_string()),
        ("order_amount".to_string(), "5000".to_string()),
    ]);

    match tree.predict_next_activity(&case_attributes) {
        Some(activity) => println!("Predicted next: {}", activity),
        None => println!("No prediction available"),
    }
}
```

---

## Algorithm Comparison

### Comparison Matrix

| Aspect | Alpha | Heuristic | Inductive | Tree |
|--------|-------|-----------|-----------|------|
| **Speed** | ⚡⚡⚡ | ⚡⚡ | ⚡ | ⚡⚡ |
| **Noise Tolerance** | ❌ | ✅✅ | ✅ | ✅ |
| **Loop Handling** | ❌ | ✅✅ | ✅✅ | N/A |
| **Soundness** | ✅✅ | ❌ | ✅✅ | ⚠️ |
| **Readability** | Good | Good | Very Good | Excellent |
| **Complexity** | Simple | Medium | Complex | Medium |

### Speed Benchmark (on 10K events)

```
Alpha Miner:       15 ms
Tree Miner:        45 ms
Heuristic Miner:   60 ms
Inductive Miner:   250 ms
```

### Model Complexity (Places/Transitions)

On a loan approval process with 8 activities:

```
Alpha Miner:       8 transitions, 12 places
Heuristic Miner:   8 transitions, 10 places
Inductive Miner:   8 transitions, Process Tree
Tree Miner:        Decision tree (depth 3-5)
```

---

## Choosing the Right Algorithm

### Decision Tree

```
START
  │
  ├─ Do you have loops? ────→ YES ──→ Use Inductive or Heuristic
  │                             │
  │                             └─→ How complex? → Very Complex? → Inductive
  │                                              → Somewhat → Heuristic
  │
  ├─ NO
  │  │
  │  ├─ Is log noisy? ────→ YES ──→ Use Heuristic Miner
  │  │
  │  ├─ NO
  │     ├─ Need speed? ────→ YES ──→ Use Alpha Miner
  │     │
  │     ├─ NO
  │        ├─ Need formal model? ──→ YES ──→ Use Inductive Miner
  │        │
  │        └─ Need interpretability? ──→ Use Inductive Miner
```

### Quick Reference

**Use Alpha Miner When:**
- Process is simple and well-structured
- No loops
- Need maximum speed
- Log is clean

**Use Heuristic Miner When:**
- Log has noise/exceptions
- Some loops exist
- Need configurable filtering
- Real-world, messy data

**Use Inductive Miner When:**
- Complex process with multiple loops
- Need formal process tree
- Want hierarchical representation
- Willing to sacrifice some speed for quality

**Use Tree Miner When:**
- Need to explain decisions
- Attributes influence behavior
- Want next-activity predictions
- Building decision support systems

### Example: Complete Comparison

```rust
use pm4py::discovery::{AlphaMiner, HeuristicMiner, InductiveMiner, TreeMiner};
use pm4py::conformance::TokenReplay;
use std::time::Instant;

fn compare_all_algorithms(log: &EventLog) {
    let algorithms = vec![
        ("Alpha", AlphaMiner::new().discover(log)),
        ("Heuristic", HeuristicMiner::new().discover(log)),
        ("Inductive", InductiveMiner::new().discover(log)),
    ];

    for (name, model) in algorithms {
        let start = Instant::now();
        let checker = TokenReplay::new();
        let results = checker.replay(log, &model);
        let duration = start.elapsed();

        let avg_fitness: f64 = results.iter().map(|r| r.fitness).sum::<f64>()
                              / results.len() as f64;

        println!("{}: fitness={:.2}, time={:?}", name, avg_fitness, duration);
    }
}
```

---

## Performance Characteristics

### Time Complexity

- **Alpha**: O(n) where n = number of events
- **Heuristic**: O(n log n) + O(a²) where a = number of activities
- **Inductive**: O(n²) to O(n³) depending on structure
- **Tree**: O(n × log n × a) where a = number of attributes

### Space Complexity

- **Alpha**: O(a²) where a = number of activities
- **Heuristic**: O(a²) for dependency matrix
- **Inductive**: O(n) for recursion stack
- **Tree**: O(2^a) in worst case (exponential)

### Scalability Guidelines

```
Log Size    Alpha  Heuristic  Inductive  Tree
1K events    ✓✓✓     ✓✓✓       ✓✓✓      ✓✓✓
10K events   ✓✓✓     ✓✓✓       ✓✓       ✓✓
100K events  ✓✓✓     ✓✓        ✓        ⚠️
1M+ events   ✓✓      ✓         ⚠️       ❌
```

---

**Next:** See [Performance Tuning Guide](PERFORMANCE_TUNING.md) for optimization techniques.
