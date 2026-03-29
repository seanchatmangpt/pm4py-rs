/// Process stability and variance analysis
///
/// This module provides analysis capabilities for:
/// - Process variance estimation (variability in execution)
/// - Stability indices over time (trend analysis)
/// - Drift detection (identifying process changes)
/// - Behavioral change points (where process behavior shifts significantly)
use crate::log::EventLog;
use std::collections::HashMap;

/// Variance metrics for process execution
#[derive(Debug, Clone)]
pub struct ProcessVariance {
    /// Trace length variance
    pub trace_length_variance: f64,
    /// Trace length standard deviation
    pub trace_length_std_dev: f64,
    /// Activity frequency variance across traces
    pub activity_variance: HashMap<String, f64>,
    /// Overall process entropy (measure of diversity)
    pub entropy: f64,
    /// Coefficient of variation for trace lengths (0.0-1.0+)
    pub coefficient_of_variation: f64,
}

/// Stability index over a time window
#[derive(Debug, Clone)]
pub struct StabilityIndex {
    /// Window start position (trace index)
    pub window_start: usize,
    /// Window end position (trace index)
    pub window_end: usize,
    /// Stability score (0.0-1.0, higher = more stable)
    pub stability_score: f64,
    /// Average variant frequency in window
    pub avg_variant_frequency: f64,
    /// Dominant variant in window
    pub dominant_variant: String,
}

/// Drift detection result
#[derive(Debug, Clone)]
pub struct DriftDetectionResult {
    /// Position(s) where drift was detected (trace indices)
    pub drift_positions: Vec<usize>,
    /// Severity of each detected drift (0.0-1.0)
    pub drift_severity: Vec<f64>,
    /// Description of detected drift
    pub drift_type: String,
}

/// Change point in process behavior
#[derive(Debug, Clone)]
pub struct ChangePoint {
    /// Position of change point (trace index)
    pub position: usize,
    /// Magnitude of change (0.0-1.0)
    pub magnitude: f64,
    /// Type of change (variant_shift, timing_change, resource_shift)
    pub change_type: String,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
}

/// Calculate variance metrics for process execution
pub fn calculate_process_variance(log: &EventLog) -> ProcessVariance {
    if log.is_empty() {
        return ProcessVariance {
            trace_length_variance: 0.0,
            trace_length_std_dev: 0.0,
            activity_variance: HashMap::new(),
            entropy: 0.0,
            coefficient_of_variation: 0.0,
        };
    }

    let lengths: Vec<usize> = log.traces.iter().map(|t| t.len()).collect();
    let mean_length = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;

    let trace_length_variance = lengths
        .iter()
        .map(|len| (*len as f64 - mean_length).powi(2))
        .sum::<f64>()
        / lengths.len() as f64;

    let trace_length_std_dev = trace_length_variance.sqrt();
    let coefficient_of_variation = if mean_length > 0.0 {
        trace_length_std_dev / mean_length
    } else {
        0.0
    };

    let mut activity_counts: HashMap<String, Vec<usize>> = HashMap::new();
    for trace in &log.traces {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for event in &trace.events {
            *counts.entry(event.activity.clone()).or_insert(0) += 1;
        }
        for (activity, count) in counts {
            activity_counts.entry(activity).or_default().push(count);
        }
    }

    let mut activity_variance = HashMap::new();
    for (activity, counts) in activity_counts {
        let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
        let variance = counts
            .iter()
            .map(|c| (*c as f64 - mean).powi(2))
            .sum::<f64>()
            / counts.len() as f64;
        activity_variance.insert(activity, variance);
    }

    let entropy = calculate_entropy(log);

    ProcessVariance {
        trace_length_variance,
        trace_length_std_dev,
        activity_variance,
        entropy,
        coefficient_of_variation,
    }
}

