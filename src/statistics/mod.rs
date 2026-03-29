//pub mod additional;  // DISABLED: Has compilation errors, will be fixed separately
/// Statistical analysis of event logs
///
/// This module provides comprehensive statistical analysis capabilities for event logs,
/// including performance metrics, bottleneck analysis, correlation analysis, and more.
///
/// # Performance Analysis
///
/// Analyze timing and throughput metrics:
///
/// ```rust
/// use pm4py::statistics::{
///     calculate_cycle_time,
///     calculate_resource_utilization,
///     case_duration_distribution,
/// };
/// use pm4py::{EventLog, Trace};
///
/// // Get cycle time statistics for a trace
/// let trace = Trace::new("case-1");
/// let cycle_time = calculate_cycle_time(&trace);
/// println!("Cycle time: {} seconds", cycle_time);
///
/// // Analyze resource utilization
/// let event_log = EventLog::new();
/// let utilization = calculate_resource_utilization(&event_log);
/// for ru in &utilization {
///     println!("{}: {} activities", ru.resource, ru.num_activities);
/// }
/// ```
///
/// # Bottleneck Analysis
///
/// Identify process bottlenecks:
///
/// ```rust
/// use pm4py::statistics::{identify_bottlenecks, BottleneckMetrics};
/// use pm4py::EventLog;
///
/// let event_log = EventLog::new();
/// let bottlenecks = identify_bottlenecks(&event_log);
/// for bottleneck in &bottlenecks {
///     println!("Bottleneck: {} (cumulative time: {:.1}s)",
///              bottleneck.activity, bottleneck.cumulative_time);
/// }
/// ```
///
/// # Rework Analysis
///
/// Detect and analyze rework patterns:
///
/// ```rust
/// use pm4py::statistics::{detect_rework, ReworkInstance};
/// use pm4py::EventLog;
///
/// let event_log = EventLog::new();
/// let rework_instances = detect_rework(&event_log);
/// println!("Rework instances found: {}", rework_instances.len());
/// ```
///
/// # Correlation Analysis
///
/// Analyze relationships between activities and attributes:
///
/// ```rust
/// use pm4py::statistics::{activity_co_occurrence, causal_dependency_analysis};
/// use pm4py::EventLog;
///
/// let event_log = EventLog::new();
/// // Activity co-occurrence
/// let co_occurrence = activity_co_occurrence(&event_log);
///
/// // Causal dependencies
/// let dependencies = causal_dependency_analysis(&event_log);
/// ```
///
/// # Stability and Drift Detection
///
/// Detect process changes over time:
///
/// ```rust
/// use pm4py::statistics::{detect_drift, detect_change_points};
/// use pm4py::EventLog;
///
/// let event_log = EventLog::new();
/// let drift = detect_drift(&event_log, 0.5);
/// if !drift.drift_positions.is_empty() {
///     println!("Drift detected at positions: {:?}", drift.drift_positions);
/// }
/// ```
///
/// # Temporal Analysis
///
/// Analyze timing patterns and constraints:
///
/// ```rust
/// use pm4py::statistics::{discover_temporal_profile, conformance_temporal_profile};
/// use pm4py::EventLog;
///
/// let event_log = EventLog::new();
/// let profile = discover_temporal_profile(&event_log);
/// let result = conformance_temporal_profile(&event_log, &profile, 0.1);
/// ```
pub mod advanced;
pub mod bottleneck;
pub mod correlation;
pub mod extended_metrics;
pub mod extended_stats2;
pub mod log_stats;
pub mod missing_stats;
pub mod ml_features;
pub mod rework;
pub mod stability;
pub mod temporal_profile;
pub mod trace_stats;
pub mod tree_stats;

// DISABLED: additional module has compilation errors
// pub use additional::{...};
pub use advanced::{
    calculate_performance_indicators, case_duration_distribution,
    case_duration_distribution_per_variant, get_activity_frequency, get_activity_throughput,
    get_bottleneck_activities, get_case_throughput, get_resource_metrics,
    get_rework_count_per_activity, get_variant_frequency, identify_rework_patterns,
    ActivityFrequency, CaseDurationStats, PerformanceIndicators, ResourceMetrics, ReworkPattern,
    VariantDurationStats, VariantFrequency,
};
pub use bottleneck::{
    analyze_path_performance, analyze_resource_performance, calculate_activity_metrics,
    check_sla_violations, identify_bottlenecks, ActivityMetrics, BottleneckMetrics, PathMetrics,
    ResourcePerformance, SLAViolation,
};
pub use correlation::{
    activity_co_occurrence, case_attribute_correlation, causal_dependency_analysis,
    network_metrics, AttributeCorrelation, CausalDependency, CoOccurrence, NetworkMetrics,
};
pub use extended_metrics::{
    calculate_cycle_time, calculate_resource_utilization, calculate_sojourn_time,
    calculate_waiting_times, process_performance_analysis, trace_performance_metrics,
    ProcessPerformanceAnalysis, ResourceUtilization, TracePerformanceMetrics,
};
pub use extended_stats2::{
    discover_bpmn_inductive, get_activity_position_summary, get_case_arrival_average,
    get_case_overlap, get_frequent_trace_segments, get_prefixes_from_log,
    get_rework_cases_per_activity, get_variants_as_tuples,
};
pub use log_stats::*;
pub use missing_stats::*;
pub use ml_features::*;
pub use rework::{
    detect_loops, detect_potential_infinite_loops, detect_rework, get_rework_cases_for_activity,
    rework_statistics, LoopPattern, ReworkInstance, ReworkStats,
};
pub use stability::{
    calculate_process_variance, detect_change_points, detect_drift, stability_analysis,
    ChangePoint, DriftDetectionResult, ProcessVariance, StabilityIndex,
};
pub use temporal_profile::{
    check_is_fitting, check_is_workflow_net, check_soundness, conformance_temporal_profile,
    discover_temporal_profile, get_enabled_transitions, get_minimum_self_distance_witnesses,
    get_minimum_self_distances, TemporalConformanceResult, TemporalProfile,
};
pub use trace_stats::*;
pub use tree_stats::{analyze_tree, TreeMetrics, TreePattern, TreeStatistics};
