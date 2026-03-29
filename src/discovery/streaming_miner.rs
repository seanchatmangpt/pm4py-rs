//! Streaming Miner - Incremental process model discovery
//!
//! This module implements streaming-based process discovery that:
//! - Performs incremental discovery as events arrive
//! - Supports sliding window analysis
//! - Detects concept drift in the process
//! - Adapts the model over time

use crate::log::EventLog;
use crate::models::dfg::DirectlyFollowsGraph;
use std::collections::{HashMap, VecDeque};

/// Represents a sliding window of events
#[derive(Debug, Clone)]
pub struct StreamingWindow {
    /// Maximum window size
    pub max_size: usize,
    /// Current events in the window
    pub events: VecDeque<(String, String)>, // (from_activity, to_activity)
    /// Activities seen in current window
    pub activities: std::collections::HashSet<String>,
}

impl StreamingWindow {
    /// Create a new streaming window with specified size
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            events: VecDeque::new(),
            activities: std::collections::HashSet::new(),
        }
    }

    /// Add an event to the window
    pub fn add_event(&mut self, from: String, to: String) {
        if self.events.len() >= self.max_size {
            if let Some((old_from, old_to)) = self.events.pop_front() {
                // Check if old activities are still in use
                let still_used = self
                    .events
                    .iter()
                    .any(|(f, t)| f == &old_from || t == &old_from || f == &old_to || t == &old_to);
                if !still_used {
                    self.activities.remove(&old_from);
                    self.activities.remove(&old_to);
                }
            }
        }

        self.activities.insert(from.clone());
        self.activities.insert(to.clone());
        self.events.push_back((from, to));
    }

    /// Get current window size
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if window is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get activity frequency in current window
    pub fn activity_frequency(&self) -> HashMap<String, usize> {
        let mut freq = HashMap::new();

        for (from, to) in &self.events {
            *freq.entry(from.clone()).or_insert(0) += 1;
            *freq.entry(to.clone()).or_insert(0) += 1;
        }

        freq
    }

    /// Get directly-follows count in current window
    pub fn directly_follows_count(&self) -> HashMap<(String, String), usize> {
        let mut count = HashMap::new();

        for (from, to) in &self.events {
            *count.entry((from.clone(), to.clone())).or_insert(0) += 1;
        }

        count
    }
}

/// Represents detected concept drift
#[derive(Debug, Clone)]
pub struct ConceptDrift {
    /// When the drift was detected (window index)
    pub window_index: usize,
    /// Activities that disappeared
    pub disappeared_activities: Vec<String>,
    /// New activities that appeared
    pub new_activities: Vec<String>,
    /// Changed directly-follows relations
    pub changed_relations: Vec<(String, String)>,
    /// Drift severity (0.0-1.0)
    pub severity: f64,
}

impl ConceptDrift {
    /// Create a new concept drift
    pub fn new(
        window_index: usize,
        disappeared_activities: Vec<String>,
        new_activities: Vec<String>,
        changed_relations: Vec<(String, String)>,
        severity: f64,
    ) -> Self {
        Self {
            window_index,
            disappeared_activities,
            new_activities,
            changed_relations,
            severity: severity.clamp(0.0, 1.0),
        }
    }
}

/// Streaming Process Miner
pub struct StreamingMiner {
    /// Size of sliding window
    pub window_size: usize,
    /// Step size for sliding window
    pub step_size: usize,
    /// Detect concept drift
    pub detect_drift: bool,
    /// Minimum change threshold for drift detection
    pub drift_threshold: f64,
}

impl StreamingMiner {
    /// Create a new streaming miner with default parameters
    pub fn new() -> Self {
        Self {
            window_size: 100,
            step_size: 10,
            detect_drift: true,
            drift_threshold: 0.3,
        }
    }

