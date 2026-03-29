//! Outcome Prediction

use crate::log::{EventLog, Trace};
use crate::observability::Tracing;
use std::collections::HashMap;

/// Outcome classification for a case
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaseOutcome {
    Successful,
    Problematic,
    Failed,
}

impl std::fmt::Display for CaseOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseOutcome::Successful => write!(f, "Successful"),
            CaseOutcome::Problematic => write!(f, "Problematic"),
            CaseOutcome::Failed => write!(f, "Failed"),
        }
    }
}

/// Risk assessment for an ongoing case
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub predicted_outcome: CaseOutcome,
    pub risk_score: f64,
    pub confidence: f64,
    pub anomalies: Vec<String>,
    pub recommendations: Vec<String>,
    pub similar_cases: usize,
}

impl RiskAssessment {
    pub fn new(
        predicted_outcome: CaseOutcome,
        risk_score: f64,
        confidence: f64,
        anomalies: Vec<String>,
        recommendations: Vec<String>,
        similar_cases: usize,
    ) -> Self {
        Self {
            predicted_outcome,
            risk_score: risk_score.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            anomalies,
            recommendations,
            similar_cases,
        }
    }
}

/// Outcome predictor for case classification
pub struct OutcomePredictor {
    successful_patterns: HashMap<String, usize>,
    problematic_patterns: HashMap<String, usize>,
    failed_patterns: HashMap<String, usize>,
    successful_duration_avg: f64,
    problematic_duration_avg: f64,
    successful_activity_freq: HashMap<String, usize>,
    problematic_activity_freq: HashMap<String, usize>,
    outcome_counts: HashMap<CaseOutcome, usize>,
}

impl OutcomePredictor {
    pub fn new<F>(log: &EventLog, outcome_fn: F) -> Self
    where
        F: Fn(&Trace) -> CaseOutcome,
    {
        let mut predictor = OutcomePredictor {
            successful_patterns: HashMap::new(),
            problematic_patterns: HashMap::new(),
            failed_patterns: HashMap::new(),
            successful_duration_avg: 0.0,
            problematic_duration_avg: 0.0,
            successful_activity_freq: HashMap::new(),
            problematic_activity_freq: HashMap::new(),
            outcome_counts: HashMap::new(),
        };

        predictor.build_model(log, outcome_fn);
        predictor
    }

    fn build_model<F>(&mut self, log: &EventLog, outcome_fn: F)
    where
        F: Fn(&Trace) -> CaseOutcome,
    {
        let mut successful_durations = Vec::new();
        let mut problematic_durations = Vec::new();

        for trace in &log.traces {
            let outcome = outcome_fn(trace);
            *self.outcome_counts.entry(outcome).or_insert(0) += 1;

            let activity_sequence = trace
                .events
                .iter()
                .map(|e| e.activity.clone())
                .collect::<Vec<_>>()
                .join("->");

            if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
                let duration = (last.timestamp - first.timestamp).num_seconds() as f64;

                match outcome {
                    CaseOutcome::Successful => {
                        successful_durations.push(duration);
                        *self
                            .successful_patterns
                            .entry(activity_sequence.clone())
                            .or_insert(0) += 1;
                        for event in &trace.events {
                            *self
                                .successful_activity_freq
                                .entry(event.activity.clone())
                                .or_insert(0) += 1;
                        }
                    }
                    CaseOutcome::Problematic => {
                        problematic_durations.push(duration);
                        *self
                            .problematic_patterns
                            .entry(activity_sequence.clone())
                            .or_insert(0) += 1;
                        for event in &trace.events {
                            *self
                                .problematic_activity_freq
                                .entry(event.activity.clone())
                                .or_insert(0) += 1;
                        }
                    }
                    CaseOutcome::Failed => {
                        *self.failed_patterns.entry(activity_sequence).or_insert(0) += 1;
                    }
                }
            }
        }

        if !successful_durations.is_empty() {
            self.successful_duration_avg =
                successful_durations.iter().sum::<f64>() / successful_durations.len() as f64;
        }

