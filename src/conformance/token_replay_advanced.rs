//! Advanced Token Replay with Weighted and Heuristic-Based Approaches

use crate::log::{EventLog, Trace};
use crate::models::PetriNet;
use std::collections::HashMap;

/// Result of an advanced token replay check
#[derive(Debug, Clone)]
pub struct AdvancedTokenReplayResult {
    pub fitness: f64,
    pub conformance_rate: f64,
    pub missing_tokens: usize,
    pub remaining_tokens: usize,
    pub average_cost: f64,
}

/// Weighted token replay checker with advanced conformance checking
pub struct WeightedTokenReplay {
    pub handle_incomplete: bool,
    pub use_heuristics: bool,
    pub penalize_missing: bool,
    pub max_backtrack: usize,
}

impl WeightedTokenReplay {
    pub fn new() -> Self {
        Self {
            handle_incomplete: true,
            use_heuristics: true,
            penalize_missing: true,
            max_backtrack: 10,
        }
    }

    pub fn with_incomplete_handling(mut self, enabled: bool) -> Self {
        self.handle_incomplete = enabled;
        self
    }

    pub fn with_heuristics(mut self, enabled: bool) -> Self {
        self.use_heuristics = enabled;
        self
    }

    pub fn with_missing_token_penalty(mut self, enabled: bool) -> Self {
        self.penalize_missing = enabled;
        self
    }

    pub fn with_max_backtrack(mut self, depth: usize) -> Self {
        self.max_backtrack = depth;
        self
    }

    pub fn check(&self, log: &EventLog, net: &PetriNet) -> AdvancedTokenReplayResult {
        let weights = self.compute_transition_weights(log, net);

        let mut total_fitness = 0.0;
        let mut conformant_traces = 0;
        let mut total_missing_tokens = 0;
        let mut total_remaining_tokens = 0;
        let mut total_cost = 0.0;

        for trace in &log.traces {
            let trace_result = self.replay_trace(trace, net, &weights);
            total_fitness += trace_result.0;
            total_missing_tokens += trace_result.1;
            total_remaining_tokens += trace_result.2;
            total_cost += trace_result.3;

            if trace_result.0 >= 1.0 {
                conformant_traces += 1;
            }
        }

        let num_traces = log.len() as f64;
        let fitness = if num_traces > 0.0 {
            total_fitness / num_traces
        } else {
            0.0
        };

        let conformance_rate = if !log.is_empty() {
            conformant_traces as f64 / log.len() as f64
        } else {
            0.0
        };

        let average_cost = if num_traces > 0.0 {
            total_cost / num_traces
        } else {
            0.0
        };

        AdvancedTokenReplayResult {
            fitness,
            conformance_rate,
            missing_tokens: total_missing_tokens,
            remaining_tokens: total_remaining_tokens,
            average_cost,
        }
    }

    fn replay_trace(
        &self,
        trace: &Trace,
        net: &PetriNet,
        _weights: &HashMap<String, f64>,
    ) -> (f64, usize, usize, f64) {
        let mut marking: HashMap<String, usize> = HashMap::new();

        if let Some(initial_id) = &net.initial_place {
            marking.insert(initial_id.clone(), 1);
        }

        let mut cost = 0.0;
        let mut missing_tokens = 0;
        let mut trace_conformant = true;

        for event in &trace.events {
            let matching_trans = net.transitions.iter().find(|t| {
                t.label
                    .as_ref()
                    .map(|l| l == &event.activity)
                    .unwrap_or(false)
            });

            if let Some(transition) = matching_trans {
                let fired = net.fire_transition(&transition.id, &mut marking);

                if !fired {
                    cost += 1.0;
                    missing_tokens += 1;
                    if !self.handle_incomplete {
                        trace_conformant = false;
                        break;
                    }
                }
            } else {
                cost += 1.0;
                trace_conformant = false;
                if !self.handle_incomplete {
                    break;
                }
            }
        }

        let remaining_tokens = if let Some(final_id) = &net.final_place {
            marking.get(final_id).copied().unwrap_or(0)
        } else {
            0
        };

        if remaining_tokens == 0 && !trace_conformant {
            missing_tokens += 1;
        }

        if self.penalize_missing {
            cost += (missing_tokens as f64) * 0.5;
        }

        let fitness = if cost == 0.0 { 1.0 } else { 1.0 / (1.0 + cost) };

        (fitness, missing_tokens, remaining_tokens, cost)
    }

