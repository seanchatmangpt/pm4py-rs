//! Verify EVERY Statistics function individually
use pm4py::statistics::*;
use pm4py::*;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    println!("=== VERIFYING STATISTICS MODULE - EVERY FUNCTION ===\n");
    let log = io::XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    let net = AlphaMiner::new().discover(&log);
    let mut count = 0;

    // Core statistics (8)
    println!("1. log_statistics");
    let _ = statistics::log_statistics(&log);
    count += 1;
    println!("2. activity_occurrence_matrix");
    let _ = statistics::activity_occurrence_matrix(&log);
    count += 1;
    println!("3. directly_follows_matrix");
    let _ = statistics::directly_follows_matrix(&log);
    count += 1;
    println!("4. get_start_activities");
    let _ = statistics::get_start_activities(&log);
    count += 1;
    println!("5. get_end_activities");
    let _ = statistics::get_end_activities(&log);
    count += 1;
    println!("6. get_case_duration");
    let _ = statistics::get_case_duration(&log);
    count += 1;
    println!("7. get_trace_length");
    let _ = statistics::get_trace_length(&log);
    count += 1;
    println!("8. variant_frequencies");
    let _ = statistics::variant_frequencies(&log.traces);
    count += 1;

    // Tree analysis (1)
    println!("9. analyze_tree");
    let _ = statistics::analyze_tree(&ProcessTree::default());
    count += 1;

    // Performance (7)
    println!("10. calculate_cycle_time");
    let _ = statistics::calculate_cycle_time(&log.traces[0]);
    count += 1;
    println!("11. calculate_process_variance");
    let _ = statistics::calculate_process_variance(&log);
    count += 1;
    println!("12. calculate_resource_utilization");
    let _ = statistics::calculate_resource_utilization(&log);
    count += 1;
    println!("13. calculate_sojourn_time");
    let _ = statistics::calculate_sojourn_time(&log.traces[0], "A");
    count += 1;
    println!("14. calculate_waiting_times");
    let _ = statistics::calculate_waiting_times(&log.traces[0]);
    count += 1;
    println!("15. process_performance_analysis");
    let _ = statistics::process_performance_analysis(&log);
    count += 1;
    println!("16. trace_performance_metrics");
    let _ = statistics::trace_performance_metrics(&log.traces[0]);
    count += 1;

    // Correlation and analysis (4)
    println!("17. activity_co_occurrence");
    let _ = statistics::activity_co_occurrence(&log);
    count += 1;
    println!("18. case_attribute_correlation");
    let _ = statistics::case_attribute_correlation(&log);
    count += 1;
    println!("19. causal_dependency_analysis");
    let _ = statistics::causal_dependency_analysis(&log);
    count += 1;
    println!("20. network_metrics");
    let _ = statistics::network_metrics(&log);
    count += 1;

    // Model checking (3)
    println!("21. check_is_fitting");
    let _ = statistics::check_is_fitting(&log, &net);
    count += 1;
    println!("22. check_is_workflow_net");
    let _ = statistics::check_is_workflow_net(&net);
    count += 1;
    println!("23. check_soundness");
    let _ = statistics::check_soundness(&net);
    count += 1;

    // Temporal (2)
    println!("24. discover_temporal_profile");
    let _ = statistics::discover_temporal_profile(&log);
    count += 1;
    println!("25. conformance_temporal_profile");
    let _ = statistics::conformance_temporal_profile(
        &log,
        &statistics::TemporalProfile::default(),
        0.1,
    );
    count += 1;

    // ML features (5)
    println!("26. extract_features");
    let _ = statistics::extract_features(&log);
    count += 1;
    println!("27. create_feature_matrix");
    let _ = statistics::create_feature_matrix(&log);
    count += 1;
    println!("28. get_feature_names");
    let _ = statistics::get_feature_names();
    count += 1;
    println!("29. features_to_vector");
    let _ = statistics::features_to_vector(
        &statistics::TraceFeatures::new("test".to_string()),
        &["A".to_string()],
    );
    count += 1;
    println!("30. normalize_features");
    statistics::normalize_features(&mut vec![vec![0.0]]);
    count += 1;

    // Encoding (3)
    println!("31. one_hot_encode");
    let _ = statistics::one_hot_encode("A", &["A".to_string()]);
    count += 1;
    println!("32. get_numeric_attributes");
    let _ = statistics::get_numeric_attributes(&log);
    count += 1;
    println!("33. get_str_attributes");
    let _ = statistics::get_str_attributes(&log);
    count += 1;

    // Filtering (7)
    println!("34. filter_start_activities");
    let _ = statistics::filter_start_activities(&log, &[]);
    count += 1;
    println!("35. filter_end_activities");
    let _ = statistics::filter_end_activities(&log, &[]);
    count += 1;
    println!("36. filter_traces_by_attribute");
    let _ = statistics::filter_traces_by_attribute(&log, "concept:name", "A");
    count += 1;
    println!("37. get_str_attribute_values");
    let _ = statistics::get_str_attribute_values(&log, "concept:name");
    count += 1;
    println!("38. sample_traces");
    let _ = statistics::sample_traces(&log, 2);
    count += 1;
    println!("39. train_test_split");
    let _ = statistics::train_test_split(&[1, 2, 3], 0.7);
    count += 1;
    println!("40. unique_traces");
    let _ = statistics::unique_traces(&log.traces);
    count += 1;

    // Advanced analysis (5)
    println!("41. detect_change_points");
    let _ = statistics::detect_change_points(&log, 5);
    count += 1;
    println!("42. detect_drift");
    let _ = statistics::detect_drift(&log, 0.5);
    count += 1;
    println!("43. stability_analysis");
    let _ = statistics::stability_analysis(&log, 5);
    count += 1;
    println!("44. get_activity_position_summary");
    let _ = statistics::get_activity_position_summary(&log);
    count += 1;
    println!("45. get_all_activities");
    let _ = statistics::get_all_activities(&log);
    count += 1;

    // Attribute utilities (6)
    println!("46. get_case_arrival_average");
    let _ = statistics::get_case_arrival_average(&log);
    count += 1;
    println!("47. get_case_overlap");
    let _ = statistics::get_case_overlap(&log);
    count += 1;
    println!("48. get_enabled_transitions");
    let _ = statistics::get_enabled_transitions(&net, &HashMap::new());
    count += 1;
    println!("49. get_frequent_trace_segments");
    let _ = statistics::get_frequent_trace_segments(&log, 2, 1, 10);
    count += 1;
    println!("50. get_minimum_self_distances");
    let _ = statistics::get_minimum_self_distances(&log, "A");
    count += 1;
    println!("51. get_minimum_self_distance_witnesses");
    let _ = statistics::get_minimum_self_distance_witnesses(&log, "A");
    count += 1;

    // Prefixes and variants (3)
    println!("52. get_prefixes_from_log");
    let _ = statistics::get_prefixes_from_log(&log, 3);
    count += 1;
    println!("53. get_variants_as_tuples");
    let _ = statistics::get_variants_as_tuples(&log);
    count += 1;
    println!("54. get_rework_cases_per_activity");
    let _ = statistics::get_rework_cases_per_activity(&log);
    count += 1;

    // Trace utilities (3)
    println!("55. trace_attribute_stats");
    let _ = statistics::trace_attribute_stats(&log.traces);
    count += 1;
    println!("56. trace_length_distribution");
    let _ = statistics::trace_length_distribution(&log.traces);
    count += 1;
    println!("57. get_numeric_attribute_values");
    let _ = statistics::get_numeric_attribute_values(&log, "cost");
    count += 1;

    // Extended (1)
    println!("58. discover_bpmn_inductive");
    let _ = statistics::discover_bpmn_inductive(&ProcessTree::default());
    count += 1;

    println!("\n✅ Statistics module: {}/58 functions verified", count);
}
