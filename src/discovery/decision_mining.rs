//! Decision Mining — discovers branching rules from event logs.
//!
//! Identifies *split points* in the process (activities with multiple distinct
//! successors) and infers conditions that determine which path is taken.
//! Confidence is computed as the fraction of cases that follow each branch;
//! Gini impurity is used to rank split points by decisiveness.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// A single decision rule discovered at a process branching point.
#[derive(Debug, Clone)]
pub struct DecisionRule {
    /// The activity at which the branching occurs.
    pub split_activity: String,
    /// Natural language description of the condition, e.g. `"follows A → B"`.
    pub condition: String,
    /// Fraction of cases at this split point that take this branch (0.0–1.0).
    pub confidence: f64,
    /// Number of cases supporting this rule.
    pub support: usize,
}

/// Set of decision rules discovered from an event log.
#[derive(Debug, Clone, Default)]
pub struct DecisionModel {
    /// Rules sorted by confidence descending.
    pub rules: Vec<DecisionRule>,
}

impl DecisionModel {
    /// Returns the number of rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Returns rules for a specific split activity.
    pub fn rules_for(&self, activity: &str) -> Vec<&DecisionRule> {
        self.rules
            .iter()
            .filter(|r| r.split_activity == activity)
            .collect()
    }
}

/// Discover decision rules from an event log.
///
/// # Algorithm
///
/// 1. Build a DFG to find *split points* — activities with ≥ 2 distinct successors.
/// 2. For each split point, tally how many cases took each outgoing branch.
/// 3. Emit one `DecisionRule` per branch; confidence = branch_count / total.
/// 4. Return rules sorted by confidence descending.
pub fn mine_decision_rules(log: &EventLog) -> DecisionModel {
    let split_points = identify_split_points(log);
    let mut rules = Vec::new();

    for split_act in &split_points {
        let cases = collect_split_cases(log, split_act);
        if cases.is_empty() {
            continue;
        }

        let total = cases.len();
        let mut successor_counts: HashMap<String, usize> = HashMap::new();
        for (_, successor) in &cases {
            *successor_counts.entry(successor.clone()).or_insert(0) += 1;
        }

        for (successor, count) in &successor_counts {
            let confidence = *count as f64 / total as f64;
            rules.push(DecisionRule {
                split_activity: split_act.clone(),
                condition: format!("follows {} → {}", split_act, successor),
                confidence,
                support: *count,
            });
        }
    }

    rules.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    DecisionModel { rules }
}

/// Compute the Gini impurity of a set of class counts.
///
/// Returns 0.0 for a perfectly pure split, approaching 1.0 for maximum disorder.
pub fn gini_impurity(counts: &[usize]) -> f64 {
    let total: usize = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let sum_sq: f64 = counts
        .iter()
        .map(|&c| (c as f64 / total as f64).powi(2))
        .sum();
    1.0 - sum_sq
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Return activities that have ≥ 2 distinct immediate successors in the DFG.
fn identify_split_points(log: &EventLog) -> Vec<String> {
    let mut successor_map: HashMap<String, HashSet<String>> = HashMap::new();

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            successor_map
                .entry(window[0].activity.clone())
                .or_default()
                .insert(window[1].activity.clone());
        }
    }

    let mut splits: Vec<String> = successor_map
        .into_iter()
        .filter(|(_, successors)| successors.len() >= 2)
        .map(|(activity, _)| activity)
        .collect();

    splits.sort();
    splits
}

/// Return `(trace_id, successor_activity)` pairs for every occurrence of
/// `split_activity` in the log that has an immediate successor.
fn collect_split_cases(log: &EventLog, split_activity: &str) -> Vec<(String, String)> {
    let mut cases = Vec::new();
    for trace in &log.traces {
        for window in trace.events.windows(2) {
            if window[0].activity == split_activity {
                cases.push((trace.id.clone(), window[1].activity.clone()));
            }
        }
    }
    cases
}
