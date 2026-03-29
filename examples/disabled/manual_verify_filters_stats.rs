/// Manual verification of pm4py-rust filtering and statistics
use pm4py::io::XESReader;
use pm4py::log::advanced_filters::AdvancedFilter;
use pm4py::log::operations;
use pm4py::statistics::{activity_occurrence_matrix, directly_follows_matrix, log_statistics};
use std::path::Path;

fn main() {
    println!("=== MANUAL VERIFICATION OF PM4PY-RUST FILTERING & STATISTICS ===\n");

    // Load the log
    let path = Path::new("/Users/sac/chatmangpt/test_simple.xes");
    let reader = XESReader::new();
    let log = reader.read(path).expect("Failed to load XES");

    println!(
        "Original log: {} traces, {} events\n",
        log.traces.len(),
        log.traces.iter().map(|t| t.events.len()).sum::<usize>()
    );

    // 1. LOG STATISTICS - MANUAL VERIFICATION
    println!("1. LOG STATISTICS");
    let stats = log_statistics(&log);
    println!("  - Traces: {}", stats.num_traces);
    println!("  - Events: {}", stats.num_events);
    println!("  - Unique activities: {}", stats.num_unique_activities);
    println!("  - Variants: {}", stats.num_variants);
    println!("  - Avg trace length: {:.2}", stats.avg_trace_length);

    if stats.num_traces == 5 && stats.num_events == 15 && stats.num_unique_activities == 3 {
        println!("  ✓ LOG STATISTICS WORKS\n");
    } else {
        println!("  ✗ LOG STATISTICS FAILED\n");
    }

    // 2. ACTIVITY FREQUENCY - MANUAL VERIFICATION
    println!("2. ACTIVITY FREQUENCY");
    let activity_freq = activity_occurrence_matrix(&log);
    println!("  - Activity frequencies: {:?}", activity_freq);

    if activity_freq.len() == 3
        && activity_freq.get("A") == Some(&5)
        && activity_freq.get("B") == Some(&5)
        && activity_freq.get("C") == Some(&5)
    {
        println!("  ✓ ACTIVITY FREQUENCY WORKS\n");
    } else {
        println!("  ✗ ACTIVITY FREQUENCY FAILED\n");
    }

    // 3. DIRECTLY FOLLOWS - MANUAL VERIFICATION
    println!("3. DIRECTLY FOLLOWS RELATIONSHIPS");
    let df_matrix = directly_follows_matrix(&log);
    println!("  - DF pairs: {:?}", df_matrix);

    if df_matrix.len() == 2
        && df_matrix.get(&("A".to_string(), "B".to_string())) == Some(&5)
        && df_matrix.get(&("B".to_string(), "C".to_string())) == Some(&5)
    {
        println!("  ✓ DIRECTLY FOLLOWS WORKS\n");
    } else {
        println!("  ✗ DIRECTLY FOLLOWS FAILED\n");
    }

    // 4. TRACE VARIANTS - MANUAL VERIFICATION
    println!("4. TRACE VARIANTS");
    let variants = operations::variants(&log);
    println!("  - Number of variants: {}", variants.len());

    if variants.len() == 1 {
        println!("  ✓ TRACE VARIANTS WORKS\n");
    } else {
        println!("  ✗ TRACE VARIANTS FAILED\n");
    }

    // 5. FILTERING BY ACTIVITY - MANUAL VERIFICATION
    println!("5. FILTERING BY ACTIVITY");
    let filtered = log.filter_by_activity("A");
    println!("  - Filtered log (A): {} traces", filtered.traces.len());
    println!(
        "  - Filtered events: {}",
        filtered
            .traces
            .iter()
            .map(|t| t.events.len())
            .sum::<usize>()
    );

    if filtered.traces.len() == 5 {
        println!("  ✓ FILTERING BY ACTIVITY WORKS\n");
    } else {
        println!("  ✗ FILTERING BY ACTIVITY FAILED\n");
    }

    // 6. FILTERING BY TRACE LENGTH - MANUAL VERIFICATION
    println!("6. FILTERING BY TRACE LENGTH");
    let filtered = log.filter_by_min_length(2);
    println!(
        "  - Filtered log (>=2 events): {} traces",
        filtered.traces.len()
    );

    if filtered.traces.len() == 5 {
        println!("  ✓ FILTERING BY TRACE LENGTH WORKS\n");
    } else {
        println!("  ✗ FILTERING BY TRACE LENGTH FAILED\n");
    }

    // 7. ADVANCED FILTERING BY VARIANT - MANUAL VERIFICATION
    println!("7. ADVANCED FILTERING BY VARIANT");
    let variant_filter = AdvancedFilter::by_variant(&log, &["A", "B", "C"]);
    println!(
        "  - Filtered by variant [A,B,C]: {} traces",
        variant_filter.log.traces.len()
    );
    println!("  - Retention rate: {:.2}", variant_filter.retention_rate());

    if variant_filter.log.traces.len() == 5 {
        println!("  ✓ ADVANCED FILTERING BY VARIANT WORKS\n");
    } else {
        println!("  ✗ ADVANCED FILTERING BY VARIANT FAILED\n");
    }

    // 8. ADVANCED FILTERING BY TIME RANGE - MANUAL VERIFICATION
    println!("8. ADVANCED FILTERING BY TIME RANGE");
    use chrono::{Duration, Utc};
    let start_time = Utc::now() - Duration::days(365);
    let end_time = Utc::now() + Duration::days(365);
    let time_filter = AdvancedFilter::by_time_range(&log, start_time, end_time);
    println!(
        "  - Filtered by time range: {} traces",
        time_filter.log.traces.len()
    );

    if time_filter.log.traces.len() == 5 {
        println!("  ✓ ADVANCED FILTERING BY TIME RANGE WORKS\n");
    } else {
        println!("  ✗ ADVANCED FILTERING BY TIME RANGE FAILED\n");
    }

    println!("=== FILTERING & STATISTICS VERIFIED ===");
}
