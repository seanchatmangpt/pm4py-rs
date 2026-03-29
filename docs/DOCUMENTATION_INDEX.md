# PM4Py Rust Complete Documentation Index

**Master index to all PM4Py Rust documentation and examples**

Last Updated: 2026-03-24
Version: 0.3.0

---

## Documentation Suite Overview

This suite consists of:
- **5 Comprehensive Guides** (2000+ lines)
- **10 Working Examples** (1500+ lines)
- **2 Index/Reference Documents**

Total: **2000+ documentation lines** + **1500+ example lines** = **3500+ comprehensive content**

---

## Getting Started

### For First-Time Users

**Start here:**
1. [Complete Getting Started Guide](COMPLETE_GETTING_STARTED.md) (500 lines)
   - Installation
   - Your first discovery
   - Your first conformance check
   - Common patterns
   - FAQ & troubleshooting

**Then try:**
2. [Examples Index](EXAMPLES_INDEX.md) - Run examples 01-04 (30 minutes)

**Then read:**
3. [Algorithm Deep-Dive](ALGORITHM_DEEPDIVE.md) - Understand how each algorithm works

---

## Complete Documentation Files

### 1. Complete Getting Started Guide
**File:** `COMPLETE_GETTING_STARTED.md` (500 lines)
**Time to read:** 15 minutes
**Level:** Beginner

**Covers:**
- Installation for Rust 1.70+
- Creating your first project
- First discovery (Alpha Miner)
- First conformance check
- Common patterns & code snippets
- 10+ frequently asked questions
- Troubleshooting section

**Best for:** Developers new to process mining

**Key sections:**
- Installation prerequisites & verification
- Step-by-step project setup
- 5-minute quickstart examples
- When to use each algorithm
- Memory/performance tips

---

### 2. Algorithm Deep-Dive Guide
**File:** `ALGORITHM_DEEPDIVE.md` (800 lines)
**Time to read:** 30 minutes
**Level:** Beginner-Intermediate

**Covers all 4 discovery algorithms:**

1. **Alpha Miner**
   - How it works (pseudocode)
   - Visual example
   - Strengths/limitations
   - Use cases
   - Code example

2. **Heuristic Miner**
   - Frequency-based discovery
   - Threshold tuning
   - Noise handling
   - Real-world use cases

3. **Inductive Miner**
   - Recursive decomposition
   - Handling loops
   - Process trees
   - Complex processes

4. **Tree Miner**
   - Decision tree discovery
   - Attribute-based analysis
   - Prediction capabilities

**Plus:**
- Complete comparison matrix
- Decision tree for choosing algorithm
- Performance characteristics
- Scalability guidelines

**Best for:** Understanding which algorithm to use

---

### 3. Comprehensive Troubleshooting Runbook
**File:** `COMPREHENSIVE_TROUBLESHOOTING.md` (700 lines)
**Time to read:** As needed (reference)
**Level:** All levels

**Organized by problem category:**

1. **Installation Issues** (2 solutions)
   - Cannot find pm4py crate
   - Wrong Rust version

2. **Compilation Errors** (4 solutions)
   - Function not found
   - Type mismatch
   - After-update errors
   - Borrow checker errors

3. **Runtime Errors** (3 solutions)
   - Index out of bounds
   - unwrap() on None
   - Mutable/immutable borrow conflict

4. **Discovery Problems** (3 solutions)
   - Empty/trivial models
   - Model too complex
   - Loops not detected

5. **Conformance Issues** (2 solutions)
   - All fitness 0.0
   - Mixed results

6. **Performance Issues** (3 solutions)
   - Discovery too slow
   - Conformance too slow
   - Out of memory

7. **Data Format Issues** (2 solutions)
   - Invalid XES files
   - CSV import problems

**Plus:** Summary trouble-shooting table

**Best for:** Solving specific problems, quick reference

---

### 4. Performance Tuning Guide
**File:** `PERFORMANCE_TUNING_GUIDE.md` (400 lines)
**Time to read:** 20 minutes
**Level:** Intermediate-Advanced

