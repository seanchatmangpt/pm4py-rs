/// Drift detection for process model staleness monitoring
///
/// Detects changes in process metrics over time and calculates a drift score
/// to identify when models need updating. Implements the same drift formula
/// as the Elixir drift detector for cross-platform consistency.
use std::collections::HashMap;

/// Configuration for drift detection
#[derive(Debug, Clone)]
pub struct DriftCalculator {
    /// Threshold above which drift triggers an alert (0.0-1.0)
    pub threshold: f64,
}

impl Default for DriftCalculator {
    fn default() -> Self {
        Self { threshold: 0.2 }
    }
}

impl DriftCalculator {
    /// Create a new drift calculator with the default threshold of 0.2
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a drift calculator with a custom threshold
    pub fn with_threshold(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Calculate drift score between baseline and recent metrics
    ///
    /// Computes normalized distance: `avg(abs(recent - baseline) / abs(baseline))`
    /// for all metrics in the HashMap.
    ///
    /// Returns a float between 0.0 and 1.0.
    pub fn calculate_drift(
        &self,
        baseline: &HashMap<String, f64>,
        recent: &HashMap<String, f64>,
    ) -> f64 {
        let metric_keys = vec!["avg_duration", "error_rate", "success_rate", "throughput"];

        let drift_values: Vec<f64> = metric_keys
            .into_iter()
            .map(|key| {
                let baseline_val = baseline.get(key).copied().unwrap_or(0.0);
                let recent_val = recent.get(key).copied().unwrap_or(0.0);

                // Avoid division by zero
                if baseline_val == 0.0 {
                    0.0
                } else {
                    (recent_val - baseline_val).abs() / baseline_val.abs()
                }
            })
            .collect();

        if drift_values.is_empty() {
            0.0
        } else {
            drift_values.iter().sum::<f64>() / drift_values.len() as f64
        }
    }

    /// Check if drift exceeds the threshold
    pub fn is_drift_detected(&self, drift_score: f64) -> bool {
        drift_score > self.threshold
    }

    /// Identify metrics that have changed significantly (drift > 0.1)
    pub fn identify_changed_metrics(
        &self,
        baseline: &HashMap<String, f64>,
        recent: &HashMap<String, f64>,
    ) -> Vec<String> {
        let metric_keys = vec!["avg_duration", "error_rate", "success_rate", "throughput"];

        metric_keys
            .into_iter()
            .filter(|key| {
                let baseline_val = baseline.get(*key).copied().unwrap_or(0.0);
                let recent_val = recent.get(*key).copied().unwrap_or(0.0);

                if baseline_val == 0.0 {
                    false
                } else {
                    (recent_val - baseline_val).abs() / baseline_val.abs() > 0.1
                }
            })
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift() {
        let calc = DriftCalculator::new();

        let baseline = vec![
            ("avg_duration".to_string(), 100.0),
            ("error_rate".to_string(), 0.05),
            ("success_rate".to_string(), 0.95),
            ("throughput".to_string(), 10.0),
        ]
        .into_iter()
        .collect();

        let recent = vec![
            ("avg_duration".to_string(), 100.0),
            ("error_rate".to_string(), 0.05),
            ("success_rate".to_string(), 0.95),
            ("throughput".to_string(), 10.0),
        ]
        .into_iter()
        .collect();

        let drift = calc.calculate_drift(&baseline, &recent);
        assert_eq!(drift, 0.0);
        assert!(!calc.is_drift_detected(drift));
    }

    #[test]
    fn test_significant_drift() {
        let calc = DriftCalculator::new();

        let baseline = vec![
            ("avg_duration".to_string(), 100.0),
            ("error_rate".to_string(), 0.05),
            ("success_rate".to_string(), 0.95),
            ("throughput".to_string(), 10.0),
        ]
        .into_iter()
        .collect();

        let recent = vec![
            ("avg_duration".to_string(), 150.0), // 50% increase
            ("error_rate".to_string(), 0.10),    // 100% increase
            ("success_rate".to_string(), 0.90),  // 5.26% decrease
            ("throughput".to_string(), 5.0),     // 50% decrease
        ]
        .into_iter()
        .collect();

        let drift = calc.calculate_drift(&baseline, &recent);
        // drift = (0.5 + 1.0 + 0.0526 + 0.5) / 4 = 2.0526 / 4 ≈ 0.513
        assert!(drift > 0.4);
        assert!(calc.is_drift_detected(drift));
    }

    #[test]
    fn test_identify_changed_metrics() {
        let calc = DriftCalculator::new();

        let baseline = vec![
            ("avg_duration".to_string(), 100.0),
            ("error_rate".to_string(), 0.05),
            ("success_rate".to_string(), 0.95),
            ("throughput".to_string(), 10.0),
        ]
        .into_iter()
        .collect();

        let recent = vec![
            ("avg_duration".to_string(), 150.0), // 50% change
            ("error_rate".to_string(), 0.10),    // 100% change
            ("success_rate".to_string(), 0.95),  // 0% change
            ("throughput".to_string(), 10.0),    // 0% change
        ]
        .into_iter()
        .collect();

        let changed = calc.identify_changed_metrics(&baseline, &recent);
        assert_eq!(changed.len(), 2);
        assert!(changed.contains(&"avg_duration".to_string()));
        assert!(changed.contains(&"error_rate".to_string()));
    }

    #[test]
    fn test_empty_metrics() {
        let calc = DriftCalculator::new();
        let baseline: HashMap<String, f64> = HashMap::new();
        let recent: HashMap<String, f64> = HashMap::new();

        let drift = calc.calculate_drift(&baseline, &recent);
        assert_eq!(drift, 0.0);
    }

    #[test]
    fn test_custom_threshold() {
        let calc = DriftCalculator::with_threshold(0.5);
        let low_drift = 0.3;
        let high_drift = 0.7;

        assert!(!calc.is_drift_detected(low_drift));
        assert!(calc.is_drift_detected(high_drift));
    }
}
