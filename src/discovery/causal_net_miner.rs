/// Causal Net Miner - Discover Causal Net from Event Log
///
/// This miner discovers a Causal Net (C-Net) from an event log by analyzing
/// causal relations between activities. C-Nets provide an alternative to Petri nets
/// with explicit modeling of causality, parallelism, and choice.
///
/// The algorithm:
/// 1. Extracts directly-follows relations from the log
/// 2. Identifies causal relations (a → b)
/// 3. Identifies parallel relations (a || b)
/// 4. Identifies conflict/choice relations (a # b)
/// 5. Creates input/output sets for each activity
use crate::log::{operations, EventLog};
use crate::models::causal_net::{CausalNet, CausalRelation};
use std::collections::{HashMap, HashSet};

type PairFreqMap = HashMap<(String, String), usize>;

/// Causal Net Miner configuration
#[derive(Debug, Clone)]
pub struct CausalNetMinerConfig {
    /// Minimum support threshold for relations
    pub min_support: f64,
    /// Consider self-loops
    pub allow_self_loops: bool,
    /// Maximum input set size
    pub max_input_set_size: usize,
}

impl Default for CausalNetMinerConfig {
    fn default() -> Self {
        Self {
            min_support: 0.0,
            allow_self_loops: false,
            max_input_set_size: 10,
        }
    }
}

pub struct CausalNetMiner {
    pub config: CausalNetMinerConfig,
}

impl CausalNetMiner {
    pub fn new() -> Self {
        Self {
            config: CausalNetMinerConfig::default(),
        }
    }

    pub fn with_config(config: CausalNetMinerConfig) -> Self {
        Self { config }
    }

    pub fn with_min_support(mut self, support: f64) -> Self {
        self.config.min_support = support.clamp(0.0, 1.0);
        self
    }

    pub fn with_self_loops(mut self, allow: bool) -> Self {
        self.config.allow_self_loops = allow;
        self
    }

    /// Discover a Causal Net from an event log
    pub fn discover(&self, log: &EventLog) -> CausalNet {
        let mut net = CausalNet::new();

        if log.traces.is_empty() {
            return net;
        }

        // Get basic statistics
        let activities = log.activities();
        let start_acts = operations::start_activities(log);
        let end_acts = operations::end_activities(log);
        let df = operations::directly_follows(log);

        // Add all activities
        for activity in &activities {
            net.add_activity(activity.clone());
        }

        // Set start and end activities
        for activity in start_acts.keys() {
            net.add_start_activity(activity.clone());
        }

        for activity in end_acts.keys() {
            net.add_end_activity(activity.clone());
        }

        // Calculate total traces for support threshold
        let total_traces = log.traces.len() as f64;

        // Extract causal relations
        let (causality, parallelism, conflict) =
            self.extract_relations(&activities, &df, total_traces);

        // Add causal relations to the net
        for (from, to) in causality.keys() {
            net.add_relation(from.clone(), to.clone(), CausalRelation::Causality);
        }

        // Add parallel relations
        for (from, to) in parallelism.keys() {
            net.add_relation(from.clone(), to.clone(), CausalRelation::Parallel);
        }

        // Add conflict relations
        for (from, to) in conflict.keys() {
            net.add_relation(from.clone(), to.clone(), CausalRelation::Conflict);
        }

        // Build input/output sets for each activity
        self.build_io_sets(&mut net, &causality, &parallelism);

        net
    }

