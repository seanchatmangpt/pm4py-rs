# PM4Py Rust Feature Guide

Complete documentation of all features, capabilities, and their current implementation status.

## Overview

PM4Py Rust provides **78% parity** with Python pm4py, covering:
- 8 discovery algorithms
- 3 conformance checking methods
- Comprehensive performance metrics
- Full statistical analysis
- XES and CSV I/O

## Process Discovery

Process discovery mines process models from event logs. Choose an algorithm based on your process characteristics.

### Algorithm Comparison

| Algorithm | Status | Best For | Characteristics |
|-----------|--------|----------|-----------------|
| **Alpha Miner** | ✅ Complete | Well-structured processes | Fundamental, simple, fast |
| **Inductive Miner** | ✅ Complete | Complex loops and hierarchies | Recursive, handles most cases well |
| **Heuristic Miner** | ✅ Complete | Noisy logs | Frequency-based, robust |
| **DFG Miner** | ✅ Complete | Quick visualization | Fast, simple graph extraction |
| **Causal Net Miner** | ✅ Complete | Dependency analysis | Shows causal relationships |
| **Split Miner** | ✅ Complete | Advanced structures | Split/join detection |
| **ILP Miner** | ✅ Complete | Optimized models | Integer linear programming |
| **Tree Miner** | ✅ Complete | Hierarchical view | Process tree generation |

### When to Use Which Algorithm

#### Alpha Miner
Best when:
- Process is simple and well-structured
- No or few loops
- Log is clean and complete

```rust
let miner = AlphaMiner::new();
let model = miner.discover(&log);
```

**Characteristics:**
- Guaranteed termination
- Fast execution
- May not find complex patterns

#### Inductive Miner
Best when:
- Process has complex structures
- Loops are common
- You want balanced model

```rust
let miner = InductiveMiner::new();
let model = miner.discover(&log);
```

**Characteristics:**
- Recursive decomposition
- Handles most real processes
- Generally produces good results

#### Heuristic Miner
Best when:
- Log contains noise or outliers
- Process is flexible
- You need robustness

```rust
let miner = HeuristicMiner::new();
// Customize threshold if needed
let model = miner.discover(&log);
```

**Characteristics:**
- Frequency-threshold based
- Robust to noise
- Configurable aggressiveness

#### DFG Miner
Best when:
- You just need the activity relationships
- Speed is critical
- Complex model not needed

```rust
let miner = DFGMiner::new();
let dfg = miner.discover(&log);
// dfg.activities, dfg.edges
```

**Characteristics:**
- Very fast
- Simple output
- Good for visualization

#### Others (Causal Net, Split, ILP, Tree)
**Advanced algorithms** for specialized use cases:
- Causal Net: Understanding dependencies
- Split Miner: Detecting parallel flows
- ILP Miner: Optimized minimal model
- Tree Miner: Hierarchical representation

## Conformance Checking

Conformance checking verifies if execution traces match an expected model.

### Available Methods

| Method | Status | Use Case | Speed |
|--------|--------|----------|-------|
| **Token Replay** | ✅ | Standard conformance | Fast |
| **Alignment** | ✅ | Detailed trace analysis | Slower |
| **Footprints** | ✅ | Quick ordering check | Very fast |

### Token Replay

Standard method for checking trace conformance:

```rust
use pm4py::conformance::TokenReplay;

let checker = TokenReplay::new();
let result = checker.check(&log, &petri_net);

println!("Fitness: {:.2}%", result.fitness * 100.0);
// Fitness 1.0 = perfect conformance
// Fitness 0.0 = no conformance
```

**Metrics returned:**
- `fitness`: Proportion of tokens successfully replayed (0-1)
- `precision`: How strictly model describes behavior
- `generalization`: Model's ability to handle variations

**When to use:**
- Standard conformance checking
- You need overall fitness score
- Model is Petri net

### Alignment Checker

Detailed analysis with path information:

```rust
use pm4py::conformance::AlignmentChecker;

let checker = AlignmentChecker::new();
let result = checker.check(&log, &petri_net);

// Returns cost-based alignments
// Shows exactly how each trace deviates
```

**When to use:**
- You need to identify specific deviations
- Want to understand WHERE traces fail
- Can afford higher computation cost

### Footprints Checking

Fast validation of ordering relationships:

```rust
use pm4py::conformance::FootprintsConformanceChecker;

let checker = FootprintsConformanceChecker::new();
let result = checker.check(&log, &petri_net);
```

**When to use:**
- Quick validation needed
- Low-latency requirements
- High-level conformance sufficient

## Performance Analysis

Extract timing and throughput metrics from event logs.

### Metrics Available

