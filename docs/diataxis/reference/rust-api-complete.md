# Reference: pm4py-rust API Complete

Complete API reference for the pm4py-rust process mining library. This document lists all public types, methods, and traits.

## Core Types

### Event

Represents a single activity occurrence in a trace.

```rust
pub struct Event {
    pub activity: String,
    pub timestamp: DateTime<Utc>,
    pub resource: Option<String>,
    pub attributes: BTreeMap<String, String>,
    pub id: Uuid,
}
```

**Constructor:**
```rust
Event::new(activity: impl Into<String>, timestamp: DateTime<Utc>) -> Self
```

**Methods:**
| Method | Input | Output | Purpose |
|--------|-------|--------|---------|
| `with_resource()` | resource: String | Event | Set the resource performing the activity |
| `with_attribute()` | key: String, value: String | Event | Add custom attribute |
| `get_attribute()` | key: &str | Option<&str> | Retrieve attribute value |

**Example:**
```rust
use chrono::Utc;
let event = Event::new("Order", Utc::now())
    .with_resource("Employee_A")
    .with_attribute("department", "sales");
```

---

### Trace

Represents a sequence of events for one process instance (case).

```rust
pub struct Trace {
    pub id: String,
    pub events: Vec<Event>,
    pub attributes: BTreeMap<String, String>,
}
```

**Constructor:**
```rust
Trace::new(id: impl Into<String>) -> Self
```

**Methods:**
| Method | Input | Output | Purpose |
|--------|-------|--------|---------|
| `add_event()` | event: Event | () | Append event to trace |
| `len()` | () | usize | Number of events |
| `is_empty()` | () | bool | Check if trace has events |
| `events_sorted()` | () | Vec<&Event> | Get events ordered by timestamp |
| `with_attribute()` | key: String, value: String | Trace | Add trace-level attribute |
| `get_attribute()` | key: &str | Option<&str> | Get trace attribute value |

**Example:**
```rust
let mut trace = Trace::new("case_001");
trace.add_event(Event::new("Order", Utc::now()));
trace.add_event(Event::new("Payment", Utc::now() + Duration::hours(1)));
println!("Trace has {} events", trace.len());
```

**Performance:**
- `add_event()`: O(1) amortized
- `len()`: O(1)
- `events_sorted()`: O(n log n) where n = number of events

---

### EventLog

Represents a complete event log (collection of traces).

```rust
pub struct EventLog {
    pub traces: Vec<Trace>,
    pub attributes: BTreeMap<String, String>,
}
```

**Constructor:**
```rust
EventLog::new() -> Self
```

**Methods:**
| Method | Input | Output | Purpose |
|--------|-------|--------|---------|
| `add_trace()` | trace: Trace | () | Add trace to log |
| `len()` | () | usize | Number of traces |
| `is_empty()` | () | bool | Check if log has traces |
| `num_events()` | () | usize | Total number of events across all traces |
| `activities()` | () | Vec<String> | Get unique activity names (sorted) |
| `with_attribute()` | key: String, value: String | EventLog | Add log-level attribute |
| `get_attribute()` | key: &str | Option<&str> | Get log attribute |

**Example:**
```rust
let mut log = EventLog::new();
log.add_trace(trace1);
log.add_trace(trace2);
println!("Log has {} traces and {} events",
    log.len(),
    log.num_events()
);

// Get all activities in the log
let activities = log.activities();
for activity in activities {
    println!("- {}", activity);
}
```

**Performance:**
- `add_trace()`: O(1) amortized
- `len()`: O(1)
- `num_events()`: O(n) where n = number of traces
- `activities()`: O(n + m log m) where n = events, m = unique activities

---

## I/O: Loading and Saving Logs

### XESReader

Reads event logs from XES (eXtensible Event Stream) XML files.

```rust
pub struct XESReader;
```

**Methods:**
```rust
impl XESReader {
    pub fn new() -> Self
    pub fn read(&self, path: &Path) -> Result<EventLog>
}
```

**Example:**
```rust
use pm4py::io::XESReader;
use std::path::Path;

let reader = XESReader::new();
let log = reader.read(Path::new("process.xes"))?;
```

