# Getting Started with PM4Py Rust

> From zero to productive in 15 minutes

Welcome! This guide will walk you through your first process mining project with PM4Py Rust.

---

## What You'll Learn

In this guide, you'll:
- ✅ Set up a Rust project with PM4Py
- ✅ Create your first event log
- ✅ Discover a process model
- ✅ Check conformance
- ✅ Analyze performance
- ✅ Load real data from CSV/XES

**Time:** 15 minutes | **Prerequisites:** Basic Rust knowledge

---

## Prerequisites

1. **Install Rust** (if you haven't already)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Verify installation**

```bash
rustc --version  # Should be 1.70 or newer
cargo --version
```

---

## Step 1: Create Your Project

```bash
cargo new my_process_mining
cd my_process_mining
```

---

## Step 2: Add PM4Py Dependency

Edit `Cargo.toml`:

```toml
[dependencies]
pm4py = "0.3"
chrono = { version = "0.4", features = ["serde"] }
```

Save the file. Cargo will download dependencies automatically when you build.

---

## Step 3: Your First Process Mining Program

Replace `src/main.rs` with this complete working example:

```rust
use pm4py::log::{EventLog, Trace, Event};
use pm4py::discovery::alpha_miner::AlphaMiner;
use chrono::Utc;

fn main() {
    println!("🦀 PM4Py Rust - Your First Process Mining Program\n");

    // Step 1: Create an event log
    let mut log = EventLog::new();
    let now = Utc::now();

    // Step 2: Add some traces (real-world process executions)
    for i in 1..=5 {
        let mut trace = Trace::new(format!("order_{:03}", i));
        let offset = now + chrono::Duration::hours(i as i64);

        // Each trace represents one order going through the process
        trace.add_event(Event::new("Order Received".to_string(), offset));
        trace.add_event(Event::new(
            "Payment Processed".to_string(),
            offset + chrono::Duration::minutes(30),
        ));
        trace.add_event(Event::new(
            "Item Shipped".to_string(),
            offset + chrono::Duration::hours(2),
        ));

        log.add_trace(trace);
    }

    println!("✓ Created event log with {} traces", log.traces().len());

    // Step 3: Discover a process model
    let miner = AlphaMiner::new();
    let petri_net = miner.mine_from_log(&log);

    println!(
        "✓ Discovered process model: {} places, {} transitions",
        petri_net.places.len(),
        petri_net.transitions.len()
    );

    println!("\n🎉 Success! You've discovered your first process model!");
}
```

---

## Step 4: Run Your Program

```bash
cargo run
```

Expected output:
```
🦀 PM4Py Rust - Your First Process Mining Program

✓ Created event log with 5 traces
✓ Discovered process model: X places, Y transitions

🎉 Success! You've discovered your first process model!
```

---

## Step 5: Understand What Just Happened

### What is an Event Log?

An **event log** is a collection of recorded process executions. Each execution is called a **trace** (or case), and contains a sequence of **events**.

```
Event Log
├── Trace 1: order_001
│   ├── Event: Order Received (10:00)
│   ├── Event: Payment Processed (10:30)
│   └── Event: Item Shipped (12:00)
├── Trace 2: order_002
│   ├── Event: Order Received (11:00)
│   ├── Event: Payment Processed (11:30)
│   └── Event: Item Shipped (13:00)
└── ...
```

### What is Process Discovery?

**Process discovery** automatically extracts a process model from an event log. The model shows:
- Which activities can happen
- In what order they can occur
- Where parallel paths exist
- Where loops (repetition) occur

### What is a Petri Net?

A **Petri net** is a mathematical modeling language for distributed systems. In process mining:
- **Places** (circles): States in the process
- **Transitions** (rectangles): Activities
- **Arcs**: Connect places to transitions (and vice versa)

---

## Step 6: Try Different Discovery Algorithms

PM4Py Rust provides multiple discovery algorithms. Update your `main.rs`:

```rust
use pm4py::discovery::{alpha_miner::AlphaMiner, inductive_miner::InductiveMiner};

fn main() {
    // ... (create event log as before)

    // Try Alpha Miner
    let alpha_miner = AlphaMiner::new();
    let alpha_net = alpha_miner.mine_from_log(&log);
    println!("Alpha: {} places", alpha_net.places.len());

    // Try Inductive Miner (better for complex processes)
    let inductive_miner = InductiveMiner::new();
    let inductive_net = inductive_miner.mine_from_log(&log);
    println!("Inductive: {} places", inductive_net.places.len());
}
```

**When to use which algorithm?**
- **Alpha Miner**: Simple, well-structured processes
- **Inductive Miner**: Complex processes with loops and parallelism
- **Heuristic Miner**: Noisy data, infrequent paths
- **DFG Miner**: Quick overview, directly-follows graph

---

## Step 7: Check Conformance

Conformance checking compares observed behavior (event log) with modeled behavior (process model).

```rust
use pm4py::conformance::token_replay::TokenReplay;

fn main() {
    // ... (discover model as before)

    // Check conformance
    let checker = TokenReplay::new();
    let result = checker.check_log_fitness(&log, &petri_net);

    println!("\n📊 Conformance Results:");
    println!("  Fitness: {:.1}%", result * 100.0);

    if result >= &0.95 {
        println!("  ✅ Excellent! Model matches reality");
    } else {
        println!("  ⚠️  Some deviations detected");
    }
}
```

**Fitness = 1.0** means perfect conformance (all traces fit the model).
**Fitness < 1.0** means some traces deviate from the model.

---

## Step 8: Analyze Performance

Extract performance insights from your event log:

```rust
use pm4py::statistics::log_statistics::LogStatistics;

fn main() {
    // ... (create event log as before)

    let stats = LogStatistics::compute(&log);

    println!("\n📈 Process Statistics:");
    println!("  Total traces: {}", log.traces().len());
    println!("  Total events: {}", log.num_events());
    println!("  Unique activities: {}", stats.num_activities());
}
```

---

## Step 9: Load Real Data

So far we've created synthetic data. Let's load real event logs.

### Option A: Load from CSV

Create `data/orders.csv`:
```csv
case_id,activity,timestamp
order_001,Order Received,2026-03-24T10:00:00Z
order_001,Payment Processed,2026-03-24T10:30:00Z
order_001,Item Shipped,2026-03-24T12:00:00Z
order_002,Order Received,2026-03-24T11:00:00Z
order_002,Payment Processed,2026-03-24T11:30:00Z
order_002,Item Shipped,2026-03-24T13:00:00Z
```

Load it in Rust:

```rust
use pm4py::io::csv_reader::CSVReader;
use std::path::Path;

fn main() {
    let reader = CSVReader::new();
    let log = reader.read_from_path(
        Path::new("data/orders.csv"),
        "case_id",      // case ID column
        "activity",     // activity column
        "timestamp"     // timestamp column
    ).expect("Failed to read CSV");

    println!("Loaded {} traces from CSV", log.traces().len());

    // Now you can discover, check conformance, etc.
    let miner = AlphaMiner::new();
    let petri_net = miner.mine_from_log(&log);
    // ...
}
```

### Option B: Load from XES

XES is the standard eXtensible Event Stream format for process mining.

```rust
use pm4py::io::xes_reader::XESReader;
use std::path::Path;

fn main() {
    let reader = XESReader::new();
    let log = reader.read_from_path(Path::new("data/log.xes"))
        .expect("Failed to read XES");

    println!("Loaded {} traces from XES", log.traces().len());
}
```

---

## Common Patterns

### Pattern 1: Filter and Discover

```rust
// Filter to only recent traces
let recent_log = log.filter_by_date_range(
    "2026-03-01T00:00:00Z",
    "2026-03-31T23:59:59Z"
);

// Discover from filtered log
let model = AlphaMiner::new().mine_from_log(&recent_log);
```

### Pattern 2: Compare Algorithms

```rust
let algorithms = [
    ("Alpha", AlphaMiner::new().mine_from_log(&log)),
    ("Inductive", InductiveMiner::new().mine_from_log(&log)),
];

for (name, model) in &algorithms {
    let fitness = TokenReplay::new().check_log_fitness(&log, model);
    println!("{}: {:.1}% fitness", name, fitness * 100.0);
}
```

### Pattern 3: Export Results

```rust
use pm4py::io::xes_writer::XESWriter;
use std::path::Path;

// Save discovered model
XESWriter::write_to_path(&log, Path::new("output.xes"))
    .expect("Failed to write XES");
```

---

## Troubleshooting

### "cannot find type `EventLog`"
**Solution**: Make sure `pm4py = "0.3"` is in `Cargo.toml` and run `cargo build`.

### "no such module as `discovery`"
**Solution**: Use the full module path: `pm4py::discovery::alpha_miner::AlphaMiner`

### "CSV parsing failed"
**Solution**: Check your CSV has the correct column names and valid ISO8601 timestamps.

### "Discovery produces unexpected results"
**Solution**: Try a different algorithm. Inductive Miner often works best for real-world data.

---

## What's Next?

### Learn More
- 📖 **[Features Guide](./FEATURES.md)** — Complete feature matrix
- 🏗️ **[Architecture](./ARCHITECTURE.md)** — How the library is organized
- 🚀 **[Performance Guide](./PERFORMANCE.md)** — Optimization techniques
- ❓ **[FAQ](./FAQ.md)** — Common questions

### Try Real Examples
- 📊 **Business Analytics**: Analyze your business processes
- 🔍 **Audit Trails**: Verify compliance with process models
- ⚡ **Performance**: Find bottlenecks and optimize workflows
- 🤖 **Automation**: Build automated process monitoring

### Advanced Topics
- **Conformance Checking**: Detect deviations and anomalies
- **Performance Analysis**: Identify bottlenecks and optimize
- **Organizational Mining**: Analyze resource behavior
- **Process Simulation**: Predict process outcomes

---

## Complete Working Example

Here's a complete example combining everything:

```rust
use pm4py::log::{EventLog, Trace, Event};
use pm4py::discovery::alpha_miner::AlphaMiner;
use pm4py::conformance::token_replay::TokenReplay;
use pm4py::statistics::log_statistics::LogStatistics;
use chrono::Utc;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║   PM4Py Rust - Complete Example             ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Create sample event log
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 1..=10 {
        let mut trace = Trace::new(format!("order_{:03}", i));
        let offset = base + chrono::Duration::hours(i as i64);

        trace.add_event(Event::new("Order Received".to_string(), offset));
        trace.add_event(Event::new(
            "Payment Processed".to_string(),
            offset + chrono::Duration::minutes(30),
        ));
        trace.add_event(Event::new(
            "Item Shipped".to_string(),
            offset + chrono::Duration::hours(2),
        ));

        log.add_trace(trace);
    }

    // Statistics
    let stats = LogStatistics::compute(&log);
    println!("📊 Event Log Statistics:");
    println!("  Traces: {}", log.traces().len());
    println!("  Events: {}", log.num_events());
    println!("  Activities: {}\n", stats.num_activities());

    // Discover process model
    let miner = AlphaMiner::new();
    let petri_net = miner.mine_from_log(&log);
    println!("🔍 Process Discovery:");
    println!("  Places: {}", petri_net.places.len());
    println!("  Transitions: {}\n", petri_net.transitions.len());

    // Check conformance
    let fitness = TokenReplay::new().check_log_fitness(&log, &petri_net);
    println!("✅ Conformance:");
    println!("  Fitness: {:.1}%\n", fitness * 100.0);

    println!("╔════════════════════════════════════════════╗");
    println!("║   Analysis Complete!                       ║");
    println!("╚════════════════════════════════════════════╝");
}
```

Run it:
```bash
cargo run
```

---

## Getting Help

- 📚 **Documentation**: See `/docs/` directory
- 💬 **GitHub Issues**: https://github.com/seanchatmangpt/pm4py-rust/issues
- 📧 **Email**: info@chatmangpt.com

---

**Happy Process Mining!** 🚀