```rust
use pm4py::performance;

let metrics = performance::metrics::compute_performance_metrics(&log);

// Case/process level
metrics.avg_case_duration       // Average time from start to finish
metrics.min_case_duration       // Fastest case
metrics.max_case_duration       // Slowest case
metrics.median_case_duration    // Middle value

// Activity level
metrics.avg_activity_duration   // Average activity time
metrics.activity_frequency      // How often each activity occurs

// Throughput
metrics.throughput              // Cases per unit time
metrics.cycle_time              // Time between case starts

// Quality
metrics.rework_percentage       // Activities that repeat
metrics.waiting_time            // Idle time between activities
```

### Example: Performance Dashboard

```rust
use pm4py::performance;
use pm4py::log::EventLog;

fn analyze_performance(log: &EventLog) {
    let metrics = performance::metrics::compute_performance_metrics(log);

    println!("=== Performance Dashboard ===");
    println!("Cases processed: {}", log.traces.len());
    println!("Avg cycle time: {:?}", metrics.avg_case_duration);
    println!("Throughput: {:.2} cases/day",
             (86400.0 / metrics.avg_case_duration.as_secs_f64()));
    println!("Rework: {:.1}%", metrics.rework_percentage * 100.0);

    // Identify bottlenecks
    let activities = extract_activity_times(log);
    let slowest = activities.iter().max_by_key(|(_, time)| time);
    println!("Bottleneck: {:?}", slowest);
}
```

## Statistical Analysis

### Log Statistics

```rust
use pm4py::statistics::LogStatistics;

let stats = LogStatistics::compute(&log);

stats.total_events           // Total event count
stats.num_traces             // Number of process instances
stats.num_activities         // Unique activities
stats.avg_trace_length       // Average events per trace
stats.min_trace_length       // Shortest trace
stats.max_trace_length       // Longest trace
stats.start_activities       // What activities begin traces
stats.end_activities         // What activities end traces
stats.activity_frequency     // How often each activity occurs
stats.directly_follows       // Activity transitions
```

### Trace Statistics

```rust
use pm4py::statistics::TraceStatistics;

let trace_stats = TraceStatistics::compute(&log);

trace_stats.num_variants           // Number of unique execution paths
trace_stats.variants               // Map of path -> frequency
trace_stats.variant_frequency      // Distribution of variants

// Identify the "happy path"
let most_common = trace_stats.variants
    .iter()
    .max_by_key(|(_, count)| count);
println!("Most common path occurs {:.1}% of time",
         (most_common.map(|(_, c)| c).unwrap_or(&0) as f64 / log.traces.len() as f64) * 100.0);
```

### Tree Statistics

For process tree models:

```rust
use pm4py::statistics::TreeStatistics;

let tree_stats = TreeStatistics::compute(&tree);

tree_stats.num_nodes              // Total nodes in tree
tree_stats.depth                  // Tree depth
tree_stats.avg_branch_factor      // Average children per node
tree_stats.patterns               // Detected control structures
```

## Process Models

### Petri Net

Standard model for process modeling:

```rust
use pm4py::models::PetriNet;

let mut net = PetriNet::new("MyProcess");

// Add places (conditions/states)
net.add_place("start", 1);    // Initial marking
net.add_place("processing", 0);
net.add_place("done", 0);

// Add transitions (activities)
net.add_transition("Process");

// Add arcs (connections)
net.add_arc("start", "Process", 1);
net.add_arc("Process", "done", 1);
```

**Best for:**
- Formal verification
- Complex control flow
- Token-based simulation

### Process Tree

Hierarchical model:

```rust
use pm4py::models::ProcessTree;

let tree = TreeMiner::new().discover(&log);

// Tree contains nested operators
// Useful for recursive processes
```

**Best for:**
- Hierarchical understanding
- Structured processes
- Visual representation

### BPMN Diagram

Business Process Model and Notation:

```rust
use pm4py::models::{BPMNDiagram, BPMNXmlBuilder};

let mut bpmn = BPMNDiagram::new();
// Add elements, define flows
let xml = BPMNXmlBuilder::build(&bpmn);
```

**Best for:**
- Business stakeholder communication
- Standard notation
- Tool interchange

### Directly-Follows Graph (DFG)

Simple activity relationship graph:

```rust
use pm4py::discovery::DFGMiner;

let dfg = DFGMiner::new().discover(&log);

// dfg.activities: Vec<String>
// dfg.edges: Map of (from, to) transitions
```

**Best for:**
- Quick visualization
- Simple process view
- Fast analysis

### Causal Net

Dependency representation:

```rust
let causal_net = CausalNetMiner::new().discover(&log);

// Shows what must/can happen after each activity
// Good for understanding constraints
```

## Input/Output Formats

### XES (eXtensible Event Stream)

Standard process mining format - XML-based:

```rust
use pm4py::io::{XESReader, XESWriter};

// Read
let log = XESReader::read("events.xes")?;

// Write
XESWriter::write(&log, "output.xes")?;
```

**Advantages:**
- Standard format (IEEE 1849)
- Preserves all metadata
- Universally supported