**Supported Format:**
- XML standard with traces → events → attributes
- Standard attributes: `concept:name` (activity), `time:timestamp` (ISO8601)
- Custom attributes via `<string key="..." value="..."/>` tags
- Security: XXE (XML External Entity) attacks prevented

**Error Handling:**
| Error | Cause | Solution |
|-------|-------|----------|
| File not found | Path incorrect | Check file exists |
| XML parsing error | Invalid XML | Validate with `xmllint` |
| No traces found | Wrong format | Check structure has `<trace>` elements |
| Invalid timestamp | Not ISO8601 | Use format `2026-01-01T12:00:00Z` |

---

### XESWriter

Writes event logs to XES format.

```rust
pub struct XESWriter;
```

**Methods:**
```rust
impl XESWriter {
    pub fn new() -> Self
    pub fn write(&self, log: &EventLog, path: &Path) -> Result<()>
}
```

**Example:**
```rust
use pm4py::io::XESWriter;
use std::path::Path;

let writer = XESWriter::new();
writer.write(&log, Path::new("output.xes"))?;
```

---

## Discovery: Process Mining Algorithms

### Variant Analysis

Discover unique patterns (activity sequences) in the log.

```rust
pub struct VariantAnalysis {
    pub variants: Vec<VariantInfo>,
    pub total_traces: usize,
    pub unique_variants: usize,
}

pub struct Variant {
    pub activities: Vec<String>,
}

pub struct VariantInfo {
    pub variant: Variant,
    pub fingerprint: VariantFingerprint,
    pub frequency: usize,
    pub trace_ids: Vec<String>,
}
```

**Methods:**
```rust
impl VariantAnalysis {
    /// Discover all variants in log
    pub fn discover(log: &EventLog) -> Self
}

impl VariantInfo {
    /// Get percentage of total traces
    pub fn coverage_percentage(&self, total_traces: usize) -> f64
}

impl Variant {
    /// String representation (comma-separated)
    pub fn to_string(&self) -> String
    pub fn len(&self) -> usize
}

impl VariantFingerprint {
    /// Deterministic hash for variant
    pub fn compute(variant: &Variant) -> Self
    /// Convert hash to hex string
    pub fn to_hex(&self) -> String
}
```

**Example:**
```rust
use pm4py::discovery::VariantAnalysis;

let analysis = VariantAnalysis::discover(&log);

println!("Found {} unique variants", analysis.unique_variants);
for variant in &analysis.variants {
    let coverage = variant.coverage_percentage(log.len());
    println!("{}: {} cases ({:.1}%)",
        variant.variant.to_string(),
        variant.frequency,
        coverage
    );
}
```

**Performance:**
- Time: O(n) where n = number of traces
- Space: O(m) where m = number of unique variants
- Typical: <100ms for 10K traces, <1s for 1M traces

---

### AlphaMiner

Discovers Petri nets using the Alpha Miner algorithm.

```rust
pub struct AlphaMiner;

pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
}
```

**Methods:**
```rust
impl AlphaMiner {
    pub fn new() -> Self
    pub fn discover(&self, log: &EventLog) -> PetriNet
}
```

**Example:**
```rust
use pm4py::discovery::AlphaMiner;

let miner = AlphaMiner::new();
let net = miner.discover(&log);

println!("Discovered {} places", net.places.len());
println!("Discovered {} transitions", net.transitions.len());
println!("Discovered {} arcs", net.arcs.len());
```

**Characteristics:**
- Best for: Simple, well-structured processes
- Time: O(n²) where n = activities
- Output: Petri net (places, transitions, arcs)
- Limitations: May not discover all complex patterns

---

## Statistics

### Log Statistics

Compute statistics on event logs.

```rust
pub struct LogStatistics {
    pub num_traces: usize,
    pub num_events: usize,
    pub num_activities: usize,
    pub avg_trace_length: f64,
}
```

**Methods:**
```rust
impl EventLog {
    /// Get comprehensive statistics
    pub fn statistics(&self) -> LogStatistics
}
```

