# Tutorial: Analyze Your First Process

**Goal:** Load a process log and extract statistics in <15 minutes

This tutorial walks you through loading an event log in XES format and discovering process statistics. By the end, you'll have a working Rust program that mines processes.

## Prerequisites

- Rust 1.70 or later
- A sample XES file (we'll show you how to create one)

## Step 1: Create a Sample XES Log File

First, create a file named `hospital.xes` with sample hospital process data:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="nested-attributes" openlog.version="1.0">
  <trace>
    <string key="concept:name" value="case_001"/>
    <event>
      <string key="concept:name" value="Registration"/>
      <date key="time:timestamp" value="2026-01-01T08:00:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Doctor_Consultation"/>
      <date key="time:timestamp" value="2026-01-01T08:30:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Treatment"/>
      <date key="time:timestamp" value="2026-01-01T09:00:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Discharge"/>
      <date key="time:timestamp" value="2026-01-01T10:00:00Z"/>
    </event>
  </trace>
  <trace>
    <string key="concept:name" value="case_002"/>
    <event>
      <string key="concept:name" value="Registration"/>
      <date key="time:timestamp" value="2026-01-02T09:00:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Doctor_Consultation"/>
      <date key="time:timestamp" value="2026-01-02T09:45:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Lab_Test"/>
      <date key="time:timestamp" value="2026-01-02T10:00:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Treatment"/>
      <date key="time:timestamp" value="2026-01-02T11:00:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Discharge"/>
      <date key="time:timestamp" value="2026-01-02T12:00:00Z"/>
    </event>
  </trace>
  <trace>
    <string key="concept:name" value="case_003"/>
    <event>
      <string key="concept:name" value="Registration"/>
      <date key="time:timestamp" value="2026-01-03T07:00:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Doctor_Consultation"/>
      <date key="time:timestamp" value="2026-01-03T07:30:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Treatment"/>
      <date key="time:timestamp" value="2026-01-03T08:00:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="Discharge"/>
      <date key="time:timestamp" value="2026-01-03T09:00:00Z"/>
    </event>
  </trace>
</log>
```

Save this in your project directory at `hospital.xes`.

## Step 2: Add pm4py to Cargo.toml

Edit your `Cargo.toml` and ensure pm4py is a dependency:

```toml
[dependencies]
pm4py = "0.3"
chrono = { version = "0.4", features = ["serde"] }
```

## Step 3: Write the Analysis Code

Create a file `src/main.rs` with the following code:

```rust
use pm4py::io::XESReader;
use pm4py::discovery::VariantAnalysis;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read the XES file
    println!("Loading hospital.xes...");
    let reader = XESReader::new();
    let log = reader.read(Path::new("hospital.xes"))?;

    // Step 2: Print basic statistics
    println!("\n=== Log Statistics ===");
    println!("Total traces (cases): {}", log.len());
    println!("Total events: {}", log.num_events());
    println!("Average trace length: {:.1}",
        log.num_events() as f64 / log.len() as f64);

    // Step 3: List all unique activities
    let activities = log.activities();
    println!("\n=== Activities Found ===");
    for (i, activity) in activities.iter().enumerate() {
        println!("  {}. {}", i + 1, activity);
    }
    println!("Total activities: {}", activities.len());

    // Step 4: Discover process variants
    let analysis = VariantAnalysis::discover(&log);
    println!("\n=== Process Variants ===");
    println!("Unique variants: {}", analysis.unique_variants);
    println!();

    for (i, variant) in analysis.variants.iter().enumerate() {
        let coverage = variant.coverage_percentage(log.len());
        println!("Variant {}", i + 1);
        println!("  Sequence: {}", variant.variant.to_string());
        println!("  Frequency: {} cases ({:.1}%)", variant.frequency, coverage);
        println!("  Case IDs: {}", variant.trace_ids.join(", "));
        println!();
    }

    println!("✓ Analysis complete!");
    Ok(())
}
```

## Step 4: Compile and Run

```bash
cargo run --release
```

### Expected Output

```
Loading hospital.xes...

=== Log Statistics ===
Total traces (cases): 3
Total events: 12
Average trace length: 4.0

=== Activities Found ===
  1. Discharge
  2. Doctor_Consultation
  3. Lab_Test
  4. Registration
  5. Treatment
Total activities: 5

=== Process Variants ===
Unique variants: 2

Variant 1
  Sequence: Registration,Doctor_Consultation,Treatment,Discharge
  Frequency: 2 cases (66.7%)
  Case IDs: case_001, case_003

Variant 2
  Sequence: Registration,Doctor_Consultation,Lab_Test,Treatment,Discharge
  Frequency: 1 cases (33.3%)
  Case IDs: case_002

✓ Analysis complete!
```

## Understanding the Output

- **Traces**: Each trace is one case (patient admission) in the hospital
- **Events**: Total number of activities across all cases
- **Activities**: The distinct types of work performed (Registration, Treatment, etc.)
- **Variants**: Different sequence patterns. Variant 1 is the most common path (2/3 cases)

## Next Steps

1. **Export variants to CSV** — See the How-To guide for extracting patterns to a file
2. **Discover a process model** — Use `AlphaMiner` to generate a Petri net (flow diagram)
3. **Analyze performance** — Calculate average time between activities per variant
4. **Large logs** — Try with your own XES file (handles logs with millions of events)

## Troubleshooting

**Error: "hospital.xes" not found**
- Make sure the file is in your current working directory, or adjust the path in the code

**Error: Invalid timestamp format**
- Timestamps must be ISO8601 format: `2026-01-01T08:00:00Z`

**Error: No activities found**
- Check that events have a `concept:name` attribute with the activity label

## Key Concepts

- **XES Format**: Standard XML format for event logs. Must have traces → events → attributes
- **Variant**: A unique sequence of activities. Case_001 and case_003 follow the same variant
- **Coverage**: Percentage of total cases that follow this variant (2/3 = 66.7%)

---

**Time to complete:** ~10 minutes | **Difficulty:** Beginner
