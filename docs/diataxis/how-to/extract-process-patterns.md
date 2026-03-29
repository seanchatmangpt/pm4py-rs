# How-To: Extract Process Patterns

**Problem:** You have a process log but need to find the common sequences of activities that occur.

This guide shows how to extract patterns from a log, identify the most frequent ones, and export them to a file for further analysis.

## What Are Process Patterns?

A process pattern is a sequence of activities that frequently occurs in your event log. For example:
- Request → Approval → Execution (common path)
- Request → Rejection (less common path)

Patterns help you understand:
- Which sequences of work are most common
- How many cases follow each pattern
- What percentage of your process follows standard flows

## Solution: Extract and Export Patterns

### Step 1: Load Your Log

```rust
use pm4py::io::XESReader;
use pm4py::discovery::VariantAnalysis;
use std::path::Path;

let reader = XESReader::new();
let log = reader.read(Path::new("process_log.xes"))?;
```

### Step 2: Discover All Patterns (Variants)

```rust
let analysis = VariantAnalysis::discover(&log);

println!("Found {} unique patterns", analysis.unique_variants);
for variant in &analysis.variants {
    println!("{}: {} cases",
        variant.variant.to_string(),
        variant.frequency
    );
}
```

### Step 3: Filter by Frequency Threshold

Keep only patterns that occur in at least 5% of cases:

```rust
let min_coverage = 0.05;  // 5%
let threshold_frequency = (log.len() as f64 * min_coverage).ceil() as usize;

let frequent_variants: Vec<_> = analysis.variants
    .iter()
    .filter(|v| v.frequency >= threshold_frequency)
    .collect();

println!("Frequent patterns (≥5%): {}", frequent_variants.len());
```

### Step 4: Export to CSV

Create a CSV file with pattern details:

```rust
use std::fs::File;
use std::io::Write;

let mut file = File::create("patterns.csv")?;
writeln!(file, "Pattern,Frequency,Coverage_Percent,Case_IDs")?;

for variant in &frequent_variants {
    let coverage = variant.coverage_percentage(log.len());
    let case_ids = variant.trace_ids.join("|");
    writeln!(file,
        "\"{}\",{},{:.1},\"{}\"",
        variant.variant.to_string(),
        variant.frequency,
        coverage,
        case_ids
    )?;
}

println!("✓ Exported to patterns.csv");
```

### Step 5: Analyze Pattern Statistics

Calculate metrics for each pattern:

```rust
for variant in &analysis.variants.iter().take(5) {  // Top 5 patterns
    let coverage = variant.coverage_percentage(log.len());

    // Calculate average time per pattern (if log has timestamps)
    let durations: Vec<f64> = variant.trace_ids.iter()
        .filter_map(|trace_id| {
            log.traces.iter()
                .find(|t| &t.id == trace_id)
                .and_then(|t| {
                    if t.events.len() >= 2 {
                        let first = t.events.first()?;
                        let last = t.events.last()?;
                        let duration_hours =
                            (last.timestamp - first.timestamp).num_hours() as f64;
                        Some(duration_hours)
                    } else {
                        None
                    }
                })
        })
        .collect();

    let avg_duration = if !durations.is_empty() {
        durations.iter().sum::<f64>() / durations.len() as f64
    } else {
        0.0
    };

    println!("Pattern: {}", variant.variant.to_string());
    println!("  Frequency: {} ({:.1}%)", variant.frequency, coverage);
    println!("  Avg Duration: {:.1} hours", avg_duration);
    println!();
}
```

## Complete Example

