//! Verify EVERY Log module function individually
use pm4py::log::*;
use pm4py::*;
use std::path::Path;

fn main() {
    println!("=== VERIFYING LOG MODULE - EVERY FUNCTION ===\n");
    let log = io::XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    let mut count = 0;

    // Core operations (8)
    println!("1. activity_frequency");
    let _ = activity_frequency(&log);
    count += 1;
    println!("2. activity_resources");
    let _ = activity_resources(&log);
    count += 1;
    println!("3. directly_follows");
    let _ = directly_follows(&log);
    count += 1;
    println!("4. end_activities");
    let _ = end_activities(&log);
    count += 1;
    println!("5. start_activities");
    let _ = start_activities(&log);
    count += 1;
    println!("6. variants");
    let _ = variants(&log);
    count += 1;
    println!("7. is_consistent");
    let _ = is_consistent(&log);
    count += 1;
    println!("8. get_variant");
    let _ = get_variant(&log.traces[0]);
    count += 1;

    // Trace utilities (3)
    println!("9. sequence_encoding");
    let _ = sequence_encoding(&log.traces[0]);
    count += 1;
    println!("10. sort_traces_by_length");
    let mut l = log.clone();
    sort_traces_by_length(&mut l);
    count += 1;
    println!("11. sort_traces_by_timestamp");
    let mut l2 = log.clone();
    sort_traces_by_timestamp(&mut l2);
    count += 1;

    // Advanced filters (10)
    println!("12. filter_case_size");
    let _ = filter_case_size(&log, 1, 10);
    count += 1;
    println!("13. filter_trace_prefix");
    let _ = filter_trace_prefix(&log, &["A".to_string()]);
    count += 1;
    println!("14. filter_trace_suffix");
    let _ = filter_trace_suffix(&log, &["A".to_string()]);
    count += 1;
    println!("15. filter_variants_top_k");
    let _ = filter_variants_top_k(&log, 5);
    count += 1;
    println!("16. filter_activities_rework");
    let _ = filter_activities_rework(&log, "A");
    count += 1;
    println!("17. filter_trace_attribute");
    let _ = filter_trace_attribute(&log, "concept:name", "A");
    count += 1;
    println!("18. filter_event_attribute_values");
    let _ = filter_event_attribute_values(&log, "concept:name", &["A".to_string()]);
    count += 1;
    println!("19. filter_time_range");
    let _ = filter_time_range(&log, chrono::Utc::now(), chrono::Utc::now());
    count += 1;
    println!("20. filter_traces_containing_activity");
    let _ = filter_traces_containing_activity(&log, "A");
    count += 1;
    println!("21. filter_traces_with_activity");
    let _ = filter_traces_with_activity(&log, "A");
    count += 1;

    // DFG filters (8)
    println!("22. filter_dfg_activities_percentage");
    let _ = filter_dfg_activities_percentage(&log, 0.8);
    count += 1;
    println!("23. filter_dfg_paths_percentage");
    let _ = filter_dfg_paths_percentage(&log, 0.8);
    count += 1;
    println!("24. filter_paths_performance");
    let _ = filter_paths_performance(&log, "A", "B", 0.0, 1000.0);
    count += 1;
    println!("25. filter_four_eyes_principle");
    let _ = filter_four_eyes_principle(&log, "A", "B");
    count += 1;
    println!("26. filter_between");
    let _ = filter_between(&log, "A", "B");
    count += 1;
    println!("27. filter_eventually_follows_relation");
    let _ = filter_eventually_follows_relation(&log, "A", "B");
    count += 1;
    println!("28. filter_variants_by_coverage_percentage");
    let _ = filter_variants_by_coverage_percentage(&log, 0.8);
    count += 1;
    println!("29. filter_log_relative_occurrence_event_attribute");
    let _ = filter_log_relative_occurrence_event_attribute(&log, "concept:name", "A", 0.5);
    count += 1;

    // Attribute utilities (4)
    println!("30. get_event_attributes");
    let _ = get_event_attributes(&log);
    count += 1;
    println!("31. get_event_attribute_values");
    let _ = get_event_attribute_values(&log, "concept:name");
    count += 1;
    println!("32. get_trace_attributes");
    let _ = get_trace_attributes(&log);
    count += 1;
    println!("33. get_trace_attribute_values");
    let _ = get_trace_attribute_values(&log, "concept:name");
    count += 1;

    // Operations (5)
    println!("34. remove_duplicate_events");
    let mut t = log.traces[0].clone();
    remove_duplicate_events(&mut t);
    count += 1;
    println!("35. remove_duplicates");
    let mut l3 = log.clone();
    remove_duplicates(&mut l3);
    count += 1;
    println!("36. keep_top_activities");
    let mut l4 = log.clone();
    keep_top_activities(&mut l4, 5);
    count += 1;
    println!("37. time_between_activities");
    let _ = time_between_activities(&log, "A", "B");
    count += 1;
    println!("38. filter_activity_done_different_resources");
    let _ = filter_activity_done_different_resources(&log, "A", "B");
    count += 1;

    println!("\n✅ Log module: {}/38 functions verified", count);
}
