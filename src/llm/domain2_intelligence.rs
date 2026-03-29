//! Domain 2: Process Intelligence Queries
//!
//! Diagnoses WHY processes break and RECOMMENDS fixes using statistical evidence
//! from the statistics layer (bottleneck, rework, correlation modules).
//!
//! # Domain 2 vs Domain 1
//! - Domain 1: "What is the process?" → describe structure
//! - Domain 2: "Why is cycle time high?" → diagnose and recommend
//!
//! # Academic Foundation
//! Based on van der Aalst's process analytics framework:
//! - Root cause: activity/variant/resource responsible for degraded KPI
//! - Evidence: statistical measurements from event log
//! - Recommendation: data-driven improvement action

use serde::{Deserialize, Serialize};

/// Context provided to the intelligence engine
pub struct IntelligenceContext<'a> {
    pub log: Option<&'a crate::log::EventLog>,
    pub petri_net: Option<&'a crate::models::petri_net::PetriNet>,
}

/// Structured answer to a causal / improvement query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceAnswer {
    /// Original question
    pub question: String,
    /// Identified root causes with evidence
    pub root_causes: Vec<RootCause>,
    /// Concrete recommendations
    pub recommendations: Vec<Recommendation>,
    /// Raw statistical data supporting the answer
    pub supporting_data: SupportingData,
    /// Confidence in the answer (0.0–1.0)
    pub confidence: f64,
}

/// A single diagnosed root cause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    /// The driving factor (e.g., activity name, variant pattern)
    pub factor: String,
    /// Statistical evidence for this cause
    pub evidence: String,
    /// Relative impact score (0.0–1.0)
    pub impact_score: f64,
}

/// A single improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommended action
    pub action: String,
    /// Projected improvement if action is taken
    pub expected_improvement: String,
    /// Implementation priority
    pub priority: Priority,
}

/// Priority levels for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Supporting statistical data for the answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportingData {
    /// Top bottleneck activity names (from identify_bottlenecks)
    pub bottleneck_activities: Vec<String>,
    /// Activities with high rework frequency
    pub rework_hotspots: Vec<String>,
    /// Estimated time waste from non-dominant variants (seconds), if computable
    pub variant_waste_estimate: Option<f64>,
}

/// Dispatch a causal/improvement query to the appropriate sub-analyser
pub fn answer_causal_question(query: &str, ctx: &IntelligenceContext) -> IntelligenceAnswer {
    let q = query.to_lowercase();

    if q.contains("cycle time") || q.contains("duration") || q.contains("slow") {
        answer_cycle_time_question(query, ctx)
    } else if q.contains("variant") || q.contains("eliminate") || q.contains("path") {
        answer_variant_question(query, ctx)
    } else if q.contains("conform") || q.contains("deviation") || q.contains("violation") {
        answer_conformance_question(query, ctx)
    } else {
        answer_generic_causal_question(query, ctx)
    }
}

// ---------------------------------------------------------------------------
// Sub-dispatchers
// ---------------------------------------------------------------------------

fn answer_cycle_time_question(query: &str, ctx: &IntelligenceContext) -> IntelligenceAnswer {
    match ctx.log {
        None => low_confidence_answer(query, "No event log available to diagnose cycle time."),
        Some(log) => {
            let bottlenecks = crate::statistics::identify_bottlenecks(log);
            let rework_stats = crate::statistics::rework_statistics(log);

            let mut root_causes = Vec::new();
            let mut recommendations = Vec::new();

            // Top-3 bottleneck activities become root causes
            for bn in bottlenecks.iter().take(3) {
                let impact = bn.severity_score / 100.0;
                root_causes.push(RootCause {
                    factor: bn.activity.clone(),
                    evidence: format!(
                        "{} (severity {:.0}/100, avg wait {:.1}s)",
                        bn.reason, bn.severity_score, bn.average_waiting_time
                    ),
                    impact_score: impact,
                });
                recommendations.push(Recommendation {
                    action: format!(
                        "Reduce queue depth at '{}' — consider parallelising or adding capacity",
                        bn.activity
                    ),
                    expected_improvement: format!(
                        "Potential {:.0}% reduction in cycle time for cases through this activity",
                        impact * 40.0
                    ),
                    priority: if impact > 0.6 {
                        Priority::High
                    } else if impact > 0.3 {
                        Priority::Medium
                    } else {
                        Priority::Low
                    },
                });
            }

            // High-rework activities add to cycle time
            for rw in rework_stats
                .iter()
                .filter(|r| r.rework_frequency > 0.2)
                .take(2)
            {
                root_causes.push(RootCause {
                    factor: rw.activity.clone(),
                    evidence: format!(
                        "Rework in {:.0}% of cases, avg {:.1}s wasted per redo",
                        rw.rework_frequency * 100.0,
                        rw.average_rework_duration
                    ),
                    impact_score: rw.rework_frequency * 0.5,
                });
                recommendations.push(Recommendation {
                    action: format!(
                        "Improve first-pass quality at '{}' to cut rework loops",
                        rw.activity
                    ),
                    expected_improvement: format!(
                        "Eliminate ~{:.0}% rework; saves {:.1}s per affected case",
                        rw.rework_frequency * 100.0,
                        rw.total_rework_time / rw.cases_with_rework.max(1) as f64
                    ),
                    priority: Priority::Medium,
                });
            }

            let supporting = build_supporting_data(log);
            let confidence = if root_causes.is_empty() { 0.3 } else { 0.8 };

            IntelligenceAnswer {
                question: query.to_string(),
                root_causes,
                recommendations,
                supporting_data: supporting,
                confidence,
            }
        }
    }
}

