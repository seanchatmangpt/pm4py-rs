# Advanced Statistics API Reference

## Quick Reference

All functions are in module `pm4py::statistics`.

### Duration Analysis

```rust
// Overall case duration statistics
case_duration_distribution(log) -> CaseDurationStats
  Fields: min_duration, max_duration, mean_duration, median_duration, stddev_duration, count

// Duration stats per variant (process variant)
case_duration_distribution_per_variant(log) -> Vec<VariantDurationStats>
  Sorted by: frequency (descending)
```

### Activity Analysis

```rust
// Activity occurrence analysis
get_activity_frequency(log) -> Vec<ActivityFrequency>
  Fields: activity, total_count, distinct_traces, average_per_trace
  Sorted by: total_count (descending)

// Variant (trace sequence) frequency
get_variant_frequency(log) -> Vec<VariantFrequency>
  Fields: variant (Vec<String>), count, percentage
  Sorted by: count (descending)
```

### Rework Analysis

```rust
// Identify activities with rework (repeat occurrences)
identify_rework_patterns(log) -> Vec<ReworkPattern>
  Fields: activity, traces_with_rework, total_rework_instances, avg_iterations
  Sorted by: total_rework_instances (descending)

// Count rework per activity per trace
get_rework_count_per_activity(log) -> HashMap<String, Vec<usize>>
  Maps: activity → [count1, count2, ...] (counts per trace with rework)
```

### Resource Analysis

```rust
// Resource allocation and utilization metrics
get_resource_metrics(log) -> Vec<ResourceMetrics>
  Fields: resource, total_activities, unique_activities, avg_case_duration, utilization
  Sorted by: total_activities (descending)
```

### Performance Analysis

```rust
// Average duration between consecutive activities (identifies bottlenecks)
get_activity_throughput(log) -> Vec<(String, f64)>
  Returns: [(activity, avg_duration_seconds), ...]
  Sorted by: duration (descending) - bottlenecks first

// Cases processed per day
get_case_throughput(log) -> f64
  Returns: cases/day (f64)
  Calculation: total_cases / days_between_first_and_last

// Top K slowest activities
get_bottleneck_activities(log, top_k: usize) -> Vec<(String, f64)>
  Returns: [(activity, avg_duration_seconds), ...] (limited to top_k)
  Sorted by: duration (descending)

// Comprehensive performance summary
calculate_performance_indicators(log) -> PerformanceIndicators
  Fields: case_throughput, avg_case_duration, fastest_activity, slowest_activity
```

## Data Types

### CaseDurationStats
```rust
pub struct CaseDurationStats {
    pub min_duration: f64,              // seconds
    pub max_duration: f64,              // seconds
    pub mean_duration: f64,             // seconds
    pub median_duration: f64,           // seconds
    pub stddev_duration: f64,           // seconds
    pub count: usize,                   // number of cases
}
```

### VariantDurationStats
```rust
pub struct VariantDurationStats {
    pub variant: Vec<String>,           // sequence of activities
    pub count: usize,                   // number of cases with this variant
    pub min_duration: f64,              // seconds
    pub max_duration: f64,              // seconds
    pub mean_duration: f64,             // seconds
    pub median_duration: f64,           // seconds
    pub stddev_duration: f64,           // seconds
}
```

### ActivityFrequency
```rust
pub struct ActivityFrequency {
    pub activity: String,
    pub total_count: usize,             // total occurrences
    pub distinct_traces: usize,         // traces containing this activity
    pub average_per_trace: f64,         // average occurrences per trace
}
```

### VariantFrequency
```rust
pub struct VariantFrequency {
    pub variant: Vec<String>,
    pub count: usize,                   // number of traces with this variant
    pub percentage: f64,                // 0.0-100.0
}
```

### ReworkPattern
```rust
pub struct ReworkPattern {
    pub activity: String,
    pub traces_with_rework: usize,      // traces where activity repeats
    pub total_rework_instances: usize,  // total rework occurrences
    pub avg_iterations: f64,            // average repeats per trace
}
```