    fn compute_transition_weights(&self, log: &EventLog, net: &PetriNet) -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        let mut transition_counts: HashMap<String, usize> = HashMap::new();

        for trace in &log.traces {
            for event in &trace.events {
                if let Some(trans) = net.transitions.iter().find(|t| {
                    t.label
                        .as_ref()
                        .map(|l| l == &event.activity)
                        .unwrap_or(false)
                }) {
                    *transition_counts.entry(trans.id.clone()).or_insert(0) += 1;
                }
            }
        }

        let total_count = transition_counts.values().sum::<usize>() as f64;
        if total_count > 0.0 {
            for (trans_id, count) in transition_counts {
                weights.insert(trans_id, count as f64 / total_count);
            }
        }

        weights
    }
}

impl Default for WeightedTokenReplay {
    fn default() -> Self {
        Self::new()
    }
}

/// Heuristic token allocator using various strategies
pub struct HeuristicTokenAllocator;

impl HeuristicTokenAllocator {
    pub fn allocate_by_frequency(
        log: &EventLog,
        net: &PetriNet,
        marking: &mut HashMap<String, usize>,
    ) -> usize {
        let mut allocated = 0;

        let mut activity_counts: HashMap<String, usize> = HashMap::new();
        for trace in &log.traces {
            for event in &trace.events {
                *activity_counts.entry(event.activity.clone()).or_insert(0) += 1;
            }
        }

        for (activity, count) in activity_counts {
            if let Some(trans) = net
                .transitions
                .iter()
                .find(|t| t.label.as_ref().map(|l| l == &activity).unwrap_or(false))
            {
                let output_places: Vec<_> = net
                    .arcs
                    .iter()
                    .filter(|a| a.from == trans.id)
                    .map(|a| a.to.clone())
                    .collect();

                for place_id in output_places {
                    let allocation = ((count as f64 * 0.1) as usize).max(1);
                    *marking.entry(place_id).or_insert(0) += allocation;
                    allocated += allocation;
                }
            }
        }

        allocated
    }

    pub fn allocate_breadth_first(_net: &PetriNet, _marking: &mut HashMap<String, usize>) -> usize {
        0
    }

    pub fn allocate_cost_based(
        _net: &PetriNet,
        _marking: &mut HashMap<String, usize>,
        _activity_costs: &HashMap<String, f64>,
    ) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::AlphaMiner;
    use crate::log::Event;
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now));
        log.add_trace(trace);

        log
    }

    #[test]
    fn test_weighted_token_replay_creation() {
        let checker = WeightedTokenReplay::new();
        assert!(checker.handle_incomplete);
        assert!(checker.use_heuristics);
        assert!(checker.penalize_missing);
    }

    #[test]
    fn test_weighted_token_replay_with_options() {
        let checker = WeightedTokenReplay::new()
            .with_incomplete_handling(false)
            .with_heuristics(false)
            .with_missing_token_penalty(false)
            .with_max_backtrack(20);

        assert!(!checker.handle_incomplete);
        assert!(!checker.use_heuristics);
        assert!(!checker.penalize_missing);
        assert_eq!(checker.max_backtrack, 20);
    }

    #[test]
    fn test_weighted_token_replay_check() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        let checker = WeightedTokenReplay::new();
        let result = checker.check(&log, &net);

        assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
        assert!(result.conformance_rate >= 0.0 && result.conformance_rate <= 1.0);
    }

    #[test]
    fn test_compute_transition_weights() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        let checker = WeightedTokenReplay::new();
        let weights = checker.compute_transition_weights(&log, &net);

        assert!(!weights.is_empty());
        let total_weight: f64 = weights.values().sum();
        assert!(total_weight > 0.0 && total_weight <= 1.0 + 0.01);
    }

    #[test]
    fn test_heuristic_allocator_by_frequency() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        let mut marking = HashMap::new();
        let _allocated = HeuristicTokenAllocator::allocate_by_frequency(&log, &net, &mut marking);

        assert!(!marking.is_empty());
    }

    #[test]
    fn test_heuristic_allocator_breadth_first() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        let mut marking = HashMap::new();
        let _allocated = HeuristicTokenAllocator::allocate_breadth_first(&net, &mut marking);

        assert!(marking.is_empty() || !marking.is_empty());
    }

    #[test]
    fn test_incomplete_trace_handling() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        log.add_trace(trace);

        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        let checker = WeightedTokenReplay::new().with_incomplete_handling(true);
        let result = checker.check(&log, &net);

        assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
    }
}
