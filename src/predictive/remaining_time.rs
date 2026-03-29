//! Remaining Time Prediction

use crate::log::{EventLog, Trace};
use crate::observability::Tracing;
use std::collections::HashMap;

/// Result of a remaining time prediction
#[derive(Debug, Clone)]
pub struct RemainingTimePrediction {
    pub predicted_seconds: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
    pub confidence: f64,
    pub sample_size: usize,
}

impl RemainingTimePrediction {
    pub fn new(
        predicted_seconds: f64,
        confidence_lower: f64,
        confidence_upper: f64,
        confidence: f64,
        sample_size: usize,
    ) -> Self {
        Self {
            predicted_seconds,
            confidence_lower,
            confidence_upper,
            confidence,
            sample_size,
        }
    }

    pub fn predicted_hours(&self) -> f64 {
        self.predicted_seconds / 3600.0
    }

    pub fn predicted_days(&self) -> f64 {
        self.predicted_seconds / 86400.0
    }
}

/// Predictor for remaining time until case completion
pub struct RemainingTimePredictor {
    activity_durations: HashMap<String, Vec<f64>>,
    case_durations: Vec<f64>,
    sequence_durations: HashMap<String, Vec<f64>>,
    /// Maps activity-prefix key → list of completed-case total durations.
    ///
    /// The prefix key is the sorted, comma-joined sequence of activity names
    /// seen before the final event in each completed trace, e.g. `"A,B"` for a
    /// trace [A, B, End].  This lets `calculate_confidence_interval` narrow the
    /// standard deviation by looking only at cases that followed the same path.
    prefix_durations: HashMap<String, Vec<f64>>,
}

impl RemainingTimePredictor {
    pub fn new(log: &EventLog) -> Self {
        let mut predictor = RemainingTimePredictor {
            activity_durations: HashMap::new(),
            case_durations: Vec::new(),
            sequence_durations: HashMap::new(),
            prefix_durations: HashMap::new(),
        };
        predictor.build_model(log);
        predictor
    }

    fn build_model(&mut self, log: &EventLog) {
        for trace in &log.traces {
            if trace.len() < 2 {
                continue;
            }

            let events_sorted = trace.events_sorted();
            if let (Some(first), Some(last)) = (events_sorted.first(), events_sorted.last()) {
                let duration_secs = (last.timestamp - first.timestamp).num_seconds() as f64;
                self.case_durations.push(duration_secs);

                // Build a prefix key from all activities except the last one.
                // This represents "what activities were seen before the case ended".
                let prefix_activities: Vec<&str> = events_sorted[..events_sorted.len() - 1]
                    .iter()
                    .map(|e| e.activity.as_str())
                    .collect();
                let prefix_key = prefix_activities.join(",");
                self.prefix_durations
                    .entry(prefix_key)
                    .or_default()
                    .push(duration_secs);
            }

            for i in 0..trace.events.len() {
                let current_activity = &trace.events[i].activity;

                if i + 1 < trace.events.len() {
                    let next_timestamp = trace.events[i + 1].timestamp;
                    let current_timestamp = trace.events[i].timestamp;
                    let duration_secs = (next_timestamp - current_timestamp).num_seconds() as f64;

                    self.activity_durations
                        .entry(current_activity.clone())
                        .or_default()
                        .push(duration_secs);

                    let next_activity = &trace.events[i + 1].activity;
                    let sequence = format!("{}->{}", current_activity, next_activity);

                    self.sequence_durations
                        .entry(sequence)
                        .or_default()
                        .push(duration_secs);
                }
            }
        }
    }

