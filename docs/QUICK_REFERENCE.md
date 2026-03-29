# PM4Py Rust - 5-Minute Quick Reference

> Print this, bookmark this, keep this handy.

## Install & Run

```bash
# Add to project
cargo add pm4py

# Run quickstart example
cargo run --example quickstart

# Run all examples
cargo run --example 1_alpha_miner_discovery
cargo run --example 2_heuristic_miner_filtering
cargo run --example 3_conformance_token_replay
cargo run --example 4_statistics_analysis
cargo run --example 5_end_to_end_pipeline
```

---

## Core Workflow (3 Steps)

```rust
use pm4py::log::{EventLog, Trace, Event};
use pm4py::discovery::alpha_miner::AlphaMiner;
use pm4py::conformance::token_replay::TokenReplay;

// 1. Create event log
let mut log = EventLog::new();
let mut trace = Trace::new("case_1".to_string());
trace.add_event(Event::new("Activity A".to_string(), Utc::now()));
log.add_trace(trace);

// 2. Discover model
let model = AlphaMiner::new().mine_from_log(&log);

// 3. Check conformance
let fitness = TokenReplay::new().check_log_fitness(&log, &model);
```

---

## Discovery Algorithms

| Algorithm | Best For | Speed | Command |
|-----------|----------|-------|---------|
| **Alpha** | Simple, structured processes | Fast | `AlphaMiner::new()` |
| **Inductive** | Complex with loops | Medium | `InductiveMiner::new()` |
| **Heuristic** | Noisy data | Medium | `HeuristicMiner::new()` |
| **DFG** | Quick overview | Very Fast | `DFGMiner::new()` |

---

## Load Data

### CSV
```rust
use pm4py::io::csv_reader::CSVReader;
let log = CSVReader::new().read_from_path(
    Path::new("data.csv"),
    "case_id",      // case ID column
    "activity",     // activity column
    "timestamp"     // timestamp column
)?;
```

### XES
```rust
use pm4py::io::xes_reader::XESReader;
let log = XESReader::new().read_from_path(Path::new("data.xes"))?;
```

---

## Common Tasks

### Filter Log
```rust
// By date
let filtered = log.filter_by_date_range(start, end);

// By activity
let filtered = log.filter_by_activity("Order Received");

// By trace length
let filtered = log.filter_by_trace_length_range(2, 10);
```

### Statistics
```rust
use pm4py::statistics::log_statistics::LogStatistics;
let stats = LogStatistics::compute(&log);
println!("Events: {}", log.num_events());
println!("Activities: {}", stats.num_activities());
```

### Export
```rust
use pm4py::io::xes_writer::XESWriter;
XESWriter::write_to_path(&log, Path::new("output.xes"))?;
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "cannot find EventLog" | Add `pm4py = "0.3"` to Cargo.toml |
| "no such module" | Use full path: `pm4py::discovery::alpha_miner::AlphaMiner` |
| "CSV parsing failed" | Check column names and timestamp format (ISO8601) |
| "Unexpected results" | Try Inductive Miner (better for real data) |
| "Slow performance" | Use release mode: `cargo run --release` |

---

## Key Concepts

- **Event Log**: Collection of process executions
- **Trace/Case**: Single process execution (one order, one ticket, etc.)
- **Event**: Activity at a point in time
- **Discovery**: Extracting process model from log
- **Conformance**: Checking if log matches model
- **Fitness**: Percentage of traces that fit the model (1.0 = perfect)

---

## Learn More

- **[Getting Started Guide](../docs/getting-started.md)** - 15-minute tutorial
- **[Features Guide](../docs/FEATURES.md)** - Complete feature matrix
- **[API Reference](../docs/API_REFERENCE.md)** - Full API docs
- **[FAQ](../docs/FAQ.md)** - Common questions

---

## Quick Tips

✅ Start with Alpha Miner for simple processes
✅ Use Inductive Miner for complex real-world data
✅ Always check conformance (fitness < 0.95 = issues)
✅ Filter data before discovery (removes noise)
✅ Use release mode for production (`--release`)
✅ Check example code in `examples/` folder

---

**Happy Process Mining!** 🚀