fn calculate_entropy(log: &EventLog) -> f64 {
    if log.is_empty() {
        return 0.0;
    }

    let mut variant_counts: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        let variant = trace
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<Vec<_>>()
            .join(",");
        *variant_counts.entry(variant).or_insert(0) += 1;
    }

    let total = log.len() as f64;
    let mut entropy = 0.0;

    for count in variant_counts.values() {
        let p = *count as f64 / total;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Analyze stability of process over time windows
pub fn stability_analysis(log: &EventLog, window_size: usize) -> Vec<StabilityIndex> {
    if log.is_empty() || window_size == 0 {
        return Vec::new();
    }

    let mut stability_indices = Vec::new();

    for window_start in (0..log.len()).step_by(window_size) {
        let window_end = (window_start + window_size).min(log.len());

        let mut variant_counts: HashMap<String, usize> = HashMap::new();

        for i in window_start..window_end {
            if let Some(trace) = log.traces.get(i) {
                let variant = trace
                    .events
                    .iter()
                    .map(|e| e.activity.clone())
                    .collect::<Vec<_>>()
                    .join(",");
                *variant_counts.entry(variant).or_insert(0) += 1;
            }
        }

        let window_size_f = (window_end - window_start) as f64;
        let avg_variant_frequency =
            variant_counts.values().sum::<usize>() as f64 / variant_counts.len().max(1) as f64;

        let dominant_variant = variant_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(v, _)| v.clone())
            .unwrap_or_default();

        let stability_score = if !variant_counts.is_empty() {
            let max_freq = variant_counts.values().max().copied().unwrap_or(0) as f64;
            max_freq / window_size_f
        } else {
            0.0
        };

        stability_indices.push(StabilityIndex {
            window_start,
            window_end,
            stability_score,
            avg_variant_frequency,
            dominant_variant,
        });
    }

    stability_indices
}

/// Detect process drift (significant behavior changes)
pub fn detect_drift(log: &EventLog, sensitivity: f64) -> DriftDetectionResult {
    if log.len() < 2 {
        return DriftDetectionResult {
            drift_positions: Vec::new(),
            drift_severity: Vec::new(),
            drift_type: "No drift".to_string(),
        };
    }

    let threshold = (1.0 - sensitivity).clamp(0.0, 1.0);
    let mut drift_positions = Vec::new();
    let mut drift_severity = Vec::new();

    let mut variant_counts: HashMap<String, usize> = HashMap::new();
    let mut last_variant_counts: HashMap<String, usize> = HashMap::new();

    for i in 0..log.len() {
        if let Some(trace) = log.traces.get(i) {
            let variant = trace
                .events
                .iter()
                .map(|e| e.activity.clone())
                .collect::<Vec<_>>()
                .join(",");
            *variant_counts.entry(variant).or_insert(0) += 1;
        }

        if i > 0 && i % 10 == 0 {
            let dissimilarity =
                calculate_distribution_dissimilarity(&last_variant_counts, &variant_counts);

            if dissimilarity > threshold {
                drift_positions.push(i);
                drift_severity.push(dissimilarity);
            }

            last_variant_counts = variant_counts.clone();
            variant_counts.clear();
        }
    }

    let drift_type = if drift_positions.is_empty() {
        "No significant drift".to_string()
    } else {
        "Variant shift detected".to_string()
    };

    DriftDetectionResult {
        drift_positions,
        drift_severity,
        drift_type,
    }
}

fn calculate_distribution_dissimilarity(
    dist1: &HashMap<String, usize>,
    dist2: &HashMap<String, usize>,
) -> f64 {
    if dist1.is_empty() || dist2.is_empty() {
        return 0.0;
    }

    let total1 = dist1.values().sum::<usize>().max(1) as f64;
    let total2 = dist2.values().sum::<usize>().max(1) as f64;

    let mut dissimilarity = 0.0;
    let all_keys: std::collections::HashSet<_> = dist1.keys().chain(dist2.keys()).collect();

    for key in all_keys {
        let p1 = dist1.get(key).copied().unwrap_or(0) as f64 / total1;
        let p2 = dist2.get(key).copied().unwrap_or(0) as f64 / total2;
        dissimilarity += (p1 - p2).abs();
    }

    dissimilarity / 2.0
}

