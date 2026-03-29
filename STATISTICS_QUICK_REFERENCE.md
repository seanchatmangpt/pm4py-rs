# Statistics Functions Quick Reference

## All 30 Statistics Functions

### ATTRIBUTE STATISTICS (4)
```rust
get_case_attributes(log)                    // Extract attributes per case
get_attribute_value_frequency(log)          // Count (key, value) pairs
get_attribute_co_occurrence(log)            // Find paired attributes
get_attribute_values_by_key(log, key)       // Get values for specific key
```

### RESOURCE/ROLE ANALYTICS (4)
```rust
get_resource_workload(log)                  // Workload per resource
get_resource_availability(log)              // Active time windows
get_resource_collaboration(log)             // Resources working together
get_resource_efficiency(log)                // Performance consistency
```

### TIME-BASED ANALYTICS (4)
```rust
calculate_activity_transition_times(log)    // Time between activities
check_sla_compliance(log, sla_seconds)      // SLA violation checking
get_peak_hours_analysis(log)                // Busiest hours (24 buckets)
get_workload_distribution_by_day(log)       // Workload by day (7 buckets)
```

### ADVANCED PROCESS METRICS (4)
```rust
calculate_process_efficiency(log, ideal)    // Actual vs ideal time
calculate_wait_processing_ratio(log)        // Wait time analysis
calculate_case_complexity(log)              // Complexity scoring
calculate_case_resource_efficiency(log)     // Resource utilization per case
```

### DEVIATION DETECTION (3)
```rust
detect_outlier_cases_by_duration(log, z)    // Unusual durations (z-score)
detect_anomalous_paths(log, threshold)      // Rare process variants
detect_frequency_anomalies(log, z)          // Unusual activity patterns
```

### EXISTING FUNCTIONS (11)
```rust
case_duration_distribution(log)             // Duration statistics
case_duration_distribution_per_variant(log) // Duration per variant
get_activity_frequency(log)                 // Activity occurrence
get_variant_frequency(log)                  // Variant percentages
identify_rework_patterns(log)               // Repeated activities
get_rework_count_per_activity(log)          // Rework per activity
get_resource_metrics(log)                   // Resource analysis
get_activity_throughput(log)                // Activity duration
get_case_throughput(log)                    // Cases per day
get_bottleneck_activities(log, k)           // Top-k slowest
calculate_performance_indicators(log)       // Overall metrics
```

---

## One-Liner Examples

```rust
// Find slowest transitions
let transitions = calculate_activity_transition_times(&log);
println!("Slowest: {} -> {}: {:.1}s",
    transitions[0].from_activity,
    transitions[0].to_activity,
    transitions[0].avg_time_seconds);

// Check SLA compliance
let compliance = check_sla_compliance(&log, Some(86400.0));  // 24 hours
let rate = (compliance.iter().filter(|c| c.is_compliant).count() as f64
    / compliance.len() as f64) * 100.0;
println!("SLA compliance: {:.1}%", rate);

// Find complex cases
let complexity = calculate_case_complexity(&log);
let top_cases = complexity.iter().take(5);
for case in top_cases {
    println!("{}: complexity {:.1}", case.case_id, case.complexity_score);
}

// Detect anomalies
let anomalies = detect_anomalous_paths(&log, Some(0.7));
println!("Found {} anomalous paths", anomalies.len());

// Peak hours
let peaks = get_peak_hours_analysis(&log);
println!("Busiest hour: {:02}:00 ({} events)",
    peaks[0].hour_of_day,
    peaks[0].event_count);

// Resource utilization
let workloads = get_resource_workload(&log);
for res in workloads {
    println!("{}: {:.1}% workload ({} cases)",
        res.resource,
        res.workload_percentage,
        res.case_count);
}
```

---

## Return Type Reference

```rust
// Attribute Statistics
Vec<HashMap<String, String>>                // get_case_attributes
Vec<AttributeFrequency>                     // get_attribute_value_frequency
Vec<AttributeCoOccurrence>                  // get_attribute_co_occurrence
Vec<(String, usize)>                        // get_attribute_values_by_key

// Resource Analytics
Vec<ResourceWorkload>                       // get_resource_workload
Vec<ResourceAvailability>                   // get_resource_availability
Vec<ResourceCollaboration>                  // get_resource_collaboration
Vec<ResourceEfficiency>                     // get_resource_efficiency

// Time-Based
Vec<ActivityTransitionTime>                 // calculate_activity_transition_times
Vec<SLACompliance>                          // check_sla_compliance
Vec<PeakHourAnalysis>                       // get_peak_hours_analysis
Vec<WorkloadDistribution>                   // get_workload_distribution_by_day

// Advanced Metrics
Vec<ProcessEfficiencyMetrics>               // calculate_process_efficiency
Vec<WaitProcessingAnalysis>                 // calculate_wait_processing_ratio
Vec<CaseComplexity>                         // calculate_case_complexity
Vec<CaseResourceEfficiency>                 // calculate_case_resource_efficiency

// Deviation Detection
Vec<OutlierCase>                            // detect_outlier_cases_by_duration
Vec<AnomalousPath>                          // detect_anomalous_paths
Vec<FrequencyAnomaly>                       // detect_frequency_anomalies
```

---