    pub fn predict_remaining_time(
        &self,
        partial_trace: &Trace,
        remaining_activities: Option<Vec<String>>,
    ) -> Option<RemainingTimePrediction> {
        if partial_trace.is_empty() || self.case_durations.is_empty() {
            return None;
        }

        let events_sorted = partial_trace.events_sorted();
        let last_event = events_sorted.last()?;
        let first_event = events_sorted.first()?;

        let elapsed_secs = (last_event.timestamp - first_event.timestamp).num_seconds() as f64;

        let mut remaining_time = 0.0;

        if let Some(remaining_acts) = remaining_activities {
            for activity in &remaining_acts {
                if let Some(durations) = self.activity_durations.get(activity) {
                    if !durations.is_empty() {
                        let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
                        remaining_time += avg_duration;
                    }
                }
            }
        } else {
            let avg_case_duration =
                self.case_durations.iter().sum::<f64>() / self.case_durations.len() as f64;
            remaining_time = (avg_case_duration - elapsed_secs).max(0.0);
        }

        // Build the prefix key from the partial trace's activity sequence
        // (all events in sorted order) so we can look up the right bucket.
        let events_sorted_partial = partial_trace.events_sorted();
        let prefix_key = events_sorted_partial
            .iter()
            .map(|e| e.activity.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let (confidence_lower, confidence_upper, confidence) =
            self.calculate_confidence_interval(remaining_time, &prefix_key);

        Some(RemainingTimePrediction::new(
            remaining_time,
            confidence_lower,
            confidence_upper,
            confidence,
            self.case_durations.len(),
        ))
    }

    /// Compute a 95 % confidence interval around `predicted_value`.
    ///
    /// If `prefix_key` matches a bucket in `self.prefix_durations` (i.e. there
    /// are completed cases whose activity sequence starts with the same
    /// activities as the current case), we use that bucket's std_dev.  This
    /// produces a narrower interval than the overall std_dev when the prefix
    /// isolates a cluster of similar cases.  Falls back to the overall
    /// `case_durations` std_dev when no matching bucket exists.
    fn calculate_confidence_interval(
        &self,
        predicted_value: f64,
        prefix_key: &str,
    ) -> (f64, f64, f64) {
        if self.case_durations.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        // Choose the duration sample: prefer prefix-specific bucket when
        // it has at least 2 observations (needed for a meaningful std_dev).
        let sample: &[f64] = self
            .prefix_durations
            .get(prefix_key)
            .filter(|v| v.len() >= 2)
            .map(|v| v.as_slice())
            .unwrap_or(&self.case_durations);

        let n = sample.len() as f64;
        let mean = sample.iter().sum::<f64>() / n;
        let variance = sample.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let margin = 1.96 * std_dev;
        let lower = (predicted_value - margin).max(0.0);
        let upper = predicted_value + margin;

        // Confidence score: fraction of the 100-case "full" sample, capped at 1.
        let confidence = (self.case_durations.len() as f64).min(100.0) / 100.0;

        (lower, upper, confidence)
    }

    /// Predict remaining time and emit an OTEL span via the provided Tracing instance.
    ///
    /// Span name: `process.mining.prediction.make`
    /// Attributes set:
    /// - `process.mining.prediction.model_type` = `"remaining_time"`
    /// - `process.mining.case_id` = partial_trace case id
    /// - `process.mining.prediction.remaining_time_s` = predicted seconds (if Some)
    /// - `process.mining.prediction.confidence` = confidence score (if Some)
    pub fn predict_remaining_time_traced(
        &self,
        partial_trace: &Trace,
        remaining_activities: Option<Vec<String>>,
        tracing: &Tracing,
    ) -> Option<RemainingTimePrediction> {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert(
            "process.mining.prediction.model_type".to_string(),
            "remaining_time".to_string(),
        );
        attrs.insert(
            "process.mining.case_id".to_string(),
            partial_trace.id.clone(),
        );

        let mut span = tracing
            .start_span(
                crate::semconv::process_mining_span_names::PROCESS_MINING_PREDICTION_MAKE_SPAN,
                attrs,
                None,
            )
            .expect("tracing start_span must not fail");

        let result = self.predict_remaining_time(partial_trace, remaining_activities);

        if let Some(ref pred) = result {
            span.attributes.insert(
                "process.mining.prediction.remaining_time_s".to_string(),
                format!("{:.2}", pred.predicted_seconds),
            );
            span.attributes.insert(
                "process.mining.prediction.confidence".to_string(),
                format!("{:.4}", pred.confidence),
            );
            span.attributes.insert(
                "process.mining.prediction.sample_size".to_string(),
                pred.sample_size.to_string(),
            );
        }

        tracing
            .end_span(&mut span, "ok", None)
            .expect("tracing end_span must not fail");

        result
    }

    pub fn average_case_duration(&self) -> Option<f64> {
        if self.case_durations.is_empty() {
            None
        } else {
            Some(self.case_durations.iter().sum::<f64>() / self.case_durations.len() as f64)
        }
    }

    pub fn average_activity_duration(&self, activity: &str) -> Option<f64> {
        self.activity_durations.get(activity).and_then(|durations| {
            if durations.is_empty() {
                None
            } else {
                Some(durations.iter().sum::<f64>() / durations.len() as f64)
            }
        })
    }
}

// ── MCP-compatible public API ────────────────────────────────────────────────

/// Flat response struct for remaining-time predictions — suitable for JSON
/// serialisation in MCP tool responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemainingTimePredictionResponse {
    /// Predicted number of seconds remaining until case completion.
    pub predicted_remaining_seconds: f64,
    /// Confidence score in [0.0, 1.0]; higher = more training cases.
    pub confidence: f64,
    /// Number of historical cases used to build the prediction model.
    pub similar_cases_count: usize,
    /// Lower bound of the 95 % confidence interval (seconds).
    pub percentile_10: f64,
    /// Upper bound of the 95 % confidence interval (seconds).
    pub percentile_90: f64,
}