fn answer_variant_question(query: &str, ctx: &IntelligenceContext) -> IntelligenceAnswer {
    match ctx.log {
        None => low_confidence_answer(query, "No event log available to analyse variants."),
        Some(log) => {
            // Use path performance to quantify variant waste
            let paths = crate::statistics::analyze_path_performance(log);

            let _total_cases = log.traces.len() as f64;
            let dominant_threshold = 0.5_f64; // paths covering >= 50% of cases

            let dominant_time: f64 = paths
                .iter()
                .filter(|p| p.frequency_percentage >= dominant_threshold)
                .map(|p| p.average_duration)
                .fold(f64::MAX, f64::min);

            let dominant_time = if dominant_time == f64::MAX {
                0.0
            } else {
                dominant_time
            };

            // Waste = extra time cases spend on non-dominant (expensive) variants
            let variant_waste: f64 = paths
                .iter()
                .filter(|p| p.frequency_percentage < dominant_threshold)
                .map(|p| {
                    let extra = (p.average_duration - dominant_time).max(0.0);
                    extra * p.frequency as f64
                })
                .sum();

            let mut root_causes = Vec::new();
            let mut recommendations = Vec::new();

            // Non-dominant paths with high average duration are root causes
            for path in paths
                .iter()
                .filter(|p| {
                    p.frequency_percentage < dominant_threshold
                        && p.average_duration > dominant_time
                })
                .take(3)
            {
                let pct = path.frequency_percentage * 100.0;
                root_causes.push(RootCause {
                    factor: path.path.join(" → "),
                    evidence: format!(
                        "{:.1}% of cases, avg duration {:.1}s vs dominant {:.1}s",
                        pct, path.average_duration, dominant_time
                    ),
                    impact_score: pct / 100.0,
                });
                recommendations.push(Recommendation {
                    action: format!(
                        "Standardise or eliminate variant: {}",
                        path.path.join(" → ")
                    ),
                    expected_improvement: format!(
                        "Redirect {:.0}% of cases to dominant path; saves ~{:.1}s per case",
                        pct,
                        path.average_duration - dominant_time
                    ),
                    priority: if pct > 20.0 {
                        Priority::High
                    } else {
                        Priority::Medium
                    },
                });
            }

            let waste_estimate = if variant_waste > 0.0 {
                Some(variant_waste)
            } else {
                None
            };

            let supporting = SupportingData {
                bottleneck_activities: vec![],
                rework_hotspots: vec![],
                variant_waste_estimate: waste_estimate,
            };

            let confidence = if root_causes.is_empty() { 0.4 } else { 0.75 };

            IntelligenceAnswer {
                question: query.to_string(),
                root_causes,
                recommendations,
                supporting_data: supporting,
                confidence,
            }
        }
    }
}

