//! Domain Classifier for Process Intelligence Queries
//!
//! Classifies natural language queries into two domains:
//! - Domain 1 (Description): describe/explain/what is
//! - Domain 2 (Intelligence): why/cause/improve/recommend

use serde::{Deserialize, Serialize};

/// Query domain classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryDomain {
    /// Domain 1: Describe, explain, or summarize a process
    Description,
    /// Domain 2: Diagnose causes, recommend improvements
    Intelligence,
}

/// Classification result with confidence and matched signals
#[derive(Debug, Clone)]
pub struct DomainClassification {
    pub domain: QueryDomain,
    pub confidence: f64,
    pub matched_signals: Vec<String>,
}

/// Classify a query string into a domain
///
/// Domain 2 signals are weighted 1.2x because causal/improvement intent
/// is more distinctive and should win when mixed with description words.
pub fn classify_domain(query: &str) -> DomainClassification {
    let q = query.to_lowercase();

    // Domain 2 signals (weight 1.2 per match)
    let d2_signals: &[&str] = &[
        "why",
        "cause",
        "root cause",
        "improve",
        "recommend",
        "eliminate",
        "reduce",
        "fix",
        "optimize",
        "should we",
        "because",
        "due to",
        "drive",
        "impact",
        "lower",
    ];

    // Domain 1 signals (weight 1.0 per match)
    let d1_signals: &[&str] = &[
        "what is",
        "describe",
        "show",
        "explain",
        "list",
        "display",
        "tell me",
        "summarize",
        "what are",
    ];

    let matched_d2: Vec<String> = d2_signals
        .iter()
        .filter(|s| q.contains(**s))
        .map(|s| s.to_string())
        .collect();

    let matched_d1: Vec<String> = d1_signals
        .iter()
        .filter(|s| q.contains(**s))
        .map(|s| s.to_string())
        .collect();

    let d2_score = matched_d2.len() as f64 * 1.2;
    let d1_score = matched_d1.len() as f64;
    let total = d1_score + d2_score;

    if total == 0.0 {
        // No signals found — default to Description at 50% confidence
        return DomainClassification {
            domain: QueryDomain::Description,
            confidence: 0.5,
            matched_signals: vec![],
        };
    }

    let (domain, confidence, signals) = if d2_score > d1_score {
        (QueryDomain::Intelligence, d2_score / total, matched_d2)
    } else {
        (QueryDomain::Description, d1_score / total, matched_d1)
    };

    // Fall back to Description when confidence is below threshold
    let final_domain = if confidence < 0.5 {
        QueryDomain::Description
    } else {
        domain
    };

    DomainClassification {
        domain: final_domain,
        confidence,
        matched_signals: signals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_intelligence_cycle_time() {
        let result = classify_domain("Why is cycle time so high?");
        assert_eq!(result.domain, QueryDomain::Intelligence);
        assert!(
            result.confidence > 0.6,
            "expected confidence > 0.6, got {}",
            result.confidence
        );
    }

    #[test]
    fn test_classify_description_process_model() {
        let result = classify_domain("What is the process model?");
        assert_eq!(result.domain, QueryDomain::Description);
    }

    #[test]
    fn test_classify_intelligence_recommend() {
        let result = classify_domain("What should we eliminate to improve throughput?");
        assert_eq!(result.domain, QueryDomain::Intelligence);
    }

    #[test]
    fn test_classify_no_signals_defaults_description() {
        let result = classify_domain("process flow overview");
        assert_eq!(result.domain, QueryDomain::Description);
        assert_eq!(result.confidence, 0.5);
    }
}
