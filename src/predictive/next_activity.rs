//! Next Activity Prediction

use crate::log::{EventLog, Trace};
use crate::observability::Tracing;
use std::collections::HashMap;

/// Prediction for the next activity
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityPrediction {
    pub activity: String,
    pub probability: f64,
    pub frequency: usize,
}

impl ActivityPrediction {
    pub fn new(activity: String, probability: f64, frequency: usize) -> Self {
        Self {
            activity,
            probability,
            frequency,
        }
    }
}

/// Next activity predictor using Markov chain models
pub struct NextActivityPredictor {
    transitions: HashMap<String, HashMap<String, usize>>,
    transition_totals: HashMap<String, usize>,
    start_activities: HashMap<String, usize>,
}

impl NextActivityPredictor {
    pub fn new(log: &EventLog) -> Self {
        let mut predictor = NextActivityPredictor {
            transitions: HashMap::new(),
            transition_totals: HashMap::new(),
            start_activities: HashMap::new(),
        };
        predictor.build_model(log);
        predictor
    }

    fn build_model(&mut self, log: &EventLog) {
        for trace in &log.traces {
            if trace.is_empty() {
                continue;
            }

            let first_activity = &trace.events[0].activity;
            *self
                .start_activities
                .entry(first_activity.clone())
                .or_insert(0) += 1;

            for i in 0..trace.events.len() - 1 {
                let current = &trace.events[i].activity;
                let next = &trace.events[i + 1].activity;

                self.transitions
                    .entry(current.clone())
                    .or_default()
                    .entry(next.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);

                *self.transition_totals.entry(current.clone()).or_insert(0) += 1;
            }
        }
    }

    pub fn predict_next_activity(
        &self,
        partial_trace: &Trace,
        top_k: usize,
    ) -> Vec<ActivityPrediction> {
        if partial_trace.is_empty() {
            return self.get_start_activities(top_k);
        }
        let last_activity = &partial_trace.events[partial_trace.len() - 1].activity;
        self.predict_from_activity(last_activity, top_k)
    }

    /// Predict next activity and emit an OTEL span via the provided Tracing instance.
    ///
    /// Span name: `process.mining.prediction.make`
    /// Attributes set:
    /// - `process.mining.prediction.model_type` = `"markov_chain"`
    /// - `process.mining.prediction.case_length` = number of events in partial_trace
    /// - `process.mining.case_id` = partial_trace case id
    pub fn predict_next_activity_traced(
        &self,
        partial_trace: &Trace,
        top_k: usize,
        tracing: &Tracing,
    ) -> Vec<ActivityPrediction> {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert(
            "process.mining.prediction.model_type".to_string(),
            "markov_chain".to_string(),
        );
        attrs.insert(
            "process.mining.prediction.case_length".to_string(),
            partial_trace.len().to_string(),
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

        let result = self.predict_next_activity(partial_trace, top_k);

        // Record result count as attribute before ending span
        {
            let stored = tracing.get_span(&span.span_id);
            if let Some(mut s) = stored {
                s.attributes.insert(
                    "process.mining.prediction.result_count".to_string(),
                    result.len().to_string(),
                );
                // re-insert updated copy (start_span already stored; we update via end_span)
                let _ = s; // attributes updated on span local copy below
            }
        }
        span.attributes.insert(
            "process.mining.prediction.result_count".to_string(),
            result.len().to_string(),
        );

        tracing
            .end_span(&mut span, "ok", None)
            .expect("tracing end_span must not fail");

        result
    }

    pub fn predict_from_activity(
        &self,
        current_activity: &str,
        top_k: usize,
    ) -> Vec<ActivityPrediction> {
        let mut predictions = Vec::new();

        if let Some(next_activities) = self.transitions.get(current_activity) {
            if let Some(total) = self.transition_totals.get(current_activity) {
                for (activity, count) in next_activities {
                    let probability = *count as f64 / *total as f64;
                    predictions.push(ActivityPrediction::new(
                        activity.clone(),
                        probability,
                        *count,
                    ));
                }
            }
        }

        predictions.sort_by(|a, b| b.probability.total_cmp(&a.probability));
        predictions.truncate(top_k);
        predictions
    }

    pub fn get_start_activities(&self, top_k: usize) -> Vec<ActivityPrediction> {
        let mut predictions = Vec::new();
        let total_starts: usize = self.start_activities.values().sum();

        if total_starts == 0 {
            return predictions;
        }

        for (activity, count) in &self.start_activities {
            let probability = *count as f64 / total_starts as f64;
            predictions.push(ActivityPrediction::new(
                activity.clone(),
                probability,
                *count,
            ));
        }

        predictions.sort_by(|a, b| b.probability.total_cmp(&a.probability));
        predictions.truncate(top_k);
        predictions
    }

    pub fn has_transition(&self, from: &str, to: &str) -> bool {
        self.transitions
            .get(from)
            .map(|nexts| nexts.contains_key(to))
            .unwrap_or(false)
    }

    pub fn transition_probability(&self, from: &str, to: &str) -> f64 {
        if let Some(next_activities) = self.transitions.get(from) {
            if let Some(count) = next_activities.get(to) {
                if let Some(total) = self.transition_totals.get(from) {
                    return *count as f64 / *total as f64;
                }
            }
        }
        0.0
    }

    pub fn num_unique_activities(&self) -> usize {
        let mut activities = std::collections::HashSet::new();
        for activity in self.transitions.keys() {
            activities.insert(activity.clone());
        }
        for activity in self.transitions.values().flat_map(|m| m.keys()) {
            activities.insert(activity.clone());
        }
        activities.len()
    }

    pub fn num_transitions(&self) -> usize {
        self.transition_totals.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_sample_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("Start", now));
        trace1.add_event(Event::new("Activity_A", now));
        trace1.add_event(Event::new("Activity_B", now));
        trace1.add_event(Event::new("End", now));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("Start", now));
        trace2.add_event(Event::new("Activity_A", now));
        trace2.add_event(Event::new("Activity_C", now));
        trace2.add_event(Event::new("End", now));
        log.add_trace(trace2);

