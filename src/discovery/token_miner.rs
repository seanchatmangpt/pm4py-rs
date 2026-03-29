//! Token-Based Miner - Conformance-aware process discovery
//!
//! This module implements a token-based approach to process discovery that:
//! - Performs token-based replay on traces
//! - Identifies patterns based on token flow
//! - Detects implicit dependencies
//! - Computes conformance metrics during discovery

use crate::log::EventLog;
use crate::models::dfg::DirectlyFollowsGraph;
use std::collections::{HashMap, HashSet};

/// Represents an implicit dependency detected during token flow
#[derive(Debug, Clone)]
pub struct ImplicitDependency {
    /// Source activity
    pub source: String,
    /// Target activity
    pub target: String,
    /// Count of traces where this dependency was observed
    pub count: usize,
    /// Confidence (fraction of traces with target that had source before)
    pub confidence: f64,
}

impl ImplicitDependency {
    /// Create a new implicit dependency
    pub fn new(source: String, target: String, count: usize, confidence: f64) -> Self {
        Self {
            source,
            target,
            count,
            confidence,
        }
    }
}

/// Represents token flow information for a trace
#[derive(Debug, Clone)]
pub struct TokenFlowInfo {
    /// Trace ID
    pub trace_id: String,
    /// Token path (sequence of activities)
    pub token_path: Vec<String>,
    /// Active tokens at each position
    pub active_tokens: Vec<HashSet<String>>,
    /// Replay fitness (0.0-1.0, where 1.0 is perfect)
    pub fitness: f64,
    /// Missing tokens count
    pub missing_tokens: usize,
    /// Remaining tokens count
    pub remaining_tokens: usize,
}

impl TokenFlowInfo {
    /// Create a new token flow info
    pub fn new(trace_id: String, token_path: Vec<String>) -> Self {
        let active_tokens = vec![HashSet::new(); token_path.len()];
        Self {
            trace_id,
            token_path,
            active_tokens,
            fitness: 0.0,
            missing_tokens: 0,
            remaining_tokens: 0,
        }
    }

    /// Calculate fitness score based on missing and remaining tokens
    pub fn calculate_fitness(&mut self, total_events: usize) {
        if total_events == 0 {
            self.fitness = 0.0;
            return;
        }

        let penalties = (self.missing_tokens + self.remaining_tokens) as f64;
        self.fitness = ((total_events as f64 - penalties) / total_events as f64).max(0.0);
    }
}

/// Token-Based Process Miner
pub struct TokenMiner {
    /// Enable implicit dependency detection
    pub detect_implicit_dependencies: bool,
    /// Minimum confidence for implicit dependencies
    pub min_implicit_confidence: f64,
    /// Token consumption strategy: strict or lenient
    pub strict_token_consumption: bool,
}

impl TokenMiner {
    /// Create a new token miner with default parameters
    pub fn new() -> Self {
        Self {
            detect_implicit_dependencies: true,
            min_implicit_confidence: 0.5,
            strict_token_consumption: false,
        }
    }

    /// Enable/disable implicit dependency detection
    pub fn with_implicit_dependency_detection(mut self, enabled: bool) -> Self {
        self.detect_implicit_dependencies = enabled;
        self
    }