**Example:**
```rust
let stats = log.statistics();
println!("Traces: {}", stats.num_traces);
println!("Events: {}", stats.num_events);
println!("Activities: {}", stats.num_activities);
println!("Avg length: {:.1}", stats.avg_trace_length);
```

---

## Filtering

### Advanced Filters

Filter logs by various criteria.

```rust
pub trait AdvancedFilter {
    fn apply(&self, log: &EventLog) -> EventLog;
}
```

**Common Filters:**
| Function | Input | Output | Purpose |
|----------|-------|--------|---------|
| `filter_case_size()` | log, min_size, max_size | EventLog | Keep cases with N-M events |
| `filter_activities_rework()` | log, threshold | EventLog | Detect cases with repeated activities |
| `filter_trace_attribute()` | log, key, value | EventLog | Keep traces matching attribute |
| `filter_event_attribute_values()` | log, key, values | EventLog | Keep events with specific attribute values |
| `filter_time_range()` | log, start, end | EventLog | Keep events within date range |

**Example:**
```rust
use pm4py::log::advanced_filters;

// Keep only cases with 3-10 events
let filtered = advanced_filters::filter_case_size(&log, 3, 10);

// Keep traces matching attribute
let dept_traces = advanced_filters::filter_trace_attribute(
    &log,
    "department",
    "sales"
)?;

// Keep events in date range
let recent = advanced_filters::filter_time_range(
    &log,
    start_time,
    end_time
)?;
```

---

## Error Handling

All I/O and discovery operations return `Result<T>` with anyhow::Error.

```rust
use anyhow::Result;

match reader.read(path) {
    Ok(log) => println!("Loaded {} traces", log.len()),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Common Errors:**
- `File not found` — Check path exists
- `XML parse error` — Validate XES format
- `Invalid timestamp` — Use ISO8601 format
- `No traces in log` — Check log has valid structure

---

## Module Organization

```
pm4py
├── log              ← Event, Trace, EventLog, filters
├── discovery        ← Mining algorithms (Alpha, Variants, etc.)
├── io               ← File I/O (XES, CSV, JSON)
├── statistics       ← Log analysis
├── models           ← Petri nets, process trees
├── conformance      ← Model checking
└── performance      ← Timing analysis
```

---

## Common Patterns

### Load → Discover → Export

```rust
use pm4py::io::{XESReader, XESWriter};
use pm4py::discovery::VariantAnalysis;

fn analyze_process(input: &str, output: &str) -> Result<()> {
    let reader = XESReader::new();
    let log = reader.read(Path::new(input))?;

    let analysis = VariantAnalysis::discover(&log);
    println!("Found {} variants", analysis.unique_variants);

    let writer = XESWriter::new();
    writer.write(&log, Path::new(output))?;

    Ok(())
}
```

### Filter → Analyze → Report

```rust
use pm4py::log::advanced_filters;
use pm4py::discovery::VariantAnalysis;

fn analyze_recent_cases(log: &EventLog, days: i64) -> Result<()> {
    let cutoff = Utc::now() - Duration::days(days);
    let recent = advanced_filters::filter_time_range(
        log,
        cutoff,
        Utc::now()
    )?;

    let analysis = VariantAnalysis::discover(&recent);
    println!("Recent variants: {}", analysis.unique_variants);

    Ok(())
}
```

---

## Performance Characteristics

| Operation | Time | Space | Typical Case |
|-----------|------|-------|--------------|
| Load XES (100K events) | ~500ms | 50MB | File I/O bound |
| Discover variants | O(n) | O(m) | 100ms for 100K events |
| Alpha Miner | O(a²) | O(a) | 10ms for 50 activities |
| Filter by time | O(n) | O(n') | Linear scan |
| Statistics | O(n) | O(1) | Single pass |

Where:
- n = number of events
- m = number of unique variants
- a = number of activities
- n' = events matching filter

---

## Version Info

```rust
use pm4py::version_string;

println!("pm4py version: {}", version_string());
// Output: "0.3.0"
```

---

## Next Steps

- **Tutorial:** First Process Mining Analysis
- **How-To:** Extract Process Patterns
- **Guide:** XES Format Structure

---

**Last Updated:** 2026-03-25 | **Stability:** Stable (1.0 API)
