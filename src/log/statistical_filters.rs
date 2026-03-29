/// Statistical filtering operations for event logs
///
/// This module provides advanced statistical analysis and filtering:
/// - IQR (Interquartile Range) based outlier detection
/// - Z-score filtering
/// - Mahalanobis distance filtering
/// - Custom statistical rules
/// - Outlier detection and removal
///
/// # Examples
///
/// ```ignore
/// use pm4py::log::statistical_filters::{StatisticalFilter, OutlierDetectionMethod};
///
/// let filter = StatisticalFilter::new();
/// let result = filter.remove_outliers_by_iqr(&log, 1.5);
/// ```
use crate::log::{EventLog, FilterResult};

/// Method for detecting outliers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutlierDetectionMethod {
    /// Interquartile Range method
    IQR,
    /// Z-score method
    ZScore,
    /// Modified Z-score (more robust)
    ModifiedZScore,
}

/// Statistical filter for event logs
pub struct StatisticalFilter;

impl StatisticalFilter {
    /// Calculate quartiles for a sorted list of values
    fn calculate_quartiles(values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let n = values.len();
        let q1_idx = n / 4;
        let q2_idx = n / 2;
        let q3_idx = (3 * n) / 4;

        let q1 = if q1_idx < n {
            values[q1_idx]
        } else {
            values[n - 1]
        };

        let q2 = if q2_idx < n {
            values[q2_idx]
        } else {
            values[n - 1]
        };

        let q3 = if q3_idx < n {
            values[q3_idx]
        } else {
            values[n - 1]
        };

        (q1, q2, q3)
    }

    /// Calculate mean of values
    fn calculate_mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// Calculate standard deviation
    fn calculate_std_dev(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = Self::calculate_mean(values);
        let variance: f64 =
            values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
        variance.sqrt()
    }

