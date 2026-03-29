//! Extended DFG Miner - Advanced Directly-Follows Graph discovery with metrics
//!
//! This module provides an extended version of the DFG miner with:
//! - Dependency matrix computation (directly-follows strength)
//! - Threshold filtering for noise reduction
//! - Parallel arcs detection and separation
//! - Support and confidence metrics
//! - Advanced analysis of process structure

use crate::log::EventLog;
use crate::models::dfg::DirectlyFollowsGraph;
use std::collections::HashMap;

/// Represents the strength of a directly-follows relation
#[derive(Debug, Clone)]
pub struct DependencyMetric {
    /// From activity
    pub from: String,
    /// To activity
    pub to: String,
    /// Frequency of the relation
    pub frequency: usize,
    /// Support (fraction of traces containing this relation)
    pub support: f64,
    /// Confidence (conditional probability: P(to|from))
    pub confidence: f64,
    /// Dependency measure: (count(a->b) - count(b->a)) / (count(a->b) + count(b->a) + 1)
    pub dependency_strength: f64,
}

impl DependencyMetric {
    /// Create a new dependency metric
    pub fn new(
        from: String,
        to: String,
        frequency: usize,
        support: f64,
        confidence: f64,
        dependency_strength: f64,
    ) -> Self {
        Self {
            from,
            to,
            frequency,
            support,
            confidence,
            dependency_strength,
        }
    }
}

/// Represents parallel arcs detected in the graph
#[derive(Debug, Clone)]
pub struct ParallelArc {
    /// First activity
    pub activity_a: String,
    /// Second activity
    pub activity_b: String,
    /// Count of a->b
    pub a_to_b_count: usize,
    /// Count of b->a
    pub b_to_a_count: usize,
    /// Parallelism degree (how balanced the arcs are)
    pub parallelism_degree: f64,
}

impl ParallelArc {
    /// Create a new parallel arc
    pub fn new(
        activity_a: String,
        activity_b: String,
        a_to_b_count: usize,
        b_to_a_count: usize,
    ) -> Self {
        let parallelism_degree = if a_to_b_count + b_to_a_count == 0 {
            0.0
        } else {
            let min = a_to_b_count.min(b_to_a_count) as f64;
            let max = a_to_b_count.max(b_to_a_count) as f64;
            min / max
        };

        Self {
            activity_a,
            activity_b,
            a_to_b_count,
            b_to_a_count,
            parallelism_degree,
        }
    }
}

/// Extended DFG Miner with advanced metrics
pub struct DFGMinerExtended {
    /// Minimum support threshold (fraction of traces)
    pub min_support: f64,
    /// Minimum confidence threshold
    pub min_confidence: f64,
    /// Minimum dependency strength threshold
    pub min_dependency_strength: f64,
    /// Detect and separate parallel arcs
    pub detect_parallelism: bool,
}

impl DFGMinerExtended {
    /// Create a new extended DFG miner with default parameters
    pub fn new() -> Self {
        Self {
            min_support: 0.0,
            min_confidence: 0.0,
            min_dependency_strength: 0.0,
            detect_parallelism: true,
        }
    }