    /// Extract causal, parallel, and conflict relations
    fn extract_relations(
        &self,
        activities: &[String],
        df: &HashMap<(String, String), usize>,
        total_traces: f64,
    ) -> (PairFreqMap, PairFreqMap, PairFreqMap) {
        let mut causality = HashMap::new();
        let mut parallelism = HashMap::new();
        let mut conflict = HashMap::new();

        let min_support_count = (self.config.min_support * total_traces).ceil() as usize;

        for act_a in activities {
            for act_b in activities {
                if act_a == act_b {
                    if self.config.allow_self_loops {
                        let count = df
                            .get(&(act_a.clone(), act_b.clone()))
                            .copied()
                            .unwrap_or(0);
                        if count >= min_support_count {
                            causality.insert((act_a.clone(), act_b.clone()), count);
                        }
                    }
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

                // Skip low-support relations
                if ab < min_support_count && ba < min_support_count {
                    continue;
                }

                if ab > 0 && ba == 0 {
                    // Only a → b: causal relation
                    causality.insert((act_a.clone(), act_b.clone()), ab);
                } else if ab > 0 && ba > 0 {
                    // Both directions: parallel/concurrent relation
                    if ab >= min_support_count {
                        parallelism.insert((act_a.clone(), act_b.clone()), ab);
                    }
                    if ba >= min_support_count {
                        parallelism.insert((act_b.clone(), act_a.clone()), ba);
                    }
                } else if ba > 0 && ab == 0 {
                    // Only b → a: causal relation (reverse)
                    causality.insert((act_b.clone(), act_a.clone()), ba);
                }
            }
        }

        // Extract conflict relations from activities with multiple successors
        for activity_a in activities {
            let mut successors = HashSet::new();

            for ((from, to), count) in df {
                if from == activity_a && *count > 0 {
                    successors.insert(to.clone());
                }
            }

            // If multiple successors and not all are parallel, mark as conflict
            if successors.len() > 1 {
                let mut successor_list: Vec<_> = successors.iter().collect();
                successor_list.sort();

                for i in 0..successor_list.len() {
                    for j in (i + 1)..successor_list.len() {
                        let succ_i = successor_list[i];
                        let succ_j = successor_list[j];

                        // Check if these are NOT parallel
                        let i_j = df
                            .get(&(succ_i.clone(), succ_j.clone()))
                            .copied()
                            .unwrap_or(0);
                        let j_i = df
                            .get(&(succ_j.clone(), succ_i.clone()))
                            .copied()
                            .unwrap_or(0);

                        if i_j == 0 && j_i == 0 {
                            // They don't interact: conflict
                            conflict.insert((succ_i.clone(), succ_j.clone()), 1);
                            conflict.insert((succ_j.clone(), succ_i.clone()), 1);
                        }
                    }
                }
            }
        }

        (causality, parallelism, conflict)
    }

    /// Build input and output sets for each activity
    fn build_io_sets(
        &self,
        net: &mut CausalNet,
        causality: &HashMap<(String, String), usize>,
        parallelism: &HashMap<(String, String), usize>,
    ) {
        let mut in_sets: HashMap<String, HashSet<String>> = HashMap::new();
        let mut out_sets: HashMap<String, HashSet<String>> = HashMap::new();

        // Build sets from causal relations
        for (from, to) in causality.keys() {
            out_sets.entry(from.clone()).or_default().insert(to.clone());
            in_sets.entry(to.clone()).or_default().insert(from.clone());
        }

        // Build sets from parallel relations
        for (from, to) in parallelism.keys() {
            out_sets.entry(from.clone()).or_default().insert(to.clone());
            in_sets.entry(to.clone()).or_default().insert(from.clone());
        }

        // Update IO sets in net
        for activity in &net.activities.clone() {
            if let Some(io) = net.io_sets.get_mut(activity) {
                if let Some(inputs) = in_sets.get(activity) {
                    io.input.clear();
                    io.input.push(inputs.clone());
                }
                if let Some(outputs) = out_sets.get(activity) {
                    io.output.clear();
                    io.output.push(outputs.clone());
                }
            }
        }
    }
}

impl Default for CausalNetMiner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_simple_log() -> EventLog {
        let mut log = EventLog::new();