        let mut trace3 = Trace::new("case_3");
        trace3.add_event(Event::new("Start", now));
        trace3.add_event(Event::new("Activity_A", now));
        trace3.add_event(Event::new("Activity_B", now));
        trace3.add_event(Event::new("Activity_C", now));
        trace3.add_event(Event::new("End", now));
        log.add_trace(trace3);

        log
    }

    #[test]
    fn test_predictor_creation() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);
        assert!(predictor.transitions.contains_key("Start"));
    }

    #[test]
    fn test_predict_next_activity_from_start() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);

        let mut partial_trace = Trace::new("case_ongoing");
        let now = Utc::now();
        partial_trace.add_event(Event::new("Start", now));

        let predictions = predictor.predict_next_activity(&partial_trace, 5);
        assert!(!predictions.is_empty());
        assert_eq!(predictions[0].activity, "Activity_A");
    }

    #[test]
    fn test_predict_from_activity() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);
        let predictions = predictor.predict_from_activity("Activity_A", 10);
        assert!(predictions.len() >= 2);
    }

    #[test]
    fn test_transition_probability() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);
        let prob = predictor.transition_probability("Start", "Activity_A");
        assert_eq!(prob, 1.0);
    }

    #[test]
    fn test_has_transition() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);
        assert!(predictor.has_transition("Start", "Activity_A"));
        assert!(!predictor.has_transition("Activity_A", "Nonexistent"));
    }

    #[test]
    fn test_get_start_activities() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);
        let start_activities = predictor.get_start_activities(10);
        assert!(!start_activities.is_empty());
        assert_eq!(start_activities[0].activity, "Start");
    }

    #[test]
    fn test_num_unique_activities() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);
        let num_activities = predictor.num_unique_activities();
        assert!(num_activities >= 5);
    }

    #[test]
    fn test_predict_empty_trace() {
        let log = create_sample_log();
        let predictor = NextActivityPredictor::new(&log);
        let empty_trace = Trace::new("empty");
        let predictions = predictor.predict_next_activity(&empty_trace, 5);
        assert!(!predictions.is_empty());
    }
}