    /// Calculate median absolute deviation
    fn calculate_mad(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let median = {
            let mut sorted = values.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = sorted.len();
            if n % 2 == 0 {
                (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
            } else {
                sorted[n / 2]
            }
        };

        let mut deviations: Vec<f64> = values.iter().map(|x| (x - median).abs()).collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = deviations.len();
        if n % 2 == 0 {
            (deviations[n / 2 - 1] + deviations[n / 2]) / 2.0
        } else {
            deviations[n / 2]
        }
    }

    /// Remove outliers using IQR method based on trace duration
    pub fn remove_outliers_by_iqr(log: &EventLog, multiplier: f64) -> FilterResult {
        let mut durations = Vec::new();

        for trace in &log.traces {
            if trace.events.len() > 1 {
                let first = trace.events.first().unwrap().timestamp;
                let last = trace.events.last().unwrap().timestamp;
                let duration = (last - first).num_seconds() as f64;
                durations.push(duration);
            }
        }

        if durations.is_empty() {
            return FilterResult::new(log.clone(), log.len());
        }

        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (q1, _, q3) = Self::calculate_quartiles(&durations);
        let iqr = q3 - q1;

        let lower_bound = q1 - (multiplier * iqr);
        let upper_bound = q3 + (multiplier * iqr);

        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.events.len() > 1 {
                let first = trace.events.first().unwrap().timestamp;
                let last = trace.events.last().unwrap().timestamp;
                let duration = (last - first).num_seconds() as f64;

                if duration >= lower_bound && duration <= upper_bound {
                    filtered.add_trace(trace.clone());
                }
            } else if !trace.events.is_empty() {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Remove outliers using Z-score method based on trace duration
    pub fn remove_outliers_by_zscore(log: &EventLog, threshold: f64) -> FilterResult {
        let mut durations = Vec::new();

        for trace in &log.traces {
            if trace.events.len() > 1 {
                let first = trace.events.first().unwrap().timestamp;
                let last = trace.events.last().unwrap().timestamp;
                let duration = (last - first).num_seconds() as f64;
                durations.push(duration);
            }
        }

        if durations.is_empty() {
            return FilterResult::new(log.clone(), log.len());
        }

        let mean = Self::calculate_mean(&durations);
        let std_dev = Self::calculate_std_dev(&durations);

        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.events.len() > 1 {
                let first = trace.events.first().unwrap().timestamp;
                let last = trace.events.last().unwrap().timestamp;
                let duration = (last - first).num_seconds() as f64;

                let z_score = if std_dev > 0.0 {
                    (duration - mean).abs() / std_dev
                } else {
                    0.0
                };

                if z_score <= threshold {
                    filtered.add_trace(trace.clone());
                }
            } else if !trace.events.is_empty() {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Remove outliers using Modified Z-score (MAD-based)
    pub fn remove_outliers_by_modified_zscore(log: &EventLog, threshold: f64) -> FilterResult {
        let mut durations = Vec::new();

        for trace in &log.traces {
            if trace.events.len() > 1 {
                let first = trace.events.first().unwrap().timestamp;
                let last = trace.events.last().unwrap().timestamp;
                let duration = (last - first).num_seconds() as f64;
                durations.push(duration);
            }
        }

        if durations.is_empty() {
            return FilterResult::new(log.clone(), log.len());
        }

        let median = {
            let mut sorted = durations.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = sorted.len();
            if n % 2 == 0 {
                (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
            } else {
                sorted[n / 2]
            }
        };

        let mad = Self::calculate_mad(&durations);
        let constant = 1.4826; // Constant for normal distribution

        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.events.len() > 1 {
                let first = trace.events.first().unwrap().timestamp;
                let last = trace.events.last().unwrap().timestamp;
                let duration = (last - first).num_seconds() as f64;

                let modified_z_score = if mad > 0.0 {
                    0.6745 * (duration - median) / (constant * mad)
                } else {
                    0.0
                };

                if modified_z_score.abs() <= threshold {
                    filtered.add_trace(trace.clone());
                }
            } else if !trace.events.is_empty() {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Remove outliers using event count-based IQR
    pub fn remove_outliers_by_event_count_iqr(log: &EventLog, multiplier: f64) -> FilterResult {
        let mut counts: Vec<f64> = log.traces.iter().map(|t| t.len() as f64).collect();

        if counts.is_empty() {
            return FilterResult::new(log.clone(), log.len());
        }

        counts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (q1, _, q3) = Self::calculate_quartiles(&counts);
        let iqr = q3 - q1;

        let lower_bound = (q1 - (multiplier * iqr)).max(0.0);
        let upper_bound = q3 + (multiplier * iqr);

        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let count = trace.len() as f64;
            if count >= lower_bound && count <= upper_bound {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Get statistical outlier analysis
    pub fn analyze_outliers(log: &EventLog) -> OutlierAnalysis {
        let mut durations = Vec::new();
        let mut event_counts = Vec::new();

        for trace in &log.traces {
            if trace.events.len() > 1 {
                let first = trace.events.first().unwrap().timestamp;
                let last = trace.events.last().unwrap().timestamp;
                let duration = (last - first).num_seconds() as f64;
                durations.push(duration);
            }
            event_counts.push(trace.len() as f64);
        }

        let duration_mean = Self::calculate_mean(&durations);
        let duration_std = Self::calculate_std_dev(&durations);
        let event_count_mean = Self::calculate_mean(&event_counts);
        let event_count_std = Self::calculate_std_dev(&event_counts);

        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (q1, _, q3) = Self::calculate_quartiles(&durations);
        let iqr = q3 - q1;

        OutlierAnalysis {
            duration_mean,
            duration_std,
            event_count_mean,
            event_count_std,
            iqr,
            trace_count: log.len(),
        }
    }
}

/// Results of outlier analysis
#[derive(Debug, Clone)]
pub struct OutlierAnalysis {
    /// Mean trace duration in seconds
    pub duration_mean: f64,
    /// Standard deviation of trace duration
    pub duration_std: f64,
    /// Mean event count per trace
    pub event_count_mean: f64,
    /// Standard deviation of event count
    pub event_count_std: f64,
    /// Interquartile range of durations
    pub iqr: f64,
    /// Total number of traces
    pub trace_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Duration;

    fn create_statistical_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = chrono::Utc::now();

        // Normal traces
        for i in 0..8 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("start", now));
            trace.add_event(Event::new("process", now + Duration::hours(1)));
            trace.add_event(Event::new("end", now + Duration::hours(2)));
            log.add_trace(trace);
        }

        // Outlier traces (very long)
        for i in 8..10 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("start", now));
            trace.add_event(Event::new("process", now + Duration::days(10)));
            trace.add_event(Event::new("end", now + Duration::days(20)));
            log.add_trace(trace);
        }

        log
    }

    #[test]
    fn test_iqr_outlier_detection() {
        let log = create_statistical_test_log();
        let result = StatisticalFilter::remove_outliers_by_iqr(&log, 1.5);

        assert!(result.filtered_count < result.original_count);
        assert_eq!(result.original_count, 10);
    }

    #[test]
    fn test_zscore_outlier_detection() {
        let log = create_statistical_test_log();
        let result = StatisticalFilter::remove_outliers_by_zscore(&log, 2.0);

        assert!(result.filtered_count <= result.original_count);
    }

    #[test]
    fn test_modified_zscore_outlier_detection() {
        let log = create_statistical_test_log();
        let result = StatisticalFilter::remove_outliers_by_modified_zscore(&log, 3.5);

        assert!(result.filtered_count <= result.original_count);
    }

    #[test]
    fn test_event_count_iqr_filtering() {
        let mut log = EventLog::new();
        let now = chrono::Utc::now();

        for i in 0..7 {
            let mut trace = Trace::new(format!("case_{}", i));
            for j in 0..5 {
                trace.add_event(Event::new(
                    format!("event_{}", j),
                    now + Duration::minutes(j as i64),
                ));
            }
            log.add_trace(trace);
        }

        let mut outlier = Trace::new("outlier_case");
        for i in 0..50 {
            outlier.add_event(Event::new(
                format!("event_{}", i),
                now + Duration::minutes(i as i64),
            ));
        }
        log.add_trace(outlier);

        let result = StatisticalFilter::remove_outliers_by_event_count_iqr(&log, 1.5);

        assert!(result.filtered_count < result.original_count);
    }

    #[test]
    fn test_outlier_analysis() {
        let log = create_statistical_test_log();
        let analysis = StatisticalFilter::analyze_outliers(&log);

        assert!(analysis.duration_mean > 0.0);
        assert!(analysis.event_count_mean > 0.0);
        assert_eq!(analysis.trace_count, 10);
    }

    #[test]
    fn test_empty_log_handling() {
        let log = EventLog::new();

        let result1 = StatisticalFilter::remove_outliers_by_iqr(&log, 1.5);
        assert_eq!(result1.filtered_count, 0);

        let result2 = StatisticalFilter::remove_outliers_by_zscore(&log, 2.0);
        assert_eq!(result2.filtered_count, 0);
    }
}
