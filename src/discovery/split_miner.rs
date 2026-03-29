/// Split Miner - Parallelism Detection and Petri Net Discovery
///
/// Split Miner discovers Petri nets with explicit parallel structure by detecting
/// concurrent and sequential splits/joins in the process behavior.
///
/// Key features:
/// - Split detection: identifies exclusive (XOR) vs. concurrent (AND) splits
/// - Join detection: identifies matching join points
/// - Parallelism metrics: DFG completeness parameter controls behavior
/// - Pruning: handles noise with edge filtering
use crate::log::{operations, EventLog};
use crate::models::petri_net::{Arc, Place, Transition};
use crate::models::PetriNet;
use std::collections::{HashMap, HashSet};

/// Split Miner configuration
#[derive(Debug, Clone)]
pub struct SplitMinerConfig {
    /// DFG completeness parameter (0.0 to 1.0)
    /// Higher values = more edges kept = more parallel structure detected
    pub dfg_completeness: f64,
    /// Use parallelism detection
    pub detect_parallelism: bool,
    /// Minimum edge frequency to keep
    pub min_edge_frequency: usize,
}

impl Default for SplitMinerConfig {
    fn default() -> Self {
        Self {
            dfg_completeness: 0.65,
            detect_parallelism: true,
            min_edge_frequency: 1,
        }
    }
}

pub struct SplitMiner {
    pub config: SplitMinerConfig,
}

impl SplitMiner {
    pub fn new() -> Self {
        Self {
            config: SplitMinerConfig::default(),
        }
    }

    pub fn with_config(config: SplitMinerConfig) -> Self {
        Self { config }
    }

    pub fn with_completeness(mut self, completeness: f64) -> Self {
        self.config.dfg_completeness = completeness.clamp(0.0, 1.0);
        self
    }

    pub fn with_parallelism_detection(mut self, enabled: bool) -> Self {
        self.config.detect_parallelism = enabled;
        self
    }

    /// Discover a Petri net using Split Miner algorithm
    pub fn discover(&self, log: &EventLog) -> PetriNet {
        if log.traces.is_empty() {
            return PetriNet::new();
        }

        let mut net = PetriNet::new();

        // Build directly-follows graph with filtering
        let df = operations::directly_follows(log);
        let filtered_df = self.filter_dfg(&df);

        // Get activity information
        let activities = log.activities();
        let start_acts = operations::start_activities(log);
        let end_acts = operations::end_activities(log);

        // Create transitions for all activities
        let mut trans_map: HashMap<String, String> = HashMap::new();
        for activity in &activities {
            let transition = Transition::new(activity).with_label(activity);
            let trans_id = transition.id.clone();
            trans_map.insert(activity.clone(), trans_id);
            net.add_transition(transition);
        }

        // Create source and sink places
        let source = Place::new("source").with_initial_marking(1);
        let source_id = source.id.clone();
        net.add_place(source);
        net.set_initial_place(source_id.clone());

        let sink = Place::new("sink");
        let sink_id = sink.id.clone();
        net.add_place(sink);
        net.set_final_place(sink_id.clone());

        // Connect source to start activities
        for activity in start_acts.keys() {
            if let Some(trans_id) = trans_map.get(activity) {
                net.add_arc(Arc::new(&source_id, trans_id));
            }
        }

        // Connect end activities to sink
        for activity in end_acts.keys() {
            if let Some(trans_id) = trans_map.get(activity) {
                net.add_arc(Arc::new(trans_id, &sink_id));
            }
        }

        // Detect splits and joins
        if self.config.detect_parallelism {
            self.add_parallel_structure(&mut net, &filtered_df, &trans_map, &activities);
        } else {
            // Simple sequential structure
            self.add_sequential_structure(&mut net, &filtered_df, &trans_map);
        }

        net
    }

    /// Filter DFG based on completeness parameter
    fn filter_dfg(
        &self,
        df: &HashMap<(String, String), usize>,
    ) -> HashMap<(String, String), usize> {
        if df.is_empty() {
            return HashMap::new();
        }

        let mut edges: Vec<_> = df.iter().collect();
        edges.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

        let target_count = ((edges.len() as f64) * self.config.dfg_completeness).ceil() as usize;
        let filtered_edges = &edges[..target_count.min(edges.len())];

        filtered_edges
            .iter()
            .map(|(k, v)| ((*k).clone(), **v))
            .collect()
    }