**Covers:**

1. **Benchmarking Methods**
   - Manual timing
   - Criterion library
   - Perf profiling

2. **Discovery Optimization**
   - Algorithm selection
   - Pre-filtering
   - Sampling
   - Parallelization
   - Caching

3. **Conformance Optimization**
   - Faster methods (DFG vs Token Replay)
   - Sampling strategies
   - Batch processing

4. **Statistics Optimization**
   - Incremental updates
   - Lazy evaluation

5. **Memory Optimization**
   - Stream processing
   - Data structure selection
   - Object pooling
   - Memory profiling

6. **Real-World Scenarios**
   - Real-time processing
   - Multi-tenant systems
   - Batch processing

**Plus:** Optimization checklist and complete example

**Best for:** Large-scale processing, performance optimization

---

### 5. Advanced Topics
**File:** `ADVANCED_TOPICS.md` (600 lines)
**Time to read:** 40 minutes
**Level:** Advanced

**Covers:**

1. **Custom Discovery Algorithms**
   - Creating custom miners
   - ML-driven discovery

2. **Advanced Conformance Checking**
   - Cost-based alignment
   - Pattern-based conformance
   - Custom checkers

3. **Domain-Specific Plugins**
   - Healthcare compliance (HIPAA)
   - Finance fraud detection

4. **Distributed Processing**
   - Map-reduce pattern
   - Parallel discovery

5. **Real-Time Streaming**
   - Event stream processing
   - Windowing

6. **ML Integration**
   - Next-activity prediction
   - Probability calculation

**Best for:** Custom implementations, domain-specific solutions

---

## Examples (10 Working Programs)

### Examples Index
**File:** `EXAMPLES_INDEX.md` (400 lines)
**Contains:** Quick reference + learning paths

All examples run with:
```bash
cargo run --example <name>
```

---

### Discovery Examples

#### Example 01: Simple Alpha Miner
- **File:** `examples/01_simple_alpha_miner.rs`
- **Time:** 5 min
- **Lines:** 60
- **Best for:** First program
- **Output:** Basic model structure

#### Example 02: Heuristic with Filtering
- **File:** `examples/02_heuristic_with_filtering.rs`
- **Time:** 10 min
- **Lines:** 120
- **Best for:** Learning noise handling
- **Output:** Threshold comparison

#### Example 03: Inductive Decomposition
- **File:** `examples/03_inductive_decomposition.rs`
- **Time:** 15 min
- **Lines:** 150
- **Best for:** Understanding trees
- **Output:** Process tree structure

#### Example 04: Algorithm Comparison
- **File:** `examples/04_algorithm_comparison.rs`
- **Time:** 10 min
- **Lines:** 180
- **Best for:** Choosing algorithm
- **Output:** Speed + quality comparison

---

### Conformance Examples

#### Example 05: Token Replay Basic
- **File:** `examples/05_token_replay_basic.rs`
- **Time:** 10 min
- **Lines:** 150
- **Best for:** First conformance check
- **Output:** Fitness scores for each trace

---

### Statistics Examples

#### Example 06: Statistical Analysis
- **File:** `examples/06_statistics_analysis.rs`
- **Time:** 10 min
- **Lines:** 180
- **Best for:** Understanding logs
- **Output:** Activity frequency, trace lengths, variants

---

### Filtering Examples

#### Example 07: Filtering Techniques
- **File:** `examples/07_filtering_techniques.rs`
- **Time:** 15 min
- **Lines:** 140
- **Best for:** Data preparation
- **Output:** Before/after filtering comparison

---

### Performance Examples

#### Example 08: Performance Benchmarking
- **File:** `examples/08_performance_benchmarking.rs`
- **Time:** 5 min
- **Lines:** 70
- **Best for:** Measuring performance
- **Output:** Algorithm timing table

---

### End-to-End Examples

#### Example 09: Complete Pipeline
- **File:** `examples/09_end_to_end_pipeline.rs`
- **Time:** 10 min
- **Lines:** 150
- **Best for:** Real workflow
- **Output:** Full pipeline results

