# PM4Py Quick Start (5 Minutes)

> Get process mining running in 5 minutes with concrete examples.

---

## Install

```bash
cargo add pm4py chrono serde_json
```

---

## Example 1: Your First Process Model (2 min)

Create `main.rs`:

```rust
use pm4py::log::{Event, EventLog, Trace};
use pm4py::discovery::InductiveMiner;
use chrono::Utc;

fn main() {
    // Create event log
    let mut log = EventLog::new();
    let now = Utc::now();

    // Trace 1: Normal flow
    let mut t1 = Trace::new("order_001");
    t1.add_event(Event::new("receive", now));
    t1.add_event(Event::new("process", now + chrono::Duration::minutes(30)));
    t1.add_event(Event::new("ship", now + chrono::Duration::hours(2)));
    log.add_trace(t1);

    // Trace 2: Same flow (for pattern discovery)
    let mut t2 = Trace::new("order_002");
    t2.add_event(Event::new("receive", now + chrono::Duration::hours(1)));
    t2.add_event(Event::new("process", now + chrono::Duration::hours(1) + chrono::Duration::minutes(30)));
    t2.add_event(Event::new("ship", now + chrono::Duration::hours(3)));
    log.add_trace(t2);

    // Mine process model
    let miner = InductiveMiner::new();
    let model = miner.mine(&log);

    println!("✓ Discovered model:");
    println!("  Places: {}", model.places.len());
    println!("  Transitions: {}", model.transitions.len());
}
```

Run:
```bash
cargo run
```

Output:
```
✓ Discovered model:
  Places: 4
  Transitions: 3
```

---

## Example 2: Check if Reality Matches (2 min)

Add to `main.rs`:

```rust
use pm4py::conformance::TokenReplay;

fn main() {
    // ... (event log and model from before)

    // Check conformance
    let checker = TokenReplay::new();
    let result = checker.replay(&log, &model);

    println!("\n✓ Conformance Check:");
    println!("  Fitness: {:.1}%", result.fitness * 100.0);
    println!("  Precision: {:.1}%", result.precision * 100.0);
    println!("  Generalization: {:.1}%", result.generalization * 100.0);

    if result.fitness >= 0.95 {
        println!("  Status: ✓ Model matches reality");
    } else {
        println!("  Status: ✗ {} deviant cases", result.deviant_traces.len());
    }
}
```

Output:
```
✓ Conformance Check:
  Fitness: 100.0%
  Precision: 100.0%
  Generalization: 100.0%
  Status: ✓ Model matches reality
```

---

## Example 3: Find Bottlenecks (1 min)

Add to `main.rs`:

```rust
fn main() {
    // ... (previous code)

    // Statistics
    let stats = log.statistics();

    println!("\n✓ Process Statistics:");
    println!("  Total traces: {}", log.traces().len());
    println!("  Total events: {}", log.num_events());
    println!("  Activities: {}", stats.num_activities);

    for activity in &stats.activities {
        println!("    - {} (frequency: {})", activity.name, activity.frequency);
    }
}
```

---

## Quick Reference: Discovery Algorithms

| Algorithm | Use When | Speed | Quality |
|-----------|----------|-------|---------|
| **Alpha** | Process is simple, few loops | Fast | Perfect for well-structured |
| **Inductive** | Process has loops, recursion | Medium | Excellent |
| **Heuristic** | Data is noisy/incomplete | Fast | Good with filtering |
| **DFG** | Need quick overview | Very Fast | Basic structure only |

```rust
// Alpha: Simple processes
let model = pm4py::discovery::AlphaMiner::new().mine(&log);

// Inductive: Complex with loops
let model = pm4py::discovery::InductiveMiner::new().mine(&log);

// Heuristic: Noisy data
let model = pm4py::discovery::HeuristicMiner::new()
    .with_frequency_threshold(0.15)
    .mine(&log);

// DFG: Quick graph
let dfg = pm4py::discovery::DFGMiner::new().mine(&log);
```

---

## Quick Reference: Filtering Logs

```rust
// By activity
let orders_received = log.filter_by_activity("receive");

// By trace length
let short_traces = log.filter_by_trace_length_range(2, 5);

// By time period
let recent = log.filter_by_date_range(
    "2026-03-01T00:00:00Z",
    "2026-03-31T23:59:59Z"
);

// By attribute
let vip_orders = log.filter_by_attribute("customer_type", "VIP");
```