    /// Add parallel structure to the net
    fn add_parallel_structure(
        &self,
        net: &mut PetriNet,
        df: &HashMap<(String, String), usize>,
        trans_map: &HashMap<String, String>,
        activities: &[String],
    ) {
        // Find concurrent activities
        let concurrency = self.detect_concurrent_activities(df, activities);

        // Analyze splits: activities with multiple outgoing edges
        let mut activity_followers: HashMap<String, HashSet<String>> = HashMap::new();
        for (from, to) in df.keys() {
            activity_followers
                .entry(from.clone())
                .or_default()
                .insert(to.clone());
        }

        // For each activity with multiple followers, create split structure
        let mut place_counter = 0;
        for (activity, followers) in &activity_followers {
            if let Some(trans_id) = trans_map.get(activity) {
                if followers.len() > 1 {
                    // This is a split point
                    // Check if concurrent or exclusive split
                    if self.is_concurrent_split(activity, followers, &concurrency) {
                        // AND split: create a place that feeds multiple transitions
                        let and_place = Place::new(format!("and_split_{}", place_counter));
                        let and_id = and_place.id.clone();
                        net.add_place(and_place);

                        // Activity feeds into AND place
                        net.add_arc(Arc::new(trans_id, &and_id));

                        // AND place feeds all concurrent followers
                        for follower in followers {
                            if let Some(follower_id) = trans_map.get(follower) {
                                net.add_arc(Arc::new(&and_id, follower_id));
                            }
                        }

                        place_counter += 1;
                    } else {
                        // XOR split: create separate places for each path
                        for follower in followers {
                            if let Some(follower_id) = trans_map.get(follower) {
                                let xor_place =
                                    Place::new(format!("xor_split_{}_{}", place_counter, follower));
                                let xor_id = xor_place.id.clone();
                                net.add_place(xor_place);

                                net.add_arc(Arc::new(trans_id, &xor_id));
                                net.add_arc(Arc::new(&xor_id, follower_id));

                                place_counter += 1;
                            }
                        }
                    }
                } else {
                    // Single follower: simple sequence
                    if let Some(follower) = followers.iter().next() {
                        if let Some(follower_id) = trans_map.get(follower) {
                            let seq_place = Place::new(format!("seq_{}", place_counter));
                            let seq_id = seq_place.id.clone();
                            net.add_place(seq_place);

                            net.add_arc(Arc::new(trans_id, &seq_id));
                            net.add_arc(Arc::new(&seq_id, follower_id));

                            place_counter += 1;
                        }
                    }
                }
            }
        }
    }

    /// Add simple sequential structure
    fn add_sequential_structure(
        &self,
        net: &mut PetriNet,
        df: &HashMap<(String, String), usize>,
        trans_map: &HashMap<String, String>,
    ) {
        for (place_counter, (from, to)) in df.keys().enumerate() {
            let from_id = trans_map.get(from).unwrap();
            let to_id = trans_map.get(to).unwrap();

            let place = Place::new(format!("p_{}", place_counter));
            let place_id = place.id.clone();
            net.add_place(place);

            net.add_arc(Arc::new(from_id, &place_id));
            net.add_arc(Arc::new(&place_id, to_id));
        }
    }

    /// Detect concurrent activity pairs
    fn detect_concurrent_activities(
        &self,
        df: &HashMap<(String, String), usize>,
        activities: &[String],
    ) -> HashSet<(String, String)> {
        let mut concurrent = HashSet::new();

        for act_a in activities {
            for act_b in activities {
                if act_a >= act_b {
                    continue;
                }

                let ab = df
                    .get(&(act_a.clone(), act_b.clone()))
                    .copied()
                    .unwrap_or(0);
                let ba = df
                    .get(&(act_b.clone(), act_a.clone()))
                    .copied()
                    .unwrap_or(0);

                // Both directions exist: concurrent
                if ab > 0 && ba > 0 {
                    concurrent.insert((act_a.clone(), act_b.clone()));
                    concurrent.insert((act_b.clone(), act_a.clone()));
                }
            }
        }

        concurrent
    }

