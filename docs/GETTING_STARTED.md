# Getting Started with PM4Py Rust

Welcome! This guide will get you from zero to a working process mining application in 15 minutes.

## Prerequisites

- **Rust 1.70+** — Get it from [rustup.rs](https://rustup.rs/)
- **Cargo** — Included with Rust
- A text editor (VS Code, Vim, etc.)

Verify installation:
```bash
rustc --version   # Should be 1.70 or newer
cargo --version
```

## Step 1: Create a New Project

```bash
cargo new my_process_mining --bin
cd my_process_mining
```

## Step 2: Add Dependencies

Edit `Cargo.toml`:

```toml
[dependencies]
pm4py = "0.1"
chrono = { version = "0.4", features = ["serde"] }
```

Save and cargo will automatically download dependencies on next build.

## Step 3: Your First Program

Replace `src/main.rs` with:

```rust
use pm4py::log::{EventLog, Trace, Event};
use pm4py::discovery::AlphaMiner;
use chrono::Utc;

fn main() {
    // Step 1: Create an event log
    let mut log = EventLog::new();
    let now = Utc::now();

    // Step 2: Add traces
    for i in 0..3 {
        let mut trace = Trace::new(format!("case_{}", i));
        let offset = now + chrono::Duration::hours(i as i64);

        trace.add_event(Event::new("Order", offset));
        trace.add_event(Event::new("Payment", offset + chrono::Duration::minutes(10)));
        trace.add_event(Event::new("Ship", offset + chrono::Duration::hours(1)));

        log.add_trace(trace);
    }

    println!("Loaded {} traces", log.traces.len());

    // Step 3: Discover a process model
    let miner = AlphaMiner::new();
    let petri_net = miner.discover(&log);

    println!("Discovered {} places and {} transitions",
             petri_net.places.len(),
             petri_net.transitions.len());
}
```

## Step 4: Run It

```bash
cargo run
```

Expected output:
```
Loaded 3 traces
Discovered X places and Y transitions
```

**Congratulations!** You've just discovered your first process model.

## Next Steps

### Run the Examples

The library includes 5 complete examples:

```bash
# Process discovery with multiple algorithms
cargo run --example discovery

# Conformance checking
cargo run --example conformance

# Statistical analysis
cargo run --example analysis

# Reading/writing logs
cargo run --example io

# Visualization
cargo run --example visualization
```

Each example is fully runnable and documented. Start with `discovery` to see all algorithms in action.

### Common Tasks

#### Task 1: Load a Real Event Log

```rust
use pm4py::io::XESReader;

// Load XES file
let log = XESReader::read("path/to/event_log.xes")?;
println!("Loaded {} traces", log.traces.len());
```

#### Task 2: Try Different Algorithms

```rust
use pm4py::discovery::{
    AlphaMiner, InductiveMiner, HeuristicMiner
};

let alpha_net = AlphaMiner::new().discover(&log);
let inductive_net = InductiveMiner::new().discover(&log);
let heuristic_net = HeuristicMiner::new().discover(&log);

// Each discovers the model slightly differently
// Try all three and see which fits best
```

#### Task 3: Check Conformance

```rust
use pm4py::conformance::TokenReplay;

let checker = TokenReplay::new();
let result = checker.check(&log, &petri_net);

println!("Fitness: {:.1}%", result.fitness * 100.0);

// Fitness = 1.0 means perfect conformance
// Fitness < 1.0 means some deviations
```

#### Task 4: Analyze Performance

```rust
use pm4py::statistics::LogStatistics;

let stats = LogStatistics::compute(&log);

println!("Total events: {}", stats.total_events);
println!("Average trace length: {}", stats.avg_trace_length);
println!("Unique activities: {}", stats.num_activities);
```

#### Task 5: Export Results

```rust
use pm4py::io::XESWriter;

// Save the log
XESWriter::write(&log, "output.xes")?;

// Save the discovered model (as Petri net)
// Models can be exported as XES, DOT, or visualized as SVG
```

## Architecture Overview

PM4Py Rust is organized into modules:

```
pm4py/
├── log/          Event Log, Trace, Event structures
├── discovery/    Alpha, Inductive, Heuristic, ILP, Split, Tree mining
├── conformance/  Token replay, alignment, footprints checking
├── performance/  Metrics, throughput, rework analysis
├── statistics/   Log and trace statistics
├── io/          XES and CSV reading/writing
├── models/       Petri net, process tree, BPMN, causal net
├── utils/        Encoding, filtering, sampling
└── visualization/ SVG generation
```

## Common Patterns

### Pattern 1: Discover and Check

```rust
// Discover a model
let model = AlphaMiner::new().discover(&log);

// Check if traces conform
let result = TokenReplay::new().check(&log, &model);

// High fitness = good model
if result.fitness > 0.95 {
    println!("Excellent conformance!");
}
```

### Pattern 2: Filter and Analyze

```rust
// Filter outliers
let filtered = log.filter_outliers(2.0);

// Discover from filtered log
let clean_model = AlphaMiner::new().discover(&filtered);

// Compare statistics
let before = LogStatistics::compute(&log);
let after = LogStatistics::compute(&filtered);
```

### Pattern 3: Try Multiple Algorithms

```rust
let algorithms = [
    ("Alpha", AlphaMiner::new().discover(&log)),
    ("Inductive", InductiveMiner::new().discover(&log)),
    ("Heuristic", HeuristicMiner::new().discover(&log)),
];

let checker = TokenReplay::new();

for (name, model) in &algorithms {
    let result = checker.check(&log, model);
    println!("{}: {:.1}% fitness", name, result.fitness * 100.0);
}
```

## Troubleshooting

### Issue: "cannot find type `EventLog`"
**Solution:** Make sure you have `pm4py` in `Cargo.toml` and run `cargo build` first.

### Issue: Compilation fails with type errors
**Solution:** PM4Py Rust is strongly typed. Check:
- Event timestamps must be `DateTime<Utc>`
- Trace IDs must be `String` or convertible to `String`
- Use `.clone()` if moving values between scopes

### Issue: Discovery produces unexpected results
**Try:**
1. Check your event log has at least 10-20 traces
2. Try a different algorithm (Inductive Miner often works best)
3. Check for noise using `LogStatistics::compute()`
4. Filter outliers with `log.filter_outliers(2.0)`

### Issue: Performance is slow
**Tips:**
1. Use release build: `cargo build --release`
2. Filter log to recent events: `log.filter_by_date(start, end)`
3. Sample traces: `log.sample_traces(0.5)` for 50% sample
4. Try smaller datasets first

## What to Read Next

- **[Features Guide](./FEATURES.md)** — Detailed feature matrix
- **[Architecture](./ARCHITECTURE.md)** — How the library is organized
- **[FAQ](./FAQ.md)** — Common questions
- **[Main README](../README.md)** — Project overview

## Real-World Example: Order Processing

Here's a complete example analyzing an order processing system:

```rust
use pm4py::log::{EventLog, Trace, Event};
use pm4py::discovery::AlphaMiner;
use pm4py::conformance::TokenReplay;
use pm4py::statistics::LogStatistics;
use chrono::Utc;

fn main() {
    // Create log from 100 orders
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..100 {
        let mut trace = Trace::new(format!("order_{}", i));
        let offset = base + chrono::Duration::hours(i as i64);

        trace.add_event(Event::new("Order Placed", offset));
        trace.add_event(Event::new("Payment Confirmed", offset + chrono::Duration::minutes(5)));
        trace.add_event(Event::new("Picked", offset + chrono::Duration::minutes(30)));
        trace.add_event(Event::new("Packed", offset + chrono::Duration::minutes(45)));
        trace.add_event(Event::new("Shipped", offset + chrono::Duration::hours(2)));

        log.add_trace(trace);
    }

    // Analyze
    let stats = LogStatistics::compute(&log);
    println!("Processed {} orders", stats.num_traces);
    println!("Average trace length: {:.1} activities", stats.avg_trace_length);

    // Discover ideal model
    let model = AlphaMiner::new().discover(&log);

    // Check conformance
    let result = TokenReplay::new().check(&log, &model);
    println!("Process efficiency: {:.1}%", result.fitness * 100.0);
}
```

Run this and you'll have a complete process mining analysis!

## Getting Help

- Check the [FAQ](./FAQ.md)
- Review [example code](../examples/)
- Read the [Features Guide](./FEATURES.md)
- Check implementation details in source code (`src/discovery/`, `src/conformance/`, etc.)

## What's Next?

1. **Experiment** with the examples
2. **Load your own data** (XES or CSV format)
3. **Compare algorithms** to find what works best
4. **Integrate** into your application
5. **Contribute** improvements back to the project!

Happy process mining!
