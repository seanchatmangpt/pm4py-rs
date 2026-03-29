# pm4py-rust Examples

This directory contains runnable examples demonstrating the core capabilities of the pm4py-rust process mining library.

## Quick Start

Run any example directly:

```bash
# From the pm4py-rust root directory
cargo run --example 1_alpha_miner_discovery
cargo run --example 2_heuristic_miner_filtering
cargo run --example 3_conformance_token_replay
cargo run --example 4_statistics_analysis
cargo run --example 5_end_to_end_pipeline
```

## Example Overview

### 1. Alpha Miner Discovery (`1_alpha_miner_discovery.rs`)

**Difficulty:** Beginner | **Time:** 5 minutes

Demonstrates the simplest form of process discovery:
- Reading event logs from CSV or creating sample data
- Discovering process models with Alpha Miner
- Displaying Petri net structure
- Exporting logs to XES format

**Use case:** Quick exploration of simple processes with clear patterns.

---

### 2. Heuristic Miner with Filtering (`2_heuristic_miner_filtering.rs`)

**Difficulty:** Intermediate | **Time:** 10 minutes

Demonstrates robust process discovery for real-world data:
- Creating realistic event logs with multiple variants
- Analyzing process variants
- Filtering to top 80% of variants
- Configurable Heuristic Miner thresholds
- Performance analysis (cycle times, bottlenecks)

**Use case:** Real-world logs with noise and multiple execution paths.

---

### 3. Conformance Checking (`3_conformance_token_replay.rs`)

**Difficulty:** Intermediate | **Time:** 10 minutes

Demonstrates model validation and deviation detection:
- Creating "golden" process models
- Simulating various deviation types
- Token replay conformance checking
- Analyzing deviation patterns
- Generating recommendations

**Use case:** Validating process compliance and detecting deviations.

---

### 4. Statistics Analysis (`4_statistics_analysis.rs`)

**Difficulty:** Intermediate | **Time:** 15 minutes

Demonstrates comprehensive statistical analysis:
- Basic log statistics
- Activity frequency analysis
- Resource workload analysis
- Case duration analysis
- Temporal analysis (day-of-week, hourly)
- Bottleneck detection
- Variant analysis

**Use case:** Understanding process performance and identifying improvements.

---

### 5. End-to-End Pipeline (`5_end_to_end_pipeline.rs`)

**Difficulty:** Advanced | **Time:** 20 minutes

Production-ready process mining pipeline:
- Data loading (CSV or synthetic)
- Data quality assessment
- Data cleaning and filtering
- Process discovery
- Conformance checking
- Performance analysis
- Organizational analysis
- Automated insights generation
- Result export (XES, CSV, reports)

**Use case:** Real process mining projects requiring comprehensive analysis.

---

## Data Files

### Sample Data Format

Example CSV files should follow this structure:

```csv
case_id,activity,timestamp,resource
case-001,Submit Request,2024-01-01T09:00:00Z,Alice
case-001,Review Request,2024-01-01T10:00:00Z,Bob
case-001,Approve Request,2024-01-01T11:00:00Z,Carol
```

### Provided Sample Data

- `data/running-example.csv` - Classic process mining example log
- `data/pipeline_log.csv` - Generated automatically by Example 5

### Output Directory

Examples create output files in `examples/data/output/`:
- `filtered_log.xes` - Filtered event log (XES format)
- `filtered_log.csv` - Filtered event log (CSV format)
- `insights_report.txt` - Analysis insights and recommendations

---

## Common Patterns

### Creating Event Logs

```rust
use pm4py::{Event, Trace, EventLog};
use chrono::Utc;

let mut log = EventLog::new();
let mut trace = Trace::new("case-001");

trace.add_event(
    Event::new("Submit Request", Utc::now())
        .with_resource("Alice")
);

log.add_trace(trace);
```

### Reading Logs

```rust
use pm4py::io::{CSVReader, XESReader};
use std::path::Path;

// CSV
let reader = CSVReader::new();
let log = reader.read(Path::new("data.csv"))?;

// XES
let reader = XESReader::new();
let log = reader.read(Path::new("data.xes"))?;
```

### Discovering Models

```rust
use pm4py::discovery::{AlphaMiner, InductiveMiner, HeuristicMiner};

// Alpha Miner (fast, simple)
let miner = AlphaMiner::new();
let petri_net = miner.discover(&log);

// Inductive Miner (robust)
let miner = InductiveMiner::new();
let petri_net = miner.discover(&log);

// Heuristic Miner (noise-tolerant)
let miner = HeuristicMiner::new()
    .with_dependency_threshold(0.9);
let petri_net = miner.discover(&log);
```

### Checking Conformance

```rust
use pm4py::conformance::TokenReplay;

let checker = TokenReplay::new();
let result = checker.check(&log, &model);

println!("Fitness: {:.2}%", result.fitness * 100.0);
```

---

## Troubleshooting

### Compilation Errors

Ensure you're running from the pm4py-rust root directory:
```bash
cd /path/to/pm4py-rust
cargo build --release
cargo run --example 1_alpha_miner_discovery
```

### Missing Data Files

Examples create sample data automatically if CSV files are missing.

### Runtime Errors

Ensure write permissions for `examples/data/output/` directory.

---

## Next Steps

1. **Try the examples**: Run each example to understand the concepts
2. **Use your own data**: Replace sample data with your event logs
3. **Customize**: Modify examples to fit your use case
4. **Build tools**: Use Example 5 as a template for production applications

---

## Additional Resources

- **Main Docs**: `cargo doc --open`
- **Source Code**: `/path/to/pm4py-rust/src/`
- **Tests**: `/path/to/pm4py-rust/tests/`

---

## License

AGPL-3.0-or-later
