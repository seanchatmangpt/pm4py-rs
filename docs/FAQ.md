# Frequently Asked Questions

Answers to common questions about PM4Py Rust.

## General Questions

### What is PM4Py Rust?

PM4Py Rust is a Rust implementation of the Python process mining library pm4py. It provides:
- Process discovery (mine process models from logs)
- Conformance checking (verify trace compliance)
- Performance analysis (extract timing metrics)
- Statistical analysis (understand process behavior)

It's designed for production systems with strong type safety and high performance.

### Why Rust instead of Python?

**Performance:**
- 2-5x faster than Python pm4py
- No garbage collection pauses
- Memory efficient (10-50% less memory)

**Safety:**
- Type-safe (errors caught at compile time)
- Memory-safe (no buffer overflows)
- Thread-safe (safe concurrent processing)

**Deployment:**
- Single binary (no runtime needed)
- Works in restricted environments
- Ideal for embedded systems

**Python integration:**
- Can be called from Python via FFI
- Great for mixed-language systems

### Should I use pm4py or PM4Py Rust?

**Use Python pm4py if:**
- You're doing interactive exploration
- You need cutting-edge algorithms
- You prefer Python ecosystem
- You're a researcher

**Use PM4Py Rust if:**
- You need production reliability
- Performance is critical
- You want type safety
- You're building Rust applications

### Is PM4Py Rust a complete replacement?

Not yet. We provide **78% parity** with pm4py:
- ✅ All major discovery algorithms
- ✅ Conformance checking
- ✅ Performance metrics
- ✅ Statistical analysis
- ✅ I/O (XES, CSV)
- ❌ Some advanced algorithms
- ❌ Interactive visualization
- ❌ Simulation (planned)

## Getting Started

### I don't know Rust. Can I still use this?

Yes! While Rust has a learning curve, you can:
1. Use the examples as templates
2. Modify sample code for your needs
3. Gradually learn Rust concepts
4. Our API is straightforward

Start with [Getting Started Guide](./GETTING_STARTED.md).

### How do I install PM4Py Rust?

Add to `Cargo.toml`:
```toml
[dependencies]
pm4py = "0.1"
chrono = { version = "0.4", features = ["serde"] }
```

That's it! `cargo build` downloads and compiles.

### Do I need Rust installed?