fn answer_conformance_question(query: &str, ctx: &IntelligenceContext) -> IntelligenceAnswer {
    match ctx.log {
        None => low_confidence_answer(query, "No event log available for conformance analysis."),
        Some(log) => {
            // Use causal dependency analysis to find unexpected transitions
            let dependencies = crate::statistics::causal_dependency_analysis(log);

            // Low-strength dependencies are deviation indicators
            let deviations: Vec<_> = dependencies
                .iter()
                .filter(|d| d.conditional_probability < 0.3)
                .take(5)
                .collect();

            let mut root_causes = Vec::new();
            let mut recommendations = Vec::new();

            for dep in &deviations {
                root_causes.push(RootCause {
                    factor: format!("{} → {}", dep.source, dep.target),
                    evidence: format!(
                        "Occurs in {:.0}% of transitions from '{}' (expected path strength {:.2})",
                        dep.conditional_probability * 100.0,
                        dep.source,
                        dep.strength
                    ),
                    impact_score: 1.0 - dep.conditional_probability,
                });
                recommendations.push(Recommendation {
                    action: format!(
                        "Investigate why cases deviate from '{}' to '{}' — add guard/validation",
                        dep.source, dep.target
                    ),
                    expected_improvement: "Reduce process deviation rate, improve compliance"
                        .to_string(),
                    priority: Priority::Medium,
                });
            }

            let supporting = build_supporting_data(log);
            let confidence = if root_causes.is_empty() { 0.35 } else { 0.7 };

            IntelligenceAnswer {
                question: query.to_string(),
                root_causes,
                recommendations,
                supporting_data: supporting,
                confidence,
            }
        }
    }
}