    /// Set sliding window size
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size.max(1);
        self
    }

    /// Set step size for sliding window
    pub fn with_step_size(mut self, size: usize) -> Self {
        self.step_size = size.max(1);
        self
    }

    /// Enable/disable drift detection
    pub fn with_drift_detection(mut self, enabled: bool) -> Self {
        self.detect_drift = enabled;
        self
    }

    /// Set drift detection threshold
    pub fn with_drift_threshold(mut self, threshold: f64) -> Self {
        self.drift_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Discover process model incrementally from log
    pub fn discover(&self, log: &EventLog) -> DirectlyFollowsGraph {
        DirectlyFollowsGraph::from_log(log)
    }

    /// Process log with sliding window and return models for each window
    pub fn discover_windowed(&self, log: &EventLog) -> Vec<DirectlyFollowsGraph> {
        let mut models = Vec::new();
        let mut window = StreamingWindow::new(self.window_size);

        // Extract all edges from log
        let mut all_edges = Vec::new();
        for trace in &log.traces {
            for i in 0..trace.events.len() - 1 {
                let from = &trace.events[i].activity;
                let to = &trace.events[i + 1].activity;
                all_edges.push((from.clone(), to.clone()));
            }
        }

        // Process edges with sliding window
        for (step_count, (from, to)) in all_edges.into_iter().enumerate() {
            window.add_event(from.clone(), to.clone());

            if window.len() >= self.window_size && step_count % self.step_size == 0 {
                models.push(self.window_to_dfg(&window));
            }
        }

        // Add final model
        if !window.is_empty() {
            models.push(self.window_to_dfg(&window));
        }

        models
    }

    /// Convert a window to a DFG model
    fn window_to_dfg(&self, window: &StreamingWindow) -> DirectlyFollowsGraph {
        let mut dfg = DirectlyFollowsGraph::new();

        // Add all activities from window
        for activity in &window.activities {
            if !dfg.nodes.contains(activity) {
                dfg.nodes.push(activity.clone());
            }
        }

        // Add edges with frequencies from window
        let follows = window.directly_follows_count();
        for ((from, to), frequency) in follows {
            dfg.edges.push(crate::models::dfg::DFGEdge {
                from,
                to,
                frequency,
            });
        }

        dfg.nodes.sort();
        dfg
    }

    /// Detect concept drift by comparing consecutive windows
    pub fn detect_concept_drift(&self, log: &EventLog) -> Vec<ConceptDrift> {
        let models = self.discover_windowed(log);
        let mut drifts = Vec::new();

        if models.len() < 2 {
            return drifts;
        }

        for i in 1..models.len() {
            let prev = &models[i - 1];
            let curr = &models[i];

            // Find disappeared and new activities
            let prev_activities: std::collections::HashSet<_> =
                prev.nodes.iter().cloned().collect();
            let curr_activities: std::collections::HashSet<_> =
                curr.nodes.iter().cloned().collect();

            let disappeared: Vec<_> = prev_activities
                .difference(&curr_activities)
                .cloned()
                .collect();

            let new: Vec<_> = curr_activities
                .difference(&prev_activities)
                .cloned()
                .collect();

            // Find changed relations
            let prev_relations: std::collections::HashSet<_> = prev
                .edges
                .iter()
                .map(|e| (e.from.clone(), e.to.clone()))
                .collect();

            let curr_relations: std::collections::HashSet<_> = curr
                .edges
                .iter()
                .map(|e| (e.from.clone(), e.to.clone()))
                .collect();

            let changed: Vec<_> = prev_relations
                .symmetric_difference(&curr_relations)
                .cloned()
                .collect();

            // Calculate severity
            let total_changes = disappeared.len() + new.len() + changed.len();
            let total_elements = prev_activities.len().max(1);
            let severity = total_changes as f64 / total_elements as f64;

            if severity >= self.drift_threshold {
                drifts.push(ConceptDrift::new(i, disappeared, new, changed, severity));
            }
        }

        drifts
    }

    /// Get stability metrics for the process (how much it changes over time)
    pub fn compute_stability_metrics(&self, log: &EventLog) -> (f64, f64) {
        let models = self.discover_windowed(log);
        if models.len() < 2 {
            return (0.0, 0.0);
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for i in 1..models.len() {
            let similarity = self.compute_model_similarity(&models[i - 1], &models[i]);
            total_similarity += similarity;
            count += 1;
        }

        let avg_similarity = if count > 0 {
            total_similarity / count as f64
        } else {
            0.0
        };

        let stability = avg_similarity; // High similarity = stable process
        let drift_rate = 1.0 - stability; // Complement for drift rate

        (stability, drift_rate)
    }

    /// Compute Jaccard similarity between two models
    fn compute_model_similarity(
        &self,
        model1: &DirectlyFollowsGraph,
        model2: &DirectlyFollowsGraph,
    ) -> f64 {
        let set1: std::collections::HashSet<_> =
            model1.edges.iter().map(|e| (&e.from, &e.to)).collect();

        let set2: std::collections::HashSet<_> =
            model2.edges.iter().map(|e| (&e.from, &e.to)).collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Process an incremental event and update model
    pub fn process_event(
        &self,
        current_model: &mut DirectlyFollowsGraph,
        from_activity: &str,
        to_activity: &str,
    ) {
        // Add activities if not present
        if !current_model.nodes.contains(&from_activity.to_string()) {
            current_model.nodes.push(from_activity.to_string());
        }
        if !current_model.nodes.contains(&to_activity.to_string()) {
            current_model.nodes.push(to_activity.to_string());
        }

        // Update or add edge
        if let Some(edge) = current_model
            .edges
            .iter_mut()
            .find(|e| e.from == from_activity && e.to == to_activity)
        {
            edge.frequency += 1;
        } else {
            current_model.edges.push(crate::models::dfg::DFGEdge {
                from: from_activity.to_string(),
                to: to_activity.to_string(),
                frequency: 1,
            });
        }

        current_model.nodes.sort();
    }
}

impl Default for StreamingMiner {
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

        // First batch: a->b->c (stable)
        for i in 0..5 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("a", now + Duration::seconds(i as i64 * 10)));
            trace.add_event(Event::new("b", now + Duration::seconds(i as i64 * 10 + 1)));
            trace.add_event(Event::new("c", now + Duration::seconds(i as i64 * 10 + 2)));
            log.add_trace(trace);
        }

        // Second batch: a->d->c (drift)
        for i in 5..10 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("a", now + Duration::seconds(i as i64 * 10)));
            trace.add_event(Event::new("d", now + Duration::seconds(i as i64 * 10 + 1)));
            trace.add_event(Event::new("c", now + Duration::seconds(i as i64 * 10 + 2)));
            log.add_trace(trace);
        }

        log
    }

    #[test]
    fn test_streaming_window_creation() {
        let window = StreamingWindow::new(10);
        assert_eq!(window.max_size, 10);
        assert_eq!(window.len(), 0);
        assert!(window.is_empty());
    }

    #[test]
    fn test_streaming_window_add_events() {
        let mut window = StreamingWindow::new(5);
        window.add_event("a".to_string(), "b".to_string());
        window.add_event("b".to_string(), "c".to_string());

        assert_eq!(window.len(), 2);
        assert!(!window.is_empty());
        assert!(window.activities.contains("a"));
        assert!(window.activities.contains("b"));
        assert!(window.activities.contains("c"));
    }

    #[test]
    fn test_streaming_window_overflow() {
        let mut window = StreamingWindow::new(2);
        window.add_event("a".to_string(), "b".to_string());
        window.add_event("b".to_string(), "c".to_string());
        window.add_event("c".to_string(), "d".to_string());

        assert_eq!(window.len(), 2); // Should not exceed max_size
    }

    #[test]
    fn test_streaming_miner_creation() {
        let miner = StreamingMiner::new();
        assert_eq!(miner.window_size, 100);
        assert_eq!(miner.step_size, 10);
        assert_eq!(miner.detect_drift, true);
    }

    #[test]
    fn test_streaming_miner_with_options() {
        let miner = StreamingMiner::new()
            .with_window_size(50)
            .with_step_size(5)
            .with_drift_detection(false)
            .with_drift_threshold(0.4);

        assert_eq!(miner.window_size, 50);
        assert_eq!(miner.step_size, 5);
        assert_eq!(miner.detect_drift, false);
        assert_eq!(miner.drift_threshold, 0.4);
    }

    #[test]
    fn test_discover_windowed() {
        let log = create_test_log();
        let miner = StreamingMiner::new().with_window_size(10).with_step_size(5);
        let models = miner.discover_windowed(&log);

        assert!(!models.is_empty());
        assert!(models.iter().all(|m| !m.nodes.is_empty()));
    }

    #[test]
    fn test_detect_concept_drift() {
        let log = create_test_log();
        let miner = StreamingMiner::new()
            .with_window_size(10)
            .with_step_size(5)
            .with_drift_threshold(0.2);
        let drifts = miner.detect_concept_drift(&log);

        // Should detect drift between first and second batch
        assert!(!drifts.is_empty());
    }

    #[test]
    fn test_compute_stability_metrics() {
        let log = create_test_log();
        let miner = StreamingMiner::new().with_window_size(10).with_step_size(5);
        let (stability, drift_rate) = miner.compute_stability_metrics(&log);

        assert!(stability >= 0.0 && stability <= 1.0);
        assert!(drift_rate >= 0.0 && drift_rate <= 1.0);
        assert_eq!(stability + drift_rate, 1.0);
    }

    #[test]
    fn test_process_event_incremental() {
        let mut dfg = DirectlyFollowsGraph::new();
        let miner = StreamingMiner::new();

        miner.process_event(&mut dfg, "a", "b");
        assert_eq!(dfg.nodes.len(), 2);
        assert_eq!(dfg.edges.len(), 1);

        miner.process_event(&mut dfg, "a", "b");
        assert_eq!(dfg.edges[0].frequency, 2);
    }

    #[test]
    fn test_window_frequency_computation() {
        let mut window = StreamingWindow::new(10);
        window.add_event("a".to_string(), "b".to_string());
        window.add_event("a".to_string(), "b".to_string());
        window.add_event("b".to_string(), "c".to_string());

        let df = window.directly_follows_count();
        assert_eq!(df.get(&("a".to_string(), "b".to_string())), Some(&2));
        assert_eq!(df.get(&("b".to_string(), "c".to_string())), Some(&1));
    }

    #[test]
    fn test_discover_creates_dfg() {
        let log = create_test_log();
        let miner = StreamingMiner::new();
        let dfg = miner.discover(&log);

        assert!(!dfg.nodes.is_empty());
        assert!(!dfg.edges.is_empty());
    }
}