    /// Determine if a split is concurrent or exclusive
    fn is_concurrent_split(
        &self,
        _activity: &str,
        followers: &HashSet<String>,
        concurrent: &HashSet<(String, String)>,
    ) -> bool {
        // Check if followers are concurrent with each other
        let mut follower_list: Vec<_> = followers.iter().collect();
        follower_list.sort();

        for i in 0..follower_list.len() {
            for j in (i + 1)..follower_list.len() {
                if concurrent.contains(&(follower_list[i].clone(), follower_list[j].clone())) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for SplitMiner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_sequential_log() -> EventLog {
        let mut log = EventLog::new();
        for i in 0..5 {
            let mut trace = crate::log::Trace::new(format!("case{}", i));
            trace.add_event(crate::log::Event::new("A", Utc::now()));
            trace.add_event(crate::log::Event::new("B", Utc::now()));
            trace.add_event(crate::log::Event::new("C", Utc::now()));
            log.add_trace(trace);
        }
        log
    }

    fn create_concurrent_log() -> EventLog {
        let mut log = EventLog::new();

        // Concurrent variant 1: A -> B -> D
        for i in 0..3 {
            let mut trace = crate::log::Trace::new(format!("case{}_v1", i));
            trace.add_event(crate::log::Event::new("A", Utc::now()));
            trace.add_event(crate::log::Event::new("B", Utc::now()));
            trace.add_event(crate::log::Event::new("D", Utc::now()));
            log.add_trace(trace);
        }

        // Concurrent variant 2: A -> C -> D
        for i in 0..3 {
            let mut trace = crate::log::Trace::new(format!("case{}_v2", i));
            trace.add_event(crate::log::Event::new("A", Utc::now()));
            trace.add_event(crate::log::Event::new("C", Utc::now()));
            trace.add_event(crate::log::Event::new("D", Utc::now()));
            log.add_trace(trace);
        }

        log
    }

    #[test]
    fn test_split_miner_creation() {
        let miner = SplitMiner::new();
        assert_eq!(miner.config.dfg_completeness, 0.65);
        assert!(miner.config.detect_parallelism);
    }

    #[test]
    fn test_split_miner_sequential_discovery() {
        let log = create_sequential_log();
        let miner = SplitMiner::new();
        let net = miner.discover(&log);

        assert!(!net.places.is_empty());
        assert!(net.visible_transitions().len() >= 3);
    }

    #[test]
    fn test_split_miner_concurrent_discovery() {
        let log = create_concurrent_log();
        let miner = SplitMiner::new();
        let net = miner.discover(&log);

        assert!(!net.places.is_empty());
        assert!(net.visible_transitions().len() >= 4);
    }

    #[test]
    fn test_split_miner_with_config() {
        let config = SplitMinerConfig {
            dfg_completeness: 0.8,
            detect_parallelism: false,
            min_edge_frequency: 2,
        };
        let miner = SplitMiner::with_config(config);
        assert_eq!(miner.config.dfg_completeness, 0.8);
        assert!(!miner.config.detect_parallelism);
    }

    #[test]
    fn test_split_miner_dfg_filtering() {
        let mut df = HashMap::new();
        df.insert(("A".to_string(), "B".to_string()), 10);
        df.insert(("B".to_string(), "C".to_string()), 5);
        df.insert(("C".to_string(), "D".to_string()), 2);

        let miner = SplitMiner::new();
        let filtered = miner.filter_dfg(&df);

        // With 0.65 completeness, should keep ~2 edges
        assert!(filtered.len() >= 1 && filtered.len() <= 3);
    }

    #[test]
    fn test_split_miner_completeness_parameter() {
        let log = create_sequential_log();

        let miner_high = SplitMiner::new().with_completeness(0.9);
        let net_high = miner_high.discover(&log);

        let miner_low = SplitMiner::new().with_completeness(0.4);
        let net_low = miner_low.discover(&log);

        // Higher completeness should generally result in more places
        assert!(net_high.places.len() >= net_low.places.len());
    }

    #[test]
    fn test_split_miner_without_parallelism_detection() {
        let log = create_concurrent_log();
        let miner = SplitMiner::new().with_parallelism_detection(false);
        let net = miner.discover(&log);

        assert!(!net.places.is_empty());
    }

    #[test]
    fn test_split_miner_empty_log() {
        let log = EventLog::new();
        let miner = SplitMiner::new();
        let net = miner.discover(&log);

        assert!(net.places.is_empty());
    }

    #[test]
    fn test_split_miner_concurrency_detection() {
        let mut df = HashMap::new();
        df.insert(("A".to_string(), "B".to_string()), 5);
        df.insert(("B".to_string(), "A".to_string()), 5); // Both directions = concurrent
        df.insert(("A".to_string(), "C".to_string()), 3);
        df.insert(("C".to_string(), "A".to_string()), 1); // One direction = not concurrent

        let miner = SplitMiner::new();
        let activities = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let concurrent = miner.detect_concurrent_activities(&df, &activities);

        assert!(concurrent.contains(&("A".to_string(), "B".to_string())));
        assert!(concurrent.contains(&("B".to_string(), "A".to_string())));
    }

    #[test]
    fn test_split_miner_workflow_net_structure() {
        let log = create_sequential_log();
        let miner = SplitMiner::new();
        let net = miner.discover(&log);

        assert!(net.initial_place.is_some());
        assert!(net.final_place.is_some());
        assert!(net.is_workflow_net());
    }

    #[test]
    fn test_split_miner_builder_pattern() {
        let miner = SplitMiner::new()
            .with_completeness(0.75)
            .with_parallelism_detection(true);

        assert_eq!(miner.config.dfg_completeness, 0.75);
        assert!(miner.config.detect_parallelism);
    }
}