```rust
use pm4py::io::XESReader;
use pm4py::discovery::VariantAnalysis;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load log
    let reader = XESReader::new();
    let log = reader.read(Path::new("process_log.xes"))?;

    // Discover patterns
    let analysis = VariantAnalysis::discover(&log);
    println!("Total patterns found: {}", analysis.unique_variants);

    // Filter frequent (>5% coverage)
    let min_coverage = 0.05;
    let threshold = (log.len() as f64 * min_coverage).ceil() as usize;

    let frequent: Vec<_> = analysis.variants
        .iter()
        .filter(|v| v.frequency >= threshold)
        .collect();

    println!("Frequent patterns: {}", frequent.len());

    // Export to CSV
    let mut file = File::create("patterns.csv")?;
    writeln!(file, "Rank,Pattern,Cases,Percent,Activities")?;

    for (rank, variant) in frequent.iter().enumerate() {
        let coverage = variant.coverage_percentage(log.len());
        let pattern_str = variant.variant.to_string();
        let num_activities = variant.variant.len();

        writeln!(file,
            "{},\"{}\",{},{:.1},{}",
            rank + 1,
            pattern_str,
            variant.frequency,
            coverage,
            num_activities
        )?;
    }

    println!("✓ Results exported to patterns.csv");
    Ok(())
}
```

## CSV Output Example

```csv
Rank,Pattern,Cases,Percent,Activities
1,"Register,Approve,Execute,Close",450,45.0,4
2,"Register,Reject,Appeal,Approve,Execute,Close",180,18.0,6
3,"Register,Approve,Execute,Revise,Execute,Close",120,12.0,6
4,"Register,Cancel,Close",100,10.0,3
5,"Register,Approve,Execute,Escalate,Approve,Execute,Close",80,8.0,7
```

## Advanced: Filter by Activity Pattern

Find patterns containing a specific activity:

```rust
// Patterns containing "Escalate"
let escalation_patterns: Vec<_> = analysis.variants
    .iter()
    .filter(|v| v.variant.activities.contains(&"Escalate".to_string()))
    .collect();

println!("Patterns with escalation: {}", escalation_patterns.len());
for variant in escalation_patterns {
    println!("  {} ({} cases)",
        variant.variant.to_string(),
        variant.frequency
    );
}
```

## Advanced: Pattern Similarity

Find patterns similar to a target pattern:

```rust
use pm4py::discovery::VariantSimilarity;

let target_pattern = "Register,Approve,Execute,Close";
let similarity_threshold = 0.7;  // 70% similar

for variant in &analysis.variants {
    let pattern_str = variant.variant.to_string();
    let similarity = VariantSimilarity::edit_distance_ratio(
        target_pattern,
        &pattern_str
    );

    if similarity >= similarity_threshold {
        println!("Similar pattern: {} ({}% match)",
            pattern_str,
            (similarity * 100.0) as i32
        );
    }
}
```

## Performance Considerations

| Log Size | Typical Time | Memory |
|----------|-------------|--------|
| 1K cases | <10ms | 1 MB |
| 10K cases | ~50ms | 10 MB |
| 100K cases | ~200ms | 100 MB |
| 1M+ cases | <1s | 1 GB |

For larger logs, filter first:

```rust
// Keep only recent traces
let cutoff_date = Utc::now() - Duration::days(30);
let filtered_log = log.filter_by_time_range(cutoff_date, Utc::now())?;

// Then discover patterns
let analysis = VariantAnalysis::discover(&filtered_log);
```

## Troubleshooting

**Issue: Too many patterns (100+)**
- Increase the coverage threshold from 5% to 10%
- Or filter to top K patterns: `analysis.variants.iter().take(20)`

**Issue: Pattern string too long**
- Abbreviate activity names: "Reg,App,Exe,Cls" instead of full names
- Or split patterns by depth: show only first 5 activities

**Issue: Exported CSV file is very large**
- Reduce number of Case IDs: `case_ids.split('|').take(10).collect()`
- Or write Case IDs to a separate file

## See Also

- **Tutorial:** First Process Mining Analysis
- **Reference:** Rust API Documentation
- **How-To:** Discover Petri Net Models

---

**Time to complete:** ~20 minutes | **Difficulty:** Intermediate