#### Example 10: Variant Analysis
- **File:** `examples/10_variant_analysis.rs`
- **Time:** 10 min
- **Lines:** 160
- **Best for:** Understanding variants
- **Output:** Variant frequency + classification

---

## Reference Documents

### API Reference
**File:** `API_REFERENCE.md`
**Type:** Quick reference

Main types and functions:
- `EventLog`, `Trace`, `Event`
- `AlphaMiner`, `HeuristicMiner`, `InductiveMiner`, `TreeMiner`
- `TokenReplay`, `DFGConformance`
- `EventLog::statistics()`, `EventLog::filter_activities_by_threshold()`

---

### FAQ
**File:** `FAQ.md`
**Type:** Common questions

Covers:
- Installation
- Basic concepts
- Algorithm selection
- Performance
- Troubleshooting

---

## Learning Paths

### Path 1: Beginner (2 hours)
1. [Complete Getting Started Guide](COMPLETE_GETTING_STARTED.md) (15 min)
2. Run Examples 01, 04, 05, 06, 09 (30 min)
3. [Algorithm Deep-Dive](ALGORITHM_DEEPDIVE.md) (30 min)
4. Try modifying Example 01 yourself (45 min)

**Outcome:** Can discover models and check conformance

---

### Path 2: Intermediate (4 hours)
1. Complete Path 1 (2 hours)
2. Run Examples 02, 03, 07, 10 (40 min)
3. [Algorithm Deep-Dive](ALGORITHM_DEEPDIVE.md) (30 min)
4. [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md) (20 min)
5. Create your own example (40 min)

**Outcome:** Can handle noisy data, optimize performance

---

### Path 3: Advanced (6 hours)
1. Complete Path 2 (4 hours)
2. Run Example 08 (5 min)
3. [Advanced Topics](ADVANCED_TOPICS.md) (40 min)
4. Build custom miner (60 min)

**Outcome:** Can build production systems, domain-specific solutions

---

## Feature Coverage

### Discovery Algorithms
- ✅ Alpha Miner (Example 01, 04)
- ✅ Heuristic Miner (Example 02, 04)
- ✅ Inductive Miner (Example 03, 04)
- ✅ Tree Miner (Example 04)

### Conformance Checking
- ✅ Token Replay (Example 05, 09)
- ✅ Basic fitness scoring
- ✅ Trace-level analysis
- 📚 Advanced alignments (guide in Advanced Topics)

### Statistics & Analysis
- ✅ Log statistics (Example 06)
- ✅ Activity frequency
- ✅ Trace length distribution
- ✅ Temporal analysis
- ✅ Variant extraction (Example 10)

### Data Handling
- ✅ Event log creation
- ✅ Filtering by activity (Example 07)
- ✅ Filtering by threshold (Example 02)
- ✅ Filtering by trace length (Example 07)
- ✅ Sampling (Example 07)

### Performance
- ✅ Benchmarking (Example 08)
- ✅ Algorithm comparison
- ✅ Optimization strategies

---

## How to Use This Suite

### Scenario 1: "I'm new to process mining"
1. Read: [Complete Getting Started](COMPLETE_GETTING_STARTED.md)
2. Run: Example 01
3. Read: [Algorithm Deep-Dive](ALGORITHM_DEEPDIVE.md)
4. Run: Examples 02-05

### Scenario 2: "I have a large event log"
1. Read: [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md)
2. Run: Example 08
3. Run: Example 09 (adapt for your log)
4. Check: [Troubleshooting](COMPREHENSIVE_TROUBLESHOOTING.md) as needed

### Scenario 3: "I need a specific domain solution"
1. Read: [Advanced Topics](ADVANCED_TOPICS.md)
2. Adapt examples 01-10 for your domain
3. Follow custom development guide in Advanced Topics

### Scenario 4: "Something's broken"
1. Check: [Comprehensive Troubleshooting](COMPREHENSIVE_TROUBLESHOOTING.md)
2. Run: Relevant example to verify
3. Check: [FAQ](FAQ.md)