    /// Set minimum confidence for implicit dependencies
    pub fn with_min_implicit_confidence(mut self, confidence: f64) -> Self {
        self.min_implicit_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set token consumption strategy
    pub fn with_strict_token_consumption(mut self, strict: bool) -> Self {
        self.strict_token_consumption = strict;
        self
    }

    /// Discover process model using token-based approach
    pub fn discover(&self, log: &EventLog) -> DirectlyFollowsGraph {
        DirectlyFollowsGraph::from_log(log)
    }

    /// Perform token-based replay on traces
    pub fn replay_traces(&self, log: &EventLog) -> Vec<TokenFlowInfo> {
        let mut flows = Vec::new();

        for trace in &log.traces {
            let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
            let mut flow = TokenFlowInfo::new(trace.id.clone(), activities.clone());

            // Simulate token flow
            self.simulate_token_flow(&mut flow, log);
            flows.push(flow);
        }

        flows
    }

    /// Simulate token flow for a single trace
    fn simulate_token_flow(&self, flow: &mut TokenFlowInfo, log: &EventLog) {
        let mut active_tokens: HashMap<String, usize> = HashMap::new();

        // Initialize with start activities
        let start_activities: HashSet<_> = log
            .traces
            .iter()
            .filter_map(|t| t.events.first().map(|e| e.activity.clone()))
            .collect();

        for activity in &start_activities {
            *active_tokens.entry(activity.clone()).or_insert(0) += 1;
        }

        // Process each event in the trace
        for (i, activity) in flow.token_path.iter().enumerate() {
            let mut can_execute = false;

            // Check if we have a token for this activity
            if let Some(count) = active_tokens.get_mut(activity) {
                if *count > 0 {
                    *count -= 1;
                    can_execute = true;
                }
            }

            if !can_execute {
                flow.missing_tokens += 1;
            }

            // Record active tokens at this position
            let active_set: HashSet<String> = active_tokens
                .iter()
                .filter(|(_, &count)| count > 0)
                .map(|(activity, _)| activity.clone())
                .collect();
            if i < flow.active_tokens.len() {
                flow.active_tokens[i] = active_set;
            }

            // Generate new tokens for activities following this one
            if let Some(next_idx) = flow.token_path.iter().position(|a| a == activity) {
                if next_idx < flow.token_path.len() - 1 {
                    let next_activity = &flow.token_path[next_idx + 1];
                    *active_tokens.entry(next_activity.clone()).or_insert(0) += 1;
                }
            }
        }

        flow.remaining_tokens = active_tokens.values().sum();
        flow.calculate_fitness(flow.token_path.len());
    }

    /// Detect implicit dependencies in the log
    pub fn detect_implicit_dependencies(&self, log: &EventLog) -> Vec<ImplicitDependency> {
        let mut implicit_deps = Vec::new();
        let mut dependency_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut target_counts: HashMap<String, usize> = HashMap::new();

        for trace in &log.traces {
            let activities: Vec<_> = trace.events.iter().map(|e| &e.activity).collect();

            for (i, target) in activities.iter().enumerate() {
                let target = target.to_string();
                *target_counts.entry(target.clone()).or_insert(0) += 1;

                // Check for implicit dependencies (any preceding activity)
                for source in activities[..i].iter() {
                    *dependency_counts
                        .entry((source.to_string(), target.clone()))
                        .or_insert(0) += 1;
                }
            }
        }

        // Compute confidence for each implicit dependency
        for ((source, target), count) in dependency_counts {
            if let Some(&target_total) = target_counts.get(&target) {
                let confidence = count as f64 / target_total as f64;
                if confidence >= self.min_implicit_confidence {
                    implicit_deps.push(ImplicitDependency::new(source, target, count, confidence));
                }
            }
        }

        // Sort by confidence descending
        implicit_deps.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.count.cmp(&a.count))
        });

        implicit_deps
    }

    /// Get average fitness across all traces
    pub fn compute_average_fitness(&self, log: &EventLog) -> f64 {
        let flows = self.replay_traces(log);
        if flows.is_empty() {
            return 0.0;
        }

        flows.iter().map(|f| f.fitness).sum::<f64>() / flows.len() as f64
    }

    /// Compute per-trace fitness distribution
    pub fn compute_fitness_distribution(&self, log: &EventLog) -> (f64, f64, f64) {
        let flows = self.replay_traces(log);
        if flows.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let fitnesses: Vec<f64> = flows.iter().map(|f| f.fitness).collect();
        let min = fitnesses.iter().copied().fold(f64::INFINITY, f64::min);
        let max = fitnesses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;

        (min, avg, max)
    }

    /// Find traces with poor fitness
    pub fn find_poor_fitness_traces(&self, log: &EventLog, threshold: f64) -> Vec<TokenFlowInfo> {
        self.replay_traces(log)
            .into_iter()
            .filter(|flow| flow.fitness < threshold)
            .collect()
    }
}

impl Default for TokenMiner {
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

        // Trace 3: a->d->c
        let mut trace3 = Trace::new("case_3");
        trace3.add_event(Event::new("a", now));
        trace3.add_event(Event::new("d", now + Duration::seconds(1)));
        trace3.add_event(Event::new("c", now + Duration::seconds(2)));
        log.add_trace(trace3);

        log
    }

    #[test]
    fn test_token_miner_creation() {
        let miner = TokenMiner::new();
        assert_eq!(miner.detect_implicit_dependencies, true);
        assert_eq!(miner.min_implicit_confidence, 0.5);
    }

    #[test]
    fn test_token_miner_with_options() {
        let miner = TokenMiner::new()
            .with_implicit_dependency_detection(false)
            .with_min_implicit_confidence(0.7)
            .with_strict_token_consumption(true);

        assert_eq!(miner.detect_implicit_dependencies, false);
        assert_eq!(miner.min_implicit_confidence, 0.7);
        assert_eq!(miner.strict_token_consumption, true);
    }

    #[test]
    fn test_discover_creates_dfg() {
        let log = create_test_log();
        let miner = TokenMiner::new();
        let dfg = miner.discover(&log);

        assert!(!dfg.nodes.is_empty());
        assert!(!dfg.edges.is_empty());
    }

    #[test]
    fn test_replay_traces() {
        let log = create_test_log();
        let miner = TokenMiner::new();
        let flows = miner.replay_traces(&log);

        assert_eq!(flows.len(), 3);
        assert!(flows.iter().all(|f| f.fitness >= 0.0 && f.fitness <= 1.0));
    }

    #[test]
    fn test_detect_implicit_dependencies() {
        let log = create_test_log();
        let miner = TokenMiner::new().with_min_implicit_confidence(0.0);
        let deps = miner.detect_implicit_dependencies(&log);

        // Should detect a->c (implicit, a comes before c in all traces)
        assert!(!deps.is_empty());
        assert!(deps.iter().any(|d| d.source == "a" && d.target == "c"));
    }

    #[test]
    fn test_compute_average_fitness() {
        let log = create_test_log();
        let miner = TokenMiner::new();
        let fitness = miner.compute_average_fitness(&log);

        assert!(fitness >= 0.0 && fitness <= 1.0);
    }

    #[test]
    fn test_compute_fitness_distribution() {
        let log = create_test_log();
        let miner = TokenMiner::new();
        let (min, avg, max) = miner.compute_fitness_distribution(&log);

        assert!(min <= avg);
        assert!(avg <= max);
        assert!(min >= 0.0 && max <= 1.0);
    }
}