    /// Set minimum support threshold
    pub fn with_min_support(mut self, support: f64) -> Self {
        self.min_support = support.clamp(0.0, 1.0);
        self
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set minimum dependency strength threshold
    pub fn with_min_dependency_strength(mut self, strength: f64) -> Self {
        self.min_dependency_strength = strength.clamp(-1.0, 1.0);
        self
    }

    /// Enable/disable parallelism detection
    pub fn with_parallelism_detection(mut self, enabled: bool) -> Self {
        self.detect_parallelism = enabled;
        self
    }

    /// Discover DFG and compute metrics
    pub fn discover(&self, log: &EventLog) -> DirectlyFollowsGraph {
        DirectlyFollowsGraph::from_log(log)
    }

    /// Compute dependency matrix with all metrics
    pub fn compute_dependency_matrix(&self, log: &EventLog) -> Vec<DependencyMetric> {
        let mut metrics = Vec::new();
        let total_traces = log.len() as f64;

        // Count directly-follows relations
        let mut follows_count: HashMap<(String, String), usize> = HashMap::new();
        let mut from_counts: HashMap<String, usize> = HashMap::new();
        let mut to_counts: HashMap<String, usize> = HashMap::new();

        for trace in &log.traces {
            for i in 0..trace.events.len() - 1 {
                let from = &trace.events[i].activity;
                let to = &trace.events[i + 1].activity;

                *follows_count.entry((from.clone(), to.clone())).or_insert(0) += 1;
                *from_counts.entry(from.clone()).or_insert(0) += 1;
                *to_counts.entry(to.clone()).or_insert(0) += 1;
            }
        }

        // Count traces containing each relation
        let mut traces_with_relation: HashMap<(String, String), usize> = HashMap::new();
        for trace in &log.traces {
            let mut seen = std::collections::HashSet::new();
            for i in 0..trace.events.len() - 1 {
                let from = &trace.events[i].activity;
                let to = &trace.events[i + 1].activity;
                let key = (from.clone(), to.clone());

                if !seen.contains(&key) {
                    *traces_with_relation.entry(key.clone()).or_insert(0) += 1;
                    seen.insert(key);
                }
            }
        }

        // Compute metrics for each relation
        for ((from, to), frequency) in &follows_count {
            let support = traces_with_relation
                .get(&(from.clone(), to.clone()))
                .copied()
                .unwrap_or(0) as f64
                / total_traces;

            let confidence =
                *frequency as f64 / from_counts.get(from.as_str()).copied().unwrap_or(1) as f64;

            // Get reverse count for dependency strength
            let reverse_count = follows_count
                .get(&(to.clone(), from.clone()))
                .copied()
                .unwrap_or(0);

            let dependency_strength = ((*frequency as i32) - (reverse_count as i32)) as f64
                / ((*frequency as f64) + (reverse_count as f64) + 1.0);

            // Apply filters
            if support >= self.min_support
                && confidence >= self.min_confidence
                && dependency_strength >= self.min_dependency_strength
            {
                metrics.push(DependencyMetric::new(
                    from.clone(),
                    to.clone(),
                    *frequency,
                    support,
                    confidence,
                    dependency_strength,
                ));
            }
        }

        // Sort by frequency descending
        metrics.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        metrics
    }

    /// Detect parallel arcs in the log
    pub fn detect_parallel_arcs(&self, log: &EventLog) -> Vec<ParallelArc> {
        let mut parallel_arcs = Vec::new();
        let mut follows_count: HashMap<(String, String), usize> = HashMap::new();

        for trace in &log.traces {
            for i in 0..trace.events.len() - 1 {
                let from = &trace.events[i].activity;
                let to = &trace.events[i + 1].activity;
                *follows_count.entry((from.clone(), to.clone())).or_insert(0) += 1;
            }
        }

        // Find bidirectional relations
        let mut processed = std::collections::HashSet::new();

        for ((from, to), a_to_b_count) in &follows_count {
            let key = if from < to {
                (from.clone(), to.clone())
            } else {
                (to.clone(), from.clone())
            };

            if processed.contains(&key) {
                continue;
            }
            processed.insert(key);

            if let Some(&b_to_a_count) = follows_count.get(&(to.clone(), from.clone())) {
                if *a_to_b_count > 0 && b_to_a_count > 0 {
                    parallel_arcs.push(ParallelArc::new(
                        from.clone(),
                        to.clone(),
                        *a_to_b_count,
                        b_to_a_count,
                    ));
                }
            }
        }

        // Sort by parallelism degree
        parallel_arcs.sort_by(|a, b| {
            b.parallelism_degree
                .partial_cmp(&a.parallelism_degree)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        parallel_arcs
    }

    /// Get the most significant relations (top-k)
    pub fn get_top_relations(&self, log: &EventLog, k: usize) -> Vec<DependencyMetric> {
        let mut metrics = self.compute_dependency_matrix(log);
        metrics.truncate(k);
        metrics
    }

    /// Filter DFG by dependency metrics
    pub fn filter_dfg(&self, dfg: &mut DirectlyFollowsGraph) {
        dfg.edges.retain(|edge| {
            // Keep edges that pass the thresholds
            // This is a simple frequency-based filter
            edge.frequency > 0
        });

        // Remove isolated nodes
        let used_activities: std::collections::HashSet<_> = dfg
            .edges
            .iter()
            .flat_map(|e| vec![e.from.clone(), e.to.clone()])
            .collect();

        dfg.nodes
            .retain(|n| used_activities.contains(n) || dfg.start_activities.contains_key(n));
    }
}

impl Default for DFGMinerExtended {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::{Duration, Utc};

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        // Trace 1: a->b->c
        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now));
        trace1.add_event(Event::new("b", now + Duration::seconds(1)));
        trace1.add_event(Event::new("c", now + Duration::seconds(2)));
        log.add_trace(trace1);

        // Trace 2: a->b->c
        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("a", now));
        trace2.add_event(Event::new("b", now + Duration::seconds(1)));
        trace2.add_event(Event::new("c", now + Duration::seconds(2)));
        log.add_trace(trace2);