---

## Document Statistics

| Document | Lines | Type | Read Time |
|----------|-------|------|-----------|
| Complete Getting Started | 500 | Guide | 15 min |
| Algorithm Deep-Dive | 800 | Guide | 30 min |
| Troubleshooting | 700 | Reference | ~5 min (lookup) |
| Performance Tuning | 400 | Guide | 20 min |
| Advanced Topics | 600 | Guide | 40 min |
| Examples Index | 400 | Reference | 10 min |
| **Total Documentation** | **3400** | | **~120 min** |

Example Code Statistics:

| Category | Count | Lines | Time |
|----------|-------|-------|------|
| Discovery | 4 | 510 | 40 min |
| Conformance | 1 | 150 | 10 min |
| Statistics | 1 | 180 | 10 min |
| Filtering | 1 | 140 | 15 min |
| Performance | 1 | 70 | 5 min |
| End-to-End | 2 | 310 | 20 min |
| **Total Examples** | **10** | **1360** | **~100 min** |

---

## Quick Links

| Need | Document | Section |
|------|----------|---------|
| Get started quickly | [Complete Getting Started](COMPLETE_GETTING_STARTED.md) | Installation |
| Choose algorithm | [Algorithm Deep-Dive](ALGORITHM_DEEPDIVE.md) | Choosing the Right Algorithm |
| Fix a problem | [Troubleshooting](COMPREHENSIVE_TROUBLESHOOTING.md) | Problem categories |
| Optimize performance | [Performance Tuning](PERFORMANCE_TUNING_GUIDE.md) | Optimization Checklist |
| See working code | [Examples Index](EXAMPLES_INDEX.md) | All 10 examples |
| Answers to questions | [FAQ](FAQ.md) | 10+ questions |
| Build custom solution | [Advanced Topics](ADVANCED_TOPICS.md) | Custom algorithms |

---

## API Quick Reference

### Creating an Event Log
```rust
use pm4py::log::{EventLog, Trace, Event};
use chrono::Utc;

let mut log = EventLog::new();
let mut trace = Trace::new("case_1");
trace.add_event(Event::new("Activity", Utc::now()));
log.add_trace(trace);
```

### Discovery
```rust
use pm4py::discovery::{AlphaMiner, HeuristicMiner, InductiveMiner};

let alpha = AlphaMiner::new().discover(&log);
let heuristic = HeuristicMiner::with_threshold(0.5).discover(&log);
let inductive = InductiveMiner::new().discover(&log);
```

### Conformance
```rust
use pm4py::conformance::TokenReplay;

let checker = TokenReplay::new();
let results = checker.replay(&log, &model);
for result in results {
    println!("Fitness: {}", result.fitness);
}
```

### Statistics
```rust
let stats = log.statistics();
println!("Traces: {}", stats.num_traces);
println!("Activities: {}", stats.num_activities);
```

### Filtering
```rust
let filtered = log.filter_activities_by_threshold(0.95);
let sampled = log.sample(1000);
```

---

## Support & Further Help

- **API Docs:** `cargo doc --open`
- **Examples:** `cargo run --example <name>`
- **Troubleshooting:** See [Comprehensive Troubleshooting](COMPREHENSIVE_TROUBLESHOOTING.md)
- **GitHub Issues:** github.com/seanchatmangpt/pm4py-rust/issues
- **Email:** info@chatmangpt.com

---

## Version Information

- **PM4Py Rust:** 0.3.0
- **Rust Edition:** 2021
- **Minimum Rust:** 1.70.0
- **Documentation Date:** 2026-03-24

---

**Start here:** [Complete Getting Started Guide](COMPLETE_GETTING_STARTED.md)

**Questions?** Check [FAQ](FAQ.md) or [Troubleshooting](COMPREHENSIVE_TROUBLESHOOTING.md)

**Ready to code?** Run your first example: `cargo run --example 01_simple_alpha_miner`

---

**Happy process mining!** 🚀