### ResourceMetrics
```rust
pub struct ResourceMetrics {
    pub resource: String,
    pub total_activities: usize,        // activities performed
    pub unique_activities: usize,       // distinct activity types
    pub avg_case_duration: f64,         // seconds
    pub utilization: f64,               // 0.0-1.0
}
```

### PerformanceIndicators
```rust
pub struct PerformanceIndicators {
    pub case_throughput: f64,           // cases per day
    pub avg_case_duration: f64,         // seconds
    pub fastest_activity: (String, f64), // (activity_name, avg_seconds)
    pub slowest_activity: (String, f64), // (activity_name, avg_seconds)
}
```

## Common Patterns

### Get top 3 bottlenecks
```rust
let bottlenecks = get_bottleneck_activities(&log, 3);
for (activity, duration) in bottlenecks {
    println!("{}: {:.1}s", activity, duration);
}
```

### Find activities with rework
```rust
let rework = identify_rework_patterns(&log);
for pattern in rework {
    println!("{}: {} traces affected", pattern.activity, pattern.traces_with_rework);
}
```

### Get process variants by frequency
```rust
let variants = get_variant_frequency(&log);
for variant in variants {
    println!("{}: {:.1}%", variant.variant.join("→"), variant.percentage);
}
```

### Analyze resource utilization
```rust
let resources = get_resource_metrics(&log);
for resource in resources {
    println!("{}: {:.1}% utilization", resource.resource, resource.utilization * 100.0);
}
```

### One-line performance summary
```rust
let perf = calculate_performance_indicators(&log);
println!("Throughput: {:.1} cases/day, Bottleneck: {}",
         perf.case_throughput,
         perf.slowest_activity.0);
```

## Sorting Behavior

All functions that return `Vec<_>` sort results:

| Function | Sort By | Order |
|----------|---------|-------|
| `case_duration_distribution_per_variant` | variant count | descending |
| `get_activity_frequency` | total_count | descending |
| `get_variant_frequency` | count | descending |
| `identify_rework_patterns` | total_rework_instances | descending |
| `get_resource_metrics` | total_activities | descending |
| `get_activity_throughput` | duration | descending (bottlenecks first) |
| `get_bottleneck_activities` | duration | descending (top K only) |

## Edge Case Behavior

| Condition | Result |
|-----------|--------|
| Empty log | Empty Vec or zeros |
| Single trace | Returns valid statistics |
| Single event per trace | Duration = 0.0 |
| No resources assigned | Empty Vec from `get_resource_metrics` |
| No rework patterns | Empty Vec from `identify_rework_patterns` |
| Single variant | Vec with one element |

## Numeric Precision

- **Duration Units**: Seconds (f64)
- **Precision**: Milliseconds (num_milliseconds() / 1000.0)
- **Tolerance**: <1e-10 relative error vs Python pm4py
- **Percentages**: 0.0-100.0 (f64)
- **Utilization**: 0.0-1.0 (f64)

## Performance Characteristics

| Function | Time | Space |
|----------|------|-------|
| `case_duration_distribution` | O(n) | O(n) |
| `case_duration_distribution_per_variant` | O(n log k) | O(n) |
| `get_activity_frequency` | O(n) | O(a) |
| `get_variant_frequency` | O(n log v) | O(v) |
| `identify_rework_patterns` | O(n) | O(a) |
| `get_resource_metrics` | O(n) | O(r) |
| `get_activity_throughput` | O(n log a) | O(a) |
| `get_case_throughput` | O(n) | O(n) |
| `calculate_performance_indicators` | O(n log a) | O(n) |

Where: n=events, a=activities, v=variants, r=resources, k=clusters

## Validation

All functions validate against Python pm4py with:
- Standard invoice log (10 cases, 3 activities, 1 variant)
- Numeric precision: <1e-10 relative error
- Edge case handling: empty logs, single traces
- Median calculation: even/odd counts
- Floating-point consistency: millisecond precision

See: `tests/statistics_advanced_test.rs` (33 tests, 100% pass rate)