        if !problematic_durations.is_empty() {
            self.problematic_duration_avg =
                problematic_durations.iter().sum::<f64>() / problematic_durations.len() as f64;
        }
    }

    pub fn assess_risk(&self, partial_trace: &Trace) -> RiskAssessment {
        let mut anomalies = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_score: f64 = 0.0;

        if partial_trace.is_empty() {
            return RiskAssessment::new(
                CaseOutcome::Successful,
                0.0,
                0.0,
                anomalies,
                recommendations,
                0,
            );
        }

        for event in &partial_trace.events {
            let prob_freq = self
                .problematic_activity_freq
                .get(&event.activity)
                .copied()
                .unwrap_or(0);
            let succ_freq = self
                .successful_activity_freq
                .get(&event.activity)
                .copied()
                .unwrap_or(0);

            if prob_freq > succ_freq && prob_freq > 0 {
                anomalies.push(format!(
                    "Activity '{}' appears more in problematic cases",
                    event.activity
                ));
                risk_score += 0.1;
            }
        }

        if let (Some(first), Some(last)) =
            (partial_trace.events.first(), partial_trace.events.last())
        {
            let current_duration = (last.timestamp - first.timestamp).num_seconds() as f64;
            if current_duration > self.successful_duration_avg * 1.5 {
                anomalies.push("Case is taking significantly longer than typical".to_string());
                recommendations.push("Review for blockers or delays".to_string());
                risk_score += 0.15;
            }
        }

        let activity_sequence = partial_trace
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<Vec<_>>()
            .join("->");

        let failed_count = self
            .failed_patterns
            .get(&activity_sequence)
            .copied()
            .unwrap_or(0);
        let successful_count = self
            .successful_patterns
            .get(&activity_sequence)
            .copied()
            .unwrap_or(0);
        let problematic_count = self
            .problematic_patterns
            .get(&activity_sequence)
            .copied()
            .unwrap_or(0);

        if failed_count > 0 && failed_count > successful_count {
            anomalies.push("This activity sequence has led to failures".to_string());
            recommendations.push("Consider alternative paths or escalation".to_string());
            risk_score += 0.25;
        }

        let similar_cases = successful_count.max(problematic_count).max(failed_count);

        let (predicted_outcome, confidence) = if risk_score > 0.5 {
            (
                CaseOutcome::Problematic,
                if risk_score > 1.0 { 1.0 } else { risk_score },
            )
        } else if risk_score > 0.25 {
            (CaseOutcome::Problematic, 0.6)
        } else {
            (CaseOutcome::Successful, 1.0 - risk_score)
        };

        if recommendations.is_empty() && !anomalies.is_empty() {
            recommendations.push("Monitor case progress closely".to_string());
        }

        RiskAssessment::new(
            predicted_outcome,
            risk_score,
            confidence,
            anomalies,
            recommendations,
            similar_cases,
        )
    }

    /// Assess risk and emit an OTEL span via the provided Tracing instance.
    ///
    /// Span name: `process.mining.prediction.make`
    /// Attributes set:
    /// - `process.mining.prediction.model_type` = `"outcome_classifier"`
    /// - `process.mining.prediction.confidence` = risk assessment confidence score
    /// - `process.mining.case_id` = partial_trace case id
    pub fn assess_risk_traced(&self, partial_trace: &Trace, tracing: &Tracing) -> RiskAssessment {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert(
            "process.mining.prediction.model_type".to_string(),
            "outcome_classifier".to_string(),
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

        let result = self.assess_risk(partial_trace);

        // Record confidence and outcome as attributes
        span.attributes.insert(
            "process.mining.prediction.confidence".to_string(),
            format!("{:.4}", result.confidence),
        );
        span.attributes.insert(
            "process.mining.prediction.risk_score".to_string(),
            format!("{:.4}", result.risk_score),
        );
        span.attributes.insert(
            "process.mining.prediction.outcome".to_string(),
            result.predicted_outcome.to_string(),
        );

        tracing
            .end_span(&mut span, "ok", None)
            .expect("tracing end_span must not fail");

        result
    }

    pub fn detect_anomalies(&self, partial_trace: &Trace) -> Vec<String> {
        self.assess_risk(partial_trace).anomalies
    }

    pub fn outcome_distribution(&self) -> HashMap<CaseOutcome, f64> {
        let total = self.outcome_counts.values().sum::<usize>();
        let mut distribution = HashMap::new();

        if total > 0 {
            for (outcome, count) in &self.outcome_counts {
                distribution.insert(*outcome, *count as f64 / total as f64);
            }
        }

        distribution
    }

    pub fn get_success_indicators(&self, top_k: usize) -> Vec<(String, f64)> {
        let mut indicators = Vec::new();

        for (activity, freq) in &self.successful_activity_freq {
            let prob_freq = self
                .problematic_activity_freq
                .get(activity)
                .copied()
                .unwrap_or(1);
            let ratio = *freq as f64 / (prob_freq as f64 + 1.0);
            indicators.push((activity.clone(), ratio));
        }

        indicators.sort_by(|a, b| b.1.total_cmp(&a.1));
        indicators.truncate(top_k);
        indicators
    }

    pub fn get_problem_indicators(&self, top_k: usize) -> Vec<(String, f64)> {
        let mut indicators = Vec::new();

        for (activity, freq) in &self.problematic_activity_freq {
            let succ_freq = self
                .successful_activity_freq
                .get(activity)
                .copied()
                .unwrap_or(1);
            let ratio = *freq as f64 / (succ_freq as f64 + 1.0);
            indicators.push((activity.clone(), ratio));
        }

        indicators.sort_by(|a, b| b.1.total_cmp(&a.1));
        indicators.truncate(top_k);
        indicators
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::{Duration, Utc};

    fn create_labeled_log() -> EventLog {
        let mut log = EventLog::new();

        let mut trace1 = Trace::new("case_1");
        let now = Utc::now();
        trace1.add_event(Event::new("Start", now));
        trace1.add_event(Event::new("Validate", now + Duration::minutes(5)));
        trace1.add_event(Event::new("Process", now + Duration::minutes(15)));
        trace1.add_event(Event::new("Review", now + Duration::minutes(25)));
        trace1.add_event(Event::new("Complete", now + Duration::minutes(30)));
        trace1 = trace1.with_attribute("outcome", "success");
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("Start", now));
        trace2.add_event(Event::new("Validate", now + Duration::minutes(4)));
        trace2.add_event(Event::new("Process", now + Duration::minutes(12)));
        trace2.add_event(Event::new("Review", now + Duration::minutes(20)));
        trace2.add_event(Event::new("Complete", now + Duration::minutes(25)));
        trace2 = trace2.with_attribute("outcome", "success");
        log.add_trace(trace2);

        let mut trace3 = Trace::new("case_3");
        trace3.add_event(Event::new("Start", now));
        trace3.add_event(Event::new("Retry", now + Duration::minutes(10)));
        trace3.add_event(Event::new("Validate", now + Duration::minutes(30)));
        trace3.add_event(Event::new("Process", now + Duration::minutes(60)));
        trace3.add_event(Event::new("Complete", now + Duration::minutes(90)));
        trace3 = trace3.with_attribute("outcome", "problematic");
        log.add_trace(trace3);

        log
    }

    fn classify_outcome(trace: &Trace) -> CaseOutcome {
        match trace.get_attribute("outcome") {
            Some("success") => CaseOutcome::Successful,
            Some("problematic") => CaseOutcome::Problematic,
            Some("failed") => CaseOutcome::Failed,
            _ => CaseOutcome::Successful,
        }
    }

    #[test]
    fn test_predictor_creation() {
        let log = create_labeled_log();
        let predictor = OutcomePredictor::new(&log, classify_outcome);
        assert!(!predictor.successful_patterns.is_empty());
    }

    #[test]
    fn test_assess_risk_successful_case() {
        let log = create_labeled_log();
        let predictor = OutcomePredictor::new(&log, classify_outcome);

        let mut partial_trace = Trace::new("case_ongoing");
        let now = Utc::now();
        partial_trace.add_event(Event::new("Start", now));
        partial_trace.add_event(Event::new("Validate", now + Duration::minutes(5)));
        partial_trace.add_event(Event::new("Process", now + Duration::minutes(15)));

        let assessment = predictor.assess_risk(&partial_trace);
        assert!(assessment.risk_score < 0.5);
    }

    #[test]
    fn test_detect_anomalies() {
        let log = create_labeled_log();
        let predictor = OutcomePredictor::new(&log, classify_outcome);

        let mut partial_trace = Trace::new("case_anomaly");
        let now = Utc::now();
        partial_trace.add_event(Event::new("Start", now));
        partial_trace.add_event(Event::new("Retry", now + Duration::minutes(10)));

        let _anomalies = predictor.detect_anomalies(&partial_trace);
    }

    #[test]
    fn test_outcome_distribution() {
        let log = create_labeled_log();
        let predictor = OutcomePredictor::new(&log, classify_outcome);
        let distribution = predictor.outcome_distribution();
        assert!(!distribution.is_empty());
        let total: f64 = distribution.values().sum();
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_get_success_indicators() {
        let log = create_labeled_log();
        let predictor = OutcomePredictor::new(&log, classify_outcome);
        let indicators = predictor.get_success_indicators(5);
        assert!(!indicators.is_empty());
    }

    #[test]
    fn test_get_problem_indicators() {
        let log = create_labeled_log();
        let predictor = OutcomePredictor::new(&log, classify_outcome);
        let _indicators = predictor.get_problem_indicators(5);
    }

    #[test]
    fn test_risk_assessment_structure() {
        let log = create_labeled_log();
        let predictor = OutcomePredictor::new(&log, classify_outcome);

        let mut partial_trace = Trace::new("case_test");
        let now = Utc::now();
        partial_trace.add_event(Event::new("Start", now));

        let assessment = predictor.assess_risk(&partial_trace);
        assert!(assessment.risk_score >= 0.0 && assessment.risk_score <= 1.0);
        assert!(assessment.confidence >= 0.0 && assessment.confidence <= 1.0);
    }
}