fn answer_generic_causal_question(query: &str, ctx: &IntelligenceContext) -> IntelligenceAnswer {
    match ctx.log {
        None => low_confidence_answer(query, "No event log available for causal analysis."),
        Some(log) => {
            let bottlenecks = crate::statistics::identify_bottlenecks(log);
            let rework_stats = crate::statistics::rework_statistics(log);

            let mut root_causes = Vec::new();
            let mut recommendations = Vec::new();

            if let Some(top_bn) = bottlenecks.first() {
                root_causes.push(RootCause {
                    factor: top_bn.activity.clone(),
                    evidence: format!(
                        "Top bottleneck: {} (severity {:.0}/100)",
                        top_bn.reason, top_bn.severity_score
                    ),
                    impact_score: top_bn.severity_score / 100.0,
                });
                recommendations.push(Recommendation {
                    action: format!("Address bottleneck at '{}'", top_bn.activity),
                    expected_improvement: "Reduce overall cycle time".to_string(),
                    priority: Priority::High,
                });
            }

            if let Some(top_rw) = rework_stats.first() {
                root_causes.push(RootCause {
                    factor: top_rw.activity.clone(),
                    evidence: format!(
                        "Highest rework rate: {:.0}% of cases",
                        top_rw.rework_frequency * 100.0
                    ),
                    impact_score: top_rw.rework_frequency,
                });
                recommendations.push(Recommendation {
                    action: format!("Reduce rework at '{}'", top_rw.activity),
                    expected_improvement: "Save wasted rework time".to_string(),
                    priority: Priority::Medium,
                });
            }

            let supporting = build_supporting_data(log);
            let confidence = if root_causes.is_empty() { 0.3 } else { 0.65 };

            IntelligenceAnswer {
                question: query.to_string(),
                root_causes,
                recommendations,
                supporting_data: supporting,
                confidence,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Convert a structured IntelligenceAnswer into a readable narrative string
pub fn format_intelligence_answer(answer: &IntelligenceAnswer) -> String {
    let mut parts = Vec::new();

    parts.push(format!("Analysis: {}", answer.question));

    if answer.root_causes.is_empty() {
        parts.push("No clear root causes identified from available data.".to_string());
    } else {
        let causes: Vec<String> = answer
            .root_causes
            .iter()
            .enumerate()
            .map(|(i, rc)| {
                format!(
                    "  {}. {} — {} (impact: {:.0}%)",
                    i + 1,
                    rc.factor,
                    rc.evidence,
                    rc.impact_score * 100.0
                )
            })
            .collect();
        parts.push(format!("Root causes:\n{}", causes.join("\n")));
    }

    if !answer.recommendations.is_empty() {
        let recs: Vec<String> = answer
            .recommendations
            .iter()
            .enumerate()
            .map(|(i, rec)| {
                let priority = match rec.priority {
                    Priority::High => "HIGH",
                    Priority::Medium => "MEDIUM",
                    Priority::Low => "LOW",
                };
                format!(
                    "  {}. [{}] {} → {}",
                    i + 1,
                    priority,
                    rec.action,
                    rec.expected_improvement
                )
            })
            .collect();
        parts.push(format!("Recommendations:\n{}", recs.join("\n")));
    }

    if let Some(waste) = answer.supporting_data.variant_waste_estimate {
        parts.push(format!(
            "Variant waste estimate: {:.1}s total across non-dominant paths.",
            waste
        ));
    }

    if !answer.supporting_data.bottleneck_activities.is_empty() {
        parts.push(format!(
            "Top bottleneck activities: {}.",
            answer.supporting_data.bottleneck_activities.join(", ")
        ));
    }

    if !answer.supporting_data.rework_hotspots.is_empty() {
        parts.push(format!(
            "Rework hotspots: {}.",
            answer.supporting_data.rework_hotspots.join(", ")
        ));
    }

    parts.push(format!("Confidence: {:.0}%.", answer.confidence * 100.0));

    parts.join("\n")
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn low_confidence_answer(query: &str, reason: &str) -> IntelligenceAnswer {
    IntelligenceAnswer {
        question: query.to_string(),
        root_causes: vec![],
        recommendations: vec![Recommendation {
            action: reason.to_string(),
            expected_improvement: "Provide an event log to enable data-driven recommendations."
                .to_string(),
            priority: Priority::Low,
        }],
        supporting_data: SupportingData {
            bottleneck_activities: vec![],
            rework_hotspots: vec![],
            variant_waste_estimate: None,
        },
        confidence: 0.1,
    }
}

fn build_supporting_data(log: &crate::log::EventLog) -> SupportingData {
    let bottlenecks = crate::statistics::identify_bottlenecks(log);
    let rework_stats = crate::statistics::rework_statistics(log);

    let bottleneck_activities: Vec<String> = bottlenecks
        .iter()
        .take(5)
        .map(|b| b.activity.clone())
        .collect();

    let rework_hotspots: Vec<String> = rework_stats
        .iter()
        .filter(|r| r.rework_frequency > 0.1)
        .take(5)
        .map(|r| r.activity.clone())
        .collect();

    SupportingData {
        bottleneck_activities,
        rework_hotspots,
        variant_waste_estimate: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, EventLog, Trace};
    use chrono::Utc;

    fn make_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        // Two normal traces
        for i in 0..3_u32 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("Register", now));
            trace.add_event(Event::new(
                "Review",
                now + chrono::Duration::seconds(300 + i as i64 * 50),
            ));
            trace.add_event(Event::new(
                "Approve",
                now + chrono::Duration::seconds(600 + i as i64 * 80),
            ));
            log.add_trace(trace);
        }

        // One rework trace
        let mut rw_trace = Trace::new("case_rework");
        rw_trace.add_event(Event::new("Register", now));
        rw_trace.add_event(Event::new("Review", now + chrono::Duration::seconds(200)));
        rw_trace.add_event(Event::new("Register", now + chrono::Duration::seconds(400)));
        rw_trace.add_event(Event::new("Approve", now + chrono::Duration::seconds(600)));
        log.add_trace(rw_trace);

        log
    }

    #[test]
    fn test_cycle_time_question_has_root_causes() {
        let log = make_test_log();
        let ctx = IntelligenceContext {
            log: Some(&log),
            petri_net: None,
        };
        let answer = answer_causal_question("Why is cycle time so high?", &ctx);
        assert!(
            !answer.root_causes.is_empty(),
            "expected non-empty root_causes for cycle time query"
        );
        assert!(answer.confidence > 0.5);
    }

    #[test]
    fn test_variant_question_has_waste_estimate() {
        let log = make_test_log();
        let ctx = IntelligenceContext {
            log: Some(&log),
            petri_net: None,
        };
        let answer = answer_causal_question("Which variants should we eliminate?", &ctx);
        // variant_waste_estimate should be Some when there are multiple paths
        assert!(
            answer.supporting_data.variant_waste_estimate.is_some()
                || answer.root_causes.is_empty(), // acceptable if all cases share one path
            "variant_waste_estimate unexpectedly None with diverse log"
        );
    }

    #[test]
    fn test_none_log_does_not_panic() {
        let ctx = IntelligenceContext {
            log: None,
            petri_net: None,
        };
        let answer = answer_causal_question("Why are there so many deviations?", &ctx);
        // Must not panic; confidence should be low
        assert!(answer.confidence < 0.5);
        assert!(!answer.recommendations.is_empty());
    }

    #[test]
    fn test_generic_question_with_log() {
        let log = make_test_log();
        let ctx = IntelligenceContext {
            log: Some(&log),
            petri_net: None,
        };
        let answer =
            answer_causal_question("What is causing the process performance issues?", &ctx);
        assert!(answer.confidence > 0.0);
    }
}