/// Detect change points in process behavior
pub fn detect_change_points(log: &EventLog, window_size: usize) -> Vec<ChangePoint> {
    if log.len() < window_size * 2 {
        return Vec::new();
    }

    let mut change_points = Vec::new();
    let threshold = 0.3;

    for i in window_size..log.len() - window_size {
        let before_start = i.saturating_sub(window_size);
        let before_end = i;
        let after_start = i;
        let after_end = (i + window_size).min(log.len());

        let mut before_variants: HashMap<String, usize> = HashMap::new();
        let mut after_variants: HashMap<String, usize> = HashMap::new();

        for j in before_start..before_end {
            if let Some(trace) = log.traces.get(j) {
                let variant = trace
                    .events
                    .iter()
                    .map(|e| e.activity.clone())
                    .collect::<Vec<_>>()
                    .join(",");
                *before_variants.entry(variant).or_insert(0) += 1;
            }
        }

        for j in after_start..after_end {
            if let Some(trace) = log.traces.get(j) {
                let variant = trace
                    .events
                    .iter()
                    .map(|e| e.activity.clone())
                    .collect::<Vec<_>>()
                    .join(",");
                *after_variants.entry(variant).or_insert(0) += 1;
            }
        }

        let dissimilarity = calculate_distribution_dissimilarity(&before_variants, &after_variants);

        if dissimilarity > threshold {
            change_points.push(ChangePoint {
                position: i,
                magnitude: dissimilarity,
                change_type: "variant_shift".to_string(),
                confidence: dissimilarity,
            });
        }
    }

    change_points
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let base_time = Utc::now();

        for i in 0..5 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("A", base_time));
            trace.add_event(Event::new("B", base_time + chrono::Duration::seconds(1)));
            trace.add_event(Event::new("C", base_time + chrono::Duration::seconds(2)));
            if i % 2 == 0 {
                trace.add_event(Event::new("D", base_time + chrono::Duration::seconds(3)));
            }
            log.add_trace(trace);
        }

        log
    }

    fn create_drifting_log() -> EventLog {
        let mut log = EventLog::new();
        let base_time = Utc::now();

        // First 10 traces follow pattern A,B,C — establishes baseline window
        for i in 0..10 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("A", base_time));
            trace.add_event(Event::new("B", base_time + chrono::Duration::seconds(1)));
            trace.add_event(Event::new("C", base_time + chrono::Duration::seconds(2)));
            log.add_trace(trace);
        }

        // Next 11 traces follow completely different pattern X,Y,Z (drift detected at position 20)
        for i in 10..21 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("X", base_time));
            trace.add_event(Event::new("Y", base_time + chrono::Duration::seconds(1)));
            trace.add_event(Event::new("Z", base_time + chrono::Duration::seconds(2)));
            log.add_trace(trace);
        }

        log
    }

    #[test]
    fn test_calculate_process_variance() {
        let log = create_test_log();
        let variance = calculate_process_variance(&log);

        assert!(variance.trace_length_variance >= 0.0);
        assert!(variance.trace_length_std_dev >= 0.0);
        assert!(variance.entropy >= 0.0);
        assert!(variance.coefficient_of_variation >= 0.0);
    }

    #[test]
    fn test_variance_empty_log() {
        let log = EventLog::new();
        let variance = calculate_process_variance(&log);

        assert_eq!(variance.trace_length_variance, 0.0);
        assert_eq!(variance.trace_length_std_dev, 0.0);
    }

    #[test]
    fn test_stability_analysis() {
        let log = create_test_log();
        let stability = stability_analysis(&log, 2);

        assert!(!stability.is_empty());
        for index in &stability {
            assert!(index.stability_score >= 0.0 && index.stability_score <= 1.0);
            assert!(index.avg_variant_frequency >= 0.0);
        }
    }

    #[test]
    fn test_stability_empty_log() {
        let log = EventLog::new();
        let stability = stability_analysis(&log, 2);

        assert!(stability.is_empty());
    }

    #[test]
    fn test_detect_drift() {
        let log = create_drifting_log();
        let drift = detect_drift(&log, 0.8);

        assert!(drift.drift_positions.len() > 0);
        for severity in &drift.drift_severity {
            assert!(*severity >= 0.0);
        }
    }

    #[test]
    fn test_detect_drift_empty_log() {
        let log = EventLog::new();
        let drift = detect_drift(&log, 0.5);

        assert!(drift.drift_positions.is_empty());
    }

    #[test]
    fn test_detect_change_points() {
        let log = create_drifting_log();
        let change_points = detect_change_points(&log, 2);

        for cp in &change_points {
            assert!(cp.magnitude >= 0.0 && cp.magnitude <= 1.0);
            assert!(cp.confidence >= 0.0 && cp.confidence <= 1.0);
        }
    }

    #[test]
    fn test_activity_variance() {
        let log = create_test_log();
        let variance = calculate_process_variance(&log);

        assert!(!variance.activity_variance.is_empty());
        for var in variance.activity_variance.values() {
            assert!(*var >= 0.0);
        }
    }
}