---

## Quick Reference: API Client

### Using REST API

```bash
# Get API key at https://dashboard.chatmangpt.com

API_KEY="your-api-key-here"

# Discover process model
curl -X POST https://api.chatmangpt.com/api/v1/discover \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d @request.json
```

File `request.json`:
```json
{
  "log": {
    "events": [
      {
        "case_id": "order_001",
        "activity": "receive",
        "timestamp": "2026-03-24T10:00:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "process",
        "timestamp": "2026-03-24T10:30:00Z"
      }
    ]
  },
  "algorithm": "inductive"
}
```

---

## Next Steps

1. **Read examples:** `examples/discovery.rs`, `examples/conformance.rs`
2. **Load real data:** See [I/O Guide](docs/GETTING_STARTED.md#loading-data)
3. **Explore API:** See [API Reference](docs/API_REFERENCE.md)
4. **Tune performance:** See [Performance Guide](docs/DEVELOPER_GUIDE.md#performance-tips)

---

## Common Issues

### "Activity X is missing from model"
→ Event log has activity not in model
→ Solution: Rediscover from complete log

### "Fitness score is 0"
→ Model structure doesn't match any trace
→ Solution: Try different algorithm or check data quality

### "Out of memory"
→ Log too large
→ Solution: Filter or sample first

See [Troubleshooting](docs/DEVELOPER_GUIDE.md#troubleshooting) for more.

---

## Full Working Example

Complete code in one file (`examples/quickstart.rs`):

```rust
use pm4py::log::{Event, EventLog, Trace};
use pm4py::discovery::InductiveMiner;
use pm4py::conformance::TokenReplay;
use pm4py::statistics::LogStatistics;
use chrono::Utc;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║     PM4Py Rust - 5 Minute Quick Start     ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Step 1: Create event log
    println!("STEP 1: Creating event log...");
    let log = create_sample_log();
    println!("✓ Created log with {} traces\n", log.traces().len());

    // Step 2: Discover process
    println!("STEP 2: Discovering process model...");
    let miner = InductiveMiner::new();
    let model = miner.mine(&log);
    println!("✓ Discovered {} places, {} transitions\n",
             model.places.len(), model.transitions.len());

    // Step 3: Check conformance
    println!("STEP 3: Checking conformance...");
    let checker = TokenReplay::new();
    let result = checker.replay(&log, &model);
    println!("✓ Fitness: {:.1}%\n", result.fitness * 100.0);

    // Step 4: Statistics
    println!("STEP 4: Analyzing statistics...");
    let stats = log.statistics();
    println!("✓ {} activities, {} total events\n",
             stats.num_activities, log.num_events());

    println!("╔════════════════════════════════════════════╗");
    println!("║          Quick Start Complete!            ║");
    println!("╚════════════════════════════════════════════╝");
}

fn create_sample_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();

    for i in 1..=3 {
        let mut trace = Trace::new(format!("order_{:03}", i));
        let offset = now + chrono::Duration::hours(i as i64);

        trace.add_event(Event::new("receive", offset));
        trace.add_event(Event::new(
            "process",
            offset + chrono::Duration::minutes(30)
        ));
        trace.add_event(Event::new(
            "ship",
            offset + chrono::Duration::hours(2)
        ));

        log.add_trace(trace);
    }

    log
}
```

Run:
```bash
cargo run --example quickstart
```

---

## Key Takeaways

✓ **5 lines** to create an event log
✓ **1 line** to discover a process model
✓ **1 line** to check conformance
✓ **Production-ready** Rust library
✓ **REST API** for cloud deployment

---

## Where to Go Next

- **Library API:** See `/docs/DEVELOPER_GUIDE.md`
- **REST API:** See `/docs/API_REFERENCE.md`
- **Full Examples:** See `/examples/` directory
- **Performance:** See `/docs/PERFORMANCE.md`

---

**Questions?**
→ GitHub: https://github.com/seanchatmangpt/pm4py-rust/issues
→ Email: info@chatmangpt.com

Enjoy! 🚀