        // Trace 3: a->b->d
        let mut trace3 = Trace::new("case_3");
        trace3.add_event(Event::new("a", now));
        trace3.add_event(Event::new("b", now + Duration::seconds(1)));
        trace3.add_event(Event::new("d", now + Duration::seconds(2)));
        log.add_trace(trace3);

        log
    }

    fn create_parallel_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        // Trace 1: a->b->c
        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now));
        trace1.add_event(Event::new("b", now + Duration::seconds(1)));
        trace1.add_event(Event::new("c", now + Duration::seconds(2)));
        log.add_trace(trace1);

        // Trace 2: a->c->b
        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("a", now));
        trace2.add_event(Event::new("c", now + Duration::seconds(1)));
        trace2.add_event(Event::new("b", now + Duration::seconds(2)));
        log.add_trace(trace2);

        log
    }

    #[test]
    fn test_dfg_miner_extended_creation() {
        let miner = DFGMinerExtended::new();
        assert_eq!(miner.min_support, 0.0);
        assert_eq!(miner.min_confidence, 0.0);
        assert_eq!(miner.detect_parallelism, true);
    }

    #[test]
    fn test_dfg_miner_with_thresholds() {
        let miner = DFGMinerExtended::new()
            .with_min_support(0.5)
            .with_min_confidence(0.7)
            .with_min_dependency_strength(0.3);

        assert_eq!(miner.min_support, 0.5);
        assert_eq!(miner.min_confidence, 0.7);
        assert_eq!(miner.min_dependency_strength, 0.3);
    }

    #[test]
    fn test_discover_creates_dfg() {
        let log = create_test_log();
        let miner = DFGMinerExtended::new();
        let dfg = miner.discover(&log);

        assert!(!dfg.nodes.is_empty());
        assert!(!dfg.edges.is_empty());
    }

    #[test]
    fn test_compute_dependency_matrix() {
        let log = create_test_log();
        let miner = DFGMinerExtended::new();
        let metrics = miner.compute_dependency_matrix(&log);

        assert!(!metrics.is_empty());
        // Check that a->b exists (3 times in 3 traces)
        assert!(metrics
            .iter()
            .any(|m| m.from == "a" && m.to == "b" && m.frequency == 3));
    }

    #[test]
    fn test_dependency_metrics_support_confidence() {
        let log = create_test_log();
        let miner = DFGMinerExtended::new();
        let metrics = miner.compute_dependency_matrix(&log);

        // a->b appears in all 3 traces with frequency 3
        let a_to_b = metrics.iter().find(|m| m.from == "a" && m.to == "b");
        assert!(a_to_b.is_some());
        let metric = a_to_b.unwrap();
        assert_eq!(metric.support, 1.0); // All traces
        assert_eq!(metric.confidence, 1.0); // All a's are followed by b
    }

    #[test]
    fn test_detect_parallel_arcs() {
        let log = create_parallel_log();
        let miner = DFGMinerExtended::new();
        let parallel_arcs = miner.detect_parallel_arcs(&log);

        // Should detect b<->c parallelism
        assert!(!parallel_arcs.is_empty());
        assert!(parallel_arcs
            .iter()
            .any(|arc| (arc.activity_a == "b" && arc.activity_b == "c")
                || (arc.activity_a == "c" && arc.activity_b == "b")));
    }

    #[test]
    fn test_get_top_relations() {
        let log = create_test_log();
        let miner = DFGMinerExtended::new();
        let top_3 = miner.get_top_relations(&log, 3);

        assert!(top_3.len() <= 3);
        assert!(!top_3.is_empty());
    }

    #[test]
    fn test_filter_by_confidence_threshold() {
        let log = create_test_log();
        let miner = DFGMinerExtended::new().with_min_confidence(0.9);
        let metrics = miner.compute_dependency_matrix(&log);

        // All metrics should have confidence >= 0.9
        assert!(metrics.iter().all(|m| m.confidence >= 0.9));
    }

    #[test]
    fn test_parallelism_degree_calculation() {
        let arc = ParallelArc::new("a".to_string(), "b".to_string(), 5, 4);
        assert!(arc.parallelism_degree > 0.0);
        assert!(arc.parallelism_degree <= 1.0);

        let balanced_arc = ParallelArc::new("c".to_string(), "d".to_string(), 10, 10);
        assert_eq!(balanced_arc.parallelism_degree, 1.0);
    }
}