/// Predict remaining time using a training log and a slice of partial-trace events.
///
/// Builds a [`RemainingTimePredictor`] from `training_log`, constructs a
/// synthetic [`Trace`] from `partial_trace_events`, and returns a
/// [`RemainingTimePredictionResponse`].
///
/// Returns `None` when `training_log` is empty or `partial_trace_events` is empty.
pub fn predict_remaining_time_from_log(
    training_log: &EventLog,
    partial_trace_events: &[crate::log::Event],
) -> Option<RemainingTimePredictionResponse> {
    if partial_trace_events.is_empty() {
        return None;
    }

    let predictor = RemainingTimePredictor::new(training_log);

    let mut trace = Trace::new("__partial__");
    for event in partial_trace_events {
        trace.add_event(event.clone());
    }

    let pred = predictor.predict_remaining_time(&trace, None)?;

    Some(RemainingTimePredictionResponse {
        predicted_remaining_seconds: pred.predicted_seconds,
        confidence: pred.confidence,
        similar_cases_count: pred.sample_size,
        percentile_10: pred.confidence_lower,
        percentile_90: pred.confidence_upper,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::{Duration as ChronoDuration, Utc};

    fn create_sample_log() -> EventLog {
        let mut log = EventLog::new();

        let mut trace1 = Trace::new("case_1");
        let start = Utc::now();
        trace1.add_event(Event::new("Start", start));
        trace1.add_event(Event::new(
            "Activity_A",
            start + ChronoDuration::minutes(10),
        ));
        trace1.add_event(Event::new(
            "Activity_B",
            start + ChronoDuration::minutes(30),
        ));
        trace1.add_event(Event::new("End", start + ChronoDuration::minutes(60)));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        let start = Utc::now();
        trace2.add_event(Event::new("Start", start));
        trace2.add_event(Event::new(
            "Activity_A",
            start + ChronoDuration::minutes(15),
        ));
        trace2.add_event(Event::new(
            "Activity_B",
            start + ChronoDuration::minutes(45),
        ));
        trace2.add_event(Event::new("End", start + ChronoDuration::minutes(90)));
        log.add_trace(trace2);

        let mut trace3 = Trace::new("case_3");
        let start = Utc::now();
        trace3.add_event(Event::new("Start", start));
        trace3.add_event(Event::new(
            "Activity_A",
            start + ChronoDuration::minutes(12),
        ));
        trace3.add_event(Event::new(
            "Activity_B",
            start + ChronoDuration::minutes(40),
        ));
        trace3.add_event(Event::new("End", start + ChronoDuration::minutes(75)));
        log.add_trace(trace3);

        log
    }

    #[test]
    fn test_predictor_creation() {
        let log = create_sample_log();
        let predictor = RemainingTimePredictor::new(&log);
        assert_eq!(predictor.case_durations.len(), 3);
    }

    #[test]
    fn test_average_case_duration() {
        let log = create_sample_log();
        let predictor = RemainingTimePredictor::new(&log);
        let avg = predictor
            .average_case_duration()
            .expect("average_case_duration should return Some for non-empty log");
        assert!((avg - 4500.0).abs() < 1.0);
    }

    #[test]
    fn test_predict_remaining_time_with_activities() {
        let log = create_sample_log();
        let predictor = RemainingTimePredictor::new(&log);

        let mut partial_trace = Trace::new("case_ongoing");
        let start = Utc::now();
        partial_trace.add_event(Event::new("Start", start));
        partial_trace.add_event(Event::new(
            "Activity_A",
            start + ChronoDuration::minutes(10),
        ));

        let remaining = predictor.predict_remaining_time(
            &partial_trace,
            Some(vec!["Activity_B".to_string(), "End".to_string()]),
        );

        assert!(remaining.is_some());
        let pred = remaining.expect("predict_remaining_time should return Some for partial trace");
        assert!(pred.predicted_seconds > 0.0);
    }

    #[test]
    fn test_predict_remaining_time_without_activities() {
        let log = create_sample_log();
        let predictor = RemainingTimePredictor::new(&log);

        let mut partial_trace = Trace::new("case_ongoing");
        let start = Utc::now();
        partial_trace.add_event(Event::new("Start", start));
        partial_trace.add_event(Event::new(
            "Activity_A",
            start + ChronoDuration::minutes(20),
        ));

        let remaining = predictor.predict_remaining_time(&partial_trace, None);
        assert!(remaining.is_some());
        let pred = remaining.expect("predict_remaining_time should return Some for partial trace");
        assert_eq!(pred.sample_size, 3);
    }

    #[test]
    fn test_average_activity_duration() {
        let log = create_sample_log();
        let predictor = RemainingTimePredictor::new(&log);
        let avg = predictor.average_activity_duration("Start");
        assert!(avg.is_some());
    }

    #[test]
    fn test_confidence_interval_calculation() {
        let log = create_sample_log();
        let predictor = RemainingTimePredictor::new(&log);
        let (lower, upper, confidence) = predictor.calculate_confidence_interval(4500.0, "");
        assert!(lower >= 0.0);
        assert!(upper > lower);
        assert!(confidence > 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_empty_log_prediction() {
        let log = EventLog::new();
        let predictor = RemainingTimePredictor::new(&log);

        let mut partial_trace = Trace::new("case_ongoing");
        let start = Utc::now();
        partial_trace.add_event(Event::new("Start", start));

        let remaining = predictor.predict_remaining_time(&partial_trace, None);
        assert!(remaining.is_none());
    }
}