**File structure:**
```xml
<?xml version="1.0"?>
<log xes.version="1.0">
  <trace>
    <string key="concept:name" value="case_1"/>
    <event>
      <string key="concept:name" value="Activity"/>
      <date key="time:timestamp" value="2024-01-01T00:00:00Z"/>
    </event>
  </trace>
</log>
```

### CSV (Comma-Separated Values)

Spreadsheet-compatible format:

```rust
use pm4py::io::{CSVReader, CSVWriter};

// Write
let columns = vec!["case_id", "activity", "timestamp", "resource"];
CSVWriter::write(&log, "events.csv", &columns)?;

// Read with column mapping
let mut mapping = HashMap::new();
mapping.insert("case_id", "case_id");
mapping.insert("activity", "activity");
CSVReader::read("events.csv", &mapping)?;
```

**Advantages:**
- Easy to edit in spreadsheet
- Human readable
- Good for data exchange

**Format:**
```csv
case_id,activity,timestamp,resource
order_1,Order Received,2024-01-01T08:00:00Z,Web
order_1,Payment,2024-01-01T08:05:00Z,PaymentGW
```

## Utilities and Encoding

### Log Operations

```rust
use pm4py::log::AdvancedFilter;

// Filtering
log.filter_by_date(start, end)?;
log.filter_outliers(2.0);           // Remove outliers
log.filter_variants(frequency)?;    // Keep frequent variants

// Merging
let merged = log1.merge(&log2);

// Sampling
let sample = log.sample_traces(0.5);  // 50% sample
```

### Encoding

Convert traces to feature vectors for machine learning:

```rust
use pm4py::utils::encoding;

// One-hot encoding
let vectors = encoding::one_hot_encode(&log)?;

// Frequency encoding
let vectors = encoding::frequency_encode(&log)?;

// Sequence encoding
let vectors = encoding::sequence_encode(&log)?;
```

## Feature Matrix: Quick Reference

| Feature | Status | Stability | Notes |
|---------|--------|-----------|-------|
| **Core Structures** | ✅ | Stable | Event, Trace, Log |
| **Alpha Mining** | ✅ | Stable | Basic algorithm |
| **Inductive Mining** | ✅ | Stable | Recommended |
| **Heuristic Mining** | ✅ | Stable | Good for noise |
| **DFG Mining** | ✅ | Stable | Fast |
| **ILP Mining** | ✅ | Stable | Optimization-based |
| **Split Mining** | ✅ | Stable | Advanced |
| **Tree Mining** | ✅ | Stable | Hierarchical |
| **Causal Net** | ✅ | Stable | Dependency-based |
| **Token Replay** | ✅ | Stable | Standard conformance |
| **Alignment** | ✅ | Stable | Detailed analysis |
| **Footprints** | ✅ | Stable | Fast checking |
| **Performance Metrics** | ✅ | Stable | Timing analysis |
| **Statistics** | ✅ | Stable | Log analysis |
| **XES I/O** | ✅ | Stable | Full support |
| **CSV I/O** | ✅ | Stable | Flexible mapping |
| **Visualization** | ✅ | Beta | SVG generation |
| **BPMN** | ✅ | Beta | XML export |
| **Petri Net** | ✅ | Stable | Full modeling |
| **Process Tree** | ✅ | Stable | Hierarchical |
| **Encoding** | ✅ | Stable | ML preparation |

## Performance Characteristics

### Algorithm Speed

On a typical dataset (10K events, 1K traces):

| Algorithm | Time | Memory | Notes |
|-----------|------|--------|-------|
| DFG Miner | <5ms | Low | Fastest |
| Alpha Miner | ~50ms | Low | Very fast |
| Token Replay | ~10ms | Low | Efficient |
| Heuristic Miner | ~100ms | Medium | Thorough |
| Inductive Miner | ~150ms | Medium | Recursive |
| ILP Miner | ~500ms | High | Optimization |
| Split Miner | ~200ms | Medium | Advanced |
| Tree Miner | ~300ms | Medium | Complete |

### Memory Usage

- Event Log: ~100 bytes per event
- Petri Net: ~1KB per place/transition
- Statistics: ~10KB per trace variant

Use `cargo build --release` for 2-3x performance improvement.

## What's New in This Version

### v0.1.0 (Current)
- ✅ 8 discovery algorithms
- ✅ Token replay & alignment conformance
- ✅ Full XES/CSV I/O
- ✅ Performance metrics
- ✅ Statistical analysis
- ✅ Process tree support
- ✅ BPMN diagram generation
- ✅ Multiple visualization options

### Coming in v0.2
- 🔄 Object-centric process mining
- 🔄 Real-time streaming
- 🔄 Advanced visualizations
- 🔄 Process simulation

## Version Compatibility

- **Rust versions**: 1.70+
- **Edition**: 2021
- **MSRV**: 1.70.0

## Next Steps

- [Getting Started](./GETTING_STARTED.md) — Quick start guide
- [Architecture](./ARCHITECTURE.md) — How it's built
- [FAQ](./FAQ.md) — Common questions
- [Examples](../examples/) — Runnable code