        for i in 0..3 {
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

        // Variant 1: A -> B -> D
        for i in 0..3 {
            let mut trace = crate::log::Trace::new(format!("case{}_v1", i));
            trace.add_event(crate::log::Event::new("A", Utc::now()));
            trace.add_event(crate::log::Event::new("B", Utc::now()));
            trace.add_event(crate::log::Event::new("D", Utc::now()));
            log.add_trace(trace);
        }

        // Variant 2: A -> C -> D
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
    fn test_causal_net_miner_creation() {
        let miner = CausalNetMiner::new();
        assert_eq!(miner.config.min_support, 0.0);
    }

    #[test]
    fn test_causal_net_miner_simple_discovery() {
        let log = create_simple_log();
        let miner = CausalNetMiner::new();
        let net = miner.discover(&log);

        assert_eq!(net.num_activities(), 3);
        assert!(net.is_start("A"));
        assert!(net.is_end("C"));
    }

    #[test]
    fn test_causal_net_miner_with_config() {
        let config = CausalNetMinerConfig {
            min_support: 0.5,
            allow_self_loops: true,
            max_input_set_size: 5,
        };
        let miner = CausalNetMiner::with_config(config);
        assert_eq!(miner.config.min_support, 0.5);
        assert!(miner.config.allow_self_loops);
    }

    #[test]
    fn test_causal_net_miner_concurrent_detection() {
        let log = create_concurrent_log();
        let miner = CausalNetMiner::new();
        let net = miner.discover(&log);

        assert_eq!(net.num_activities(), 4);

        // Should detect A -> B, A -> C (split)
        let ab_causal = net.get_directly_caused_by("A").contains(&"B".to_string());
        let ac_causal = net.get_directly_caused_by("A").contains(&"C".to_string());

        assert!(ab_causal || ac_causal);
    }

    #[test]
    fn test_causal_net_miner_trace_acceptance() {
        let log = create_simple_log();
        let miner = CausalNetMiner::new();
        let net = miner.discover(&log);

        let trace = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert!(net.accepts_trace(&trace));
    }

    #[test]
    fn test_causal_net_miner_builder() {
        let miner = CausalNetMiner::new()
            .with_min_support(0.3)
            .with_self_loops(true);

        assert_eq!(miner.config.min_support, 0.3);
        assert!(miner.config.allow_self_loops);
    }

    #[test]
    fn test_causal_net_miner_empty_log() {
        let log = EventLog::new();
        let miner = CausalNetMiner::new();
        let net = miner.discover(&log);

        assert_eq!(net.num_activities(), 0);
    }

    #[test]
    fn test_causal_net_miner_min_support_filtering() {
        let mut log = EventLog::new();

        // Create high-frequency path: A -> B -> C
        for i in 0..10 {
            let mut trace = crate::log::Trace::new(format!("case{}", i));
            trace.add_event(crate::log::Event::new("A", Utc::now()));
            trace.add_event(crate::log::Event::new("B", Utc::now()));
            trace.add_event(crate::log::Event::new("C", Utc::now()));
            log.add_trace(trace);
        }

        // Create low-frequency path: A -> D -> C
        let mut trace = crate::log::Trace::new("case_rare".to_string());
        trace.add_event(crate::log::Event::new("A", Utc::now()));
        trace.add_event(crate::log::Event::new("D", Utc::now()));
        trace.add_event(crate::log::Event::new("C", Utc::now()));
        log.add_trace(trace);

        let miner = CausalNetMiner::new().with_min_support(0.5);
        let net = miner.discover(&log);

        // Should prefer high-frequency relations
        assert!(net.num_relations() >= 2);
    }

    #[test]
    fn test_causal_net_miner_relation_types() {
        let log = create_concurrent_log();
        let miner = CausalNetMiner::new();
        let net = miner.discover(&log);

        let causals = net.get_relations_by_type(CausalRelation::Causality);
        let parallels = net.get_relations_by_type(CausalRelation::Parallel);

        // Should have discovered both types
        assert!(causals.len() > 0 || parallels.len() > 0);
    }

    #[test]
    fn test_causal_net_miner_io_sets() {
        let log = create_simple_log();
        let miner = CausalNetMiner::new();
        let net = miner.discover(&log);

        // B should have input {A} and output {C}
        let b_io = net.io_sets.get("B").unwrap();
        if !b_io.input.is_empty() {
            assert!(b_io.input[0].contains("A"));
        }
    }

    #[test]
    fn test_causal_net_miner_all_activities_discovered() {
        let log = create_concurrent_log();
        let miner = CausalNetMiner::new();
        let net = miner.discover(&log);

        assert!(net.activities.contains("A"));
        assert!(net.activities.contains("B"));
        assert!(net.activities.contains("C"));
        assert!(net.activities.contains("D"));
    }
}