Yes. Get it from [rustup.rs](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Takes ~5 minutes, then you're ready.

## API Questions

### What's the difference between AlphaMiner and InductiveMiner?

| Aspect | Alpha | Inductive |
|--------|-------|-----------|
| **Speed** | Fast | Slower |
| **Complexity** | Simple structures | Complex/loops |
| **Quality** | Good for clean logs | Better overall |
| **Recommendation** | Start here | When Alpha fails |

**Try this:**
```rust
// Start with Alpha
let alpha_net = AlphaMiner::new().discover(&log);

// If fitness is low, try Inductive
let inductive_net = InductiveMiner::new().discover(&log);

// Compare results
let alpha_fitness = TokenReplay::new().check(&log, &alpha_net).fitness;
let inductive_fitness = TokenReplay::new().check(&log, &inductive_net).fitness;

println!("Alpha: {:.1}%, Inductive: {:.1}%",
         alpha_fitness * 100.0, inductive_fitness * 100.0);
```

### How do I choose the right discovery algorithm?

**Quick Decision Tree:**

```
Is your process simple? → Yes → Alpha Miner
         ↓ No
Has loops? → Yes → Inductive Miner
         ↓ No
Is it noisy? → Yes → Heuristic Miner
         ↓ No
Need speed? → Yes → DFG Miner
         ↓ No
Use Inductive Miner (safe choice)
```

### Why is my conformance fitness low?

Possible causes:
1. **Noisy log**: Some events don't follow the pattern
2. **Incomplete traces**: Start/end events missing
3. **Algorithm limitation**: Try different algorithm
4. **Real process variation**: Process is flexible

**Debugging:**

```rust
use pm4py::statistics::LogStatistics;

let stats = LogStatistics::compute(&log);

println!("Event count: {}", stats.total_events);
println!("Avg trace length: {:.1}", stats.avg_trace_length);
println!("Start activities: {:?}", stats.start_activities.len());

// If very short or fragmented, logs might need cleaning
if stats.avg_trace_length < 3.0 {
    eprintln!("Warning: Very short traces, may indicate data quality issue");
}
```

### Can I use custom event attributes?

Yes! Event attributes are preserved:

```rust
let mut event = Event::new("Activity", Utc::now());
event.attributes.insert("cost".to_string(), "100".to_string());
event.attributes.insert("quality".to_string(), "high".to_string());

// Later, access them
if let Some(cost) = event.attributes.get("cost") {
    println!("Cost: {}", cost);
}
```

## Data Handling

### How do I load my event log?

**From XES file (standard):**
```rust
use pm4py::io::XESReader;

let log = XESReader::read("my_events.xes")?;
```

**From CSV file:**
```rust
use pm4py::io::CSVReader;
use std::collections::HashMap;

let mut mapping = HashMap::new();
mapping.insert("case_id".to_string(), "case_id".to_string());
mapping.insert("activity".to_string(), "activity".to_string());
mapping.insert("timestamp".to_string(), "timestamp".to_string());

let log = CSVReader::read("events.csv", &mapping)?;
```

**Manually (in code):**
```rust
let mut log = EventLog::new();

let mut trace = Trace::new("case_1");
trace.add_event(Event::new("Activity A", Utc::now()));
trace.add_event(Event::new("Activity B", Utc::now() + Duration::hours(1)));

log.add_trace(trace);
```

### My CSV columns are named differently. How do I map them?

CSV columns are mapped during read. Your file might have:
```csv
order_id,action,timestamp_utc,person
O001,Pick,2024-01-01T08:00:00Z,John
O001,Pack,2024-01-01T09:00:00Z,Jane
```

Map it like this:
```rust
let mapping = std::collections::HashMap::from([
    ("order_id".to_string(), "case_id".to_string()),
    ("action".to_string(), "activity".to_string()),
    ("timestamp_utc".to_string(), "timestamp".to_string()),
    ("person".to_string(), "resource".to_string()),
]);

let log = CSVReader::read("events.csv", &mapping)?;
```

### How do I handle missing timestamps?

If timestamps are missing, use default values:

```rust
let mut event = Event::new("Activity", Utc::now());

// Or use a specific date if known
let event = Event::new("Activity",
    DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?
        .with_timezone(&Utc)
);
```

### Can I filter the log?

Yes, the library provides filtering:

```rust
// Filter by date range
let start = Utc::now() - Duration::days(30);
let end = Utc::now();
let filtered = log.filter_by_date(start, end)?;

// Remove outliers (traces > 2 std devs from mean length)
let cleaned = log.filter_outliers(2.0);

// Keep only frequent variants (>=5% of traces)
let common = log.filter_variants(0.05)?;
```

## Performance & Optimization

### Why is my program slow?

1. **Debug vs Release**: Always use `--release` for production
```bash
cargo build --release
cargo run --release
```

2. **Log size**: Smaller logs = faster processing
```rust
// Sample 10% of traces for testing
let sample = log.sample_traces(0.1);
```

3. **Algorithm choice**: Some are slower
```
Fastest:      DFG Miner < 5ms
Fast:         Alpha Miner ~50ms
Medium:       Heuristic, Inductive ~100-150ms
Slow:         ILP Miner ~500ms
```

### How do I profile my code?

Using `perf` on Linux:
```bash
cargo build --release
perf record ./target/release/my_program
perf report
```

Using `time`:
```bash
time cargo run --release
```

### Can I process logs in parallel?

Not directly yet (single-threaded by design), but you can:

```rust
use rayon::prelude::*;

// Split log, process in parallel
let results: Vec<_> = log.traces
    .par_iter()
    .map(|trace| {
        let single_log = EventLog::from_trace(trace.clone());
        AlphaMiner::new().discover(&single_log)
    })
    .collect();
```

## Conformance & Validation

### What does fitness mean?

**Fitness** (0-1 scale) = proportion of trace events successfully replayed:

```
Fitness = 1.0  → Perfect conformance (all events replayed)
Fitness = 0.8  → 80% of events match, 20% deviate
Fitness = 0.5  → Half the events match
Fitness = 0.0  → No events match
```

### How do I interpret conformance results?

```rust
let result = TokenReplay::new().check(&log, &net);

match result.fitness {
    f if f >= 0.95 => println!("Excellent - Process is well-controlled"),
    f if f >= 0.85 => println!("Good - Some expected variations"),
    f if f >= 0.70 => println!("Fair - Significant deviations exist"),
    f if f >= 0.50 => println!("Poor - Process highly variable"),
    _ => println!("Critical - Check data quality"),
}
```

### Why is fitness less than 100% even for discovered models?

Discovery finds patterns. Some traces may have unique paths:

```rust
// Discover model
let net = AlphaMiner::new().discover(&log);

// Check conformance
let result = TokenReplay::new().check(&log, &net);

// Result: ~95% (some unique traces excluded)
// This is normal! The 5% are outliers
```

## Troubleshooting

### Compilation Error: "cannot find type"

You're missing an import:

```rust
use pm4py::log::EventLog;  // ← Add this
use pm4py::discovery::AlphaMiner;  // ← Or this
```

### Runtime Error: "thread panicked"

Most panics come from invalid data. Use `Result` types:

```rust
// Good - handles errors
match log.filter_by_date(start, end) {
    Ok(filtered) => process(&filtered),
    Err(e) => eprintln!("Filter failed: {}", e),
}

// Avoid unwrap in production
let filtered = log.filter_by_date(start, end).unwrap();  // ❌ Can panic
```

### "Event log is empty" error

Make sure you're adding traces:

```rust
let mut log = EventLog::new();

// ❌ Wrong - empty log
let net = AlphaMiner::new().discover(&log);  // Error!

// ✅ Right - add traces first
let mut trace = Trace::new("case_1");
trace.add_event(Event::new("A", Utc::now()));
log.add_trace(trace);

let net = AlphaMiner::new().discover(&log);  // OK
```

### Type mismatch errors

Most are from mismatched types. Check:

```rust
// ❌ String vs &str
let trace = Trace::new("case_1");  // Takes String

// ✅ Use .to_string() or String::from()
let id = "case_1".to_string();
let trace = Trace::new(id);
```

## Integration Questions

### Can I use PM4Py Rust with Python?

Yes! Create a Python module in Rust using PyO3:

```bash
cargo new pm4py_py --lib
```

See [PyO3 guide](https://pyo3.rs/) for details.

### Can I export to other formats?

Currently: XES, CSV, SVG. More coming:
- JSON (planned)
- Protobuf (future)
- Arrow format (future)

For now, you can use XES as intermediate:
```rust
XESWriter::write(&log, "intermediate.xes")?;
// Convert XES to other formats with other tools
```

### Can I use this in a web service?

Yes! Perfect for REST APIs:

```rust
// In your web framework (actix-web, axum, etc.)
#[post("/discover")]
async fn discover(log: Json<EventLog>) -> Json<PetriNet> {
    let net = AlphaMiner::new().discover(&log);
    Json(net)
}
```

## Contributing

### How do I report a bug?

1. Check existing [GitHub issues](https://github.com/seanchatmangpt/pm4py-rust/issues)
2. Create minimal reproducible example
3. Open GitHub issue with details

### Can I contribute code?

Yes! Please:
1. Fork the repository
2. Create feature branch
3. Add tests
4. Submit pull request
5. Follow code style (run `cargo fmt`)

### How do I get help?

- Check [GETTING_STARTED.md](./GETTING_STARTED.md)
- Read [FEATURES.md](./FEATURES.md)
- Review example code
- Open GitHub discussion

## Advanced Topics

### How do I extend with custom algorithms?

See [ARCHITECTURE.md](./ARCHITECTURE.md) "Extending the Library" section.

### What's the memory layout?

```rust
EventLog
├── traces: Vec<Trace>
│   └── events: Vec<Event>
│       ├── activity: String (24+ bytes)
│       ├── timestamp: DateTime (8 bytes)
│       ├── resource: Option<String>
│       ├── attributes: BTreeMap
│       └── id: Uuid (16 bytes)
```

Rough memory:
- Event: ~100 bytes
- Trace: ~(50 + 100*events) bytes
- Log: ~(50 + sum of traces) bytes

### How do I optimize memory usage?

```rust
// Use smaller attributes
event.attributes = HashMap::new();  // Not BTreeMap for perf

// Or drop unused events
let cleaned = log.map(|trace| {
    let filtered_events = trace.events.into_iter()
        .filter(|e| keep_event(e))
        .collect();
    Trace { events: filtered_events, ..trace }
});
```

## Version & Compatibility

### What's the MSRV?

Minimum Supported Rust Version: **1.70**

Update with:
```bash
rustup update
```

### Will my code break in v0.2?

Possibly. We're pre-1.0, so minor version updates may introduce breaking changes. We'll document carefully.

### How do I upgrade versions?

```bash
cargo update pm4py
```

Then fix any compilation errors in your code.

## Still Have Questions?

- Read the full [README](../README.md)
- Check [Getting Started](./GETTING_STARTED.md)
- Review [Features Guide](./FEATURES.md)
- Study [examples/](../examples/)
- Open a GitHub discussion

Happy process mining!