## Performance: O(n) vs O(n log n)

### Linear Time O(n)
```
get_case_attributes
get_resource_availability
check_sla_compliance
get_peak_hours_analysis
get_workload_distribution_by_day
calculate_process_efficiency
calculate_case_resource_efficiency
```

### Log-Linear Time O(n log n)
```
get_attribute_value_frequency
get_resource_workload
get_resource_efficiency
calculate_activity_transition_times
calculate_wait_processing_ratio
calculate_case_complexity
detect_outlier_cases_by_duration
detect_anomalous_paths
detect_frequency_anomalies
```

### Quadratic in Small Sets O(n × r²)
```
get_attribute_co_occurrence           // r = attributes per event (typically 2-5)
get_resource_collaboration            // r = resources per case (typically 2-10)
```

---

## Default Parameters

```rust
check_sla_compliance(log, None)             // Defaults to 24 hours (86400s)
detect_outlier_cases_by_duration(log, None) // Defaults to z > 2.0
detect_anomalous_paths(log, None)           // Defaults to threshold 0.7
detect_frequency_anomalies(log, None)       // Defaults to z > 2.0
```

---

## Use Cases By Scenario

### "I want to find bottlenecks"
```rust
let transitions = calculate_activity_transition_times(&log);
// transitions[0] is slowest
```

### "I want to check SLA compliance"
```rust
let compliance = check_sla_compliance(&log, Some(sla_seconds));
let violations = compliance.iter().filter(|c| !c.is_compliant).count();
```

### "I want to find overworked resources"
```rust
let workload = get_resource_workload(&log);
let overworked = workload.iter()
    .filter(|w| w.workload_percentage > 40.0);
```

### "I want to find inconsistent resources"
```rust
let efficiency = get_resource_efficiency(&log);
let inconsistent = efficiency.iter()
    .filter(|e| e.efficiency_score < 0.5);
```

### "I want to find unusual cases"
```rust
let outliers = detect_outlier_cases_by_duration(&log, Some(2.0));
// All cases in outliers are anomalies
```

### "I want to find rare process paths"
```rust
let anomalies = detect_anomalous_paths(&log, Some(0.7));
// All paths in anomalies are rare
```

### "I want to analyze peak hours"
```rust
let peaks = get_peak_hours_analysis(&log);
let busiest_hour = peaks[0].hour_of_day;
```

### "I want to understand case complexity"
```rust
let complexity = calculate_case_complexity(&log);
let simple_cases = complexity.iter()
    .filter(|c| c.complexity_score < 2.0);
let complex_cases = complexity.iter()
    .filter(|c| c.complexity_score > 5.0);
```

### "I want to analyze resource collaboration"
```rust
let collab = get_resource_collaboration(&log);
for pair in collab.iter().take(5) {
    println!("{} & {} work together {:.1}% of time",
        pair.resource1, pair.resource2, pair.collaboration_percentage);
}
```

### "I want to measure process efficiency"
```rust
let efficiency = calculate_process_efficiency(&log, ideal_seconds);
let avg_waste: f64 = efficiency.iter()
    .map(|e| e.waste_percentage)
    .sum::<f64>() / efficiency.len() as f64;
println!("Average waste: {:.1}%", avg_waste);
```

---

## Data Structure Fields

### AttributeFrequency
- `attribute_key: String` - Key name
- `attribute_value: String` - Value
- `frequency: usize` - Count
- `percentage: f64` - Percentage of total

### ResourceWorkload
- `resource: String` - Resource name
- `event_count: usize` - Total events
- `case_count: usize` - Unique cases
- `avg_events_per_case: f64` - Average
- `workload_percentage: f64` - Percentage

### ActivityTransitionTime
- `from_activity: String` - Source
- `to_activity: String` - Target
- `avg_time_seconds: f64` - Average
- `min_time_seconds: f64` - Minimum
- `max_time_seconds: f64` - Maximum
- `count: usize` - Occurrences

### SLACompliance
- `case_id: String` - Case ID
- `total_duration_seconds: f64` - Actual time
- `sla_threshold_seconds: f64` - SLA limit
- `is_compliant: bool` - Pass/fail
- `variance_seconds: f64` - Difference

### CaseComplexity
- `case_id: String` - Case ID
- `num_events: usize` - Event count
- `num_unique_activities: usize` - Activity diversity
- `num_rework_instances: usize` - Rework count
- `complexity_score: f64` - Weighted score

### OutlierCase
- `case_id: String` - Case ID
- `metric_name: String` - "case_duration"
- `value: f64` - Actual value
- `mean: f64` - Average
- `stddev: f64` - Standard deviation
- `z_score: f64` - Z-score
- `is_outlier: bool` - Flag

---

## Sorting

All results are sorted by primary metric (descending):
- **Workload**: by event_count
- **Transitions**: by avg_time_seconds
- **Complexity**: by complexity_score
- **Anomalies**: by anomaly_score
- **Outliers**: by |z_score|
- **Efficiency**: by efficiency_score
- **Availability**: by num_activities

---

## Testing

Run all statistics tests:
```bash
cargo test --test statistics_additional_test
```

Run specific test:
```bash
cargo test test_calculate_case_complexity_basic
```

Run with output:
```bash
cargo test --test statistics_additional_test -- --nocapture
```
