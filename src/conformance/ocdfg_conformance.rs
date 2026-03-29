//! OCDFG (Object-Centric Directly-Follows Graph) Conformance
//!
//! Conformance checking for object-centric directly-follows graphs.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// OCDFG conformance result
#[derive(Debug, Clone)]
pub struct OCDFGConformanceResult {
    /// Deviations detected
    pub deviations: usize,
    /// Total checks performed
    pub total_checks: usize,
    /// Fitness value (0-1)
    pub fitness: f64,
    /// Missing edges in log
    pub missing_edges: Vec<(String, String)>,
    /// Extra edges in log
    pub extra_edges: Vec<(String, String)>,
}

impl OCDFGConformanceResult {
    pub fn new() -> Self {
        Self {
            deviations: 0,
            total_checks: 0,
            fitness: 1.0,
            missing_edges: Vec::new(),
            extra_edges: Vec::new(),
        }
    }

    pub fn calculate_fitness(&mut self) {
        if self.total_checks == 0 {
            self.fitness = 1.0;
        } else {
            self.fitness = 1.0 - (self.deviations as f64 / self.total_checks as f64);
        }
    }
}

impl Default for OCDFGConformanceResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Conformance checking for OCDFG
///
/// Compares an OCDFG model against an event log.
pub fn conformance_ocdfg(
    log: &EventLog,
    model_dfg: &HashMap<(String, String), usize>,
) -> OCDFGConformanceResult {
    let mut result = OCDFGConformanceResult::new();

    // Discover DFG from log
    let log_dfg = discover_dfg_from_log(log);

    // Check for missing edges (in model but not in log)
    for edge in model_dfg.keys() {
        result.total_checks += 1;
        if !log_dfg.contains_key(edge) {
            result.deviations += 1;
            result.missing_edges.push(edge.clone());
        }
    }

    // Check for extra edges (in log but not in model)
    for edge in log_dfg.keys() {
        result.total_checks += 1;
        if !model_dfg.contains_key(edge) {
            result.deviations += 1;
            result.extra_edges.push(edge.clone());
        }
    }

    result.calculate_fitness();
    result
}

/// Discover DFG from event log
fn discover_dfg_from_log(log: &EventLog) -> HashMap<(String, String), usize> {
    let mut dfg: HashMap<(String, String), usize> = HashMap::new();

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            let from = &window[0].activity;
            let to = &window[1].activity;
            *dfg.entry((from.clone(), to.clone())).or_insert(0) += 1;
        }
    }

    dfg
}

/// OTG conformance result
#[derive(Debug, Clone)]
pub struct OTGConformanceResult {
    /// Deviations detected
    pub deviations: usize,
    /// Total checks performed
    pub total_checks: usize,
    /// Fitness value (0-1)
    pub fitness: f64,
    /// Missing transitions
    pub missing_transitions: Vec<(String, String)>,
    /// Extra transitions
    pub extra_transitions: Vec<(String, String)>,
}

impl OTGConformanceResult {
    pub fn new() -> Self {
        Self {
            deviations: 0,
            total_checks: 0,
            fitness: 1.0,
            missing_transitions: Vec::new(),
            extra_transitions: Vec::new(),
        }
    }

    pub fn calculate_fitness(&mut self) {
        if self.total_checks == 0 {
            self.fitness = 1.0;
        } else {
            self.fitness = 1.0 - (self.deviations as f64 / self.total_checks as f64);
        }
    }
}

impl Default for OTGConformanceResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Conformance checking for OTG (Occurrence Transition Graph)
///
/// Compares an OTG model against an event log.
pub fn conformance_otg(
    log: &EventLog,
    model_transitions: &HashSet<(String, String)>,
) -> OTGConformanceResult {
    let mut result = OTGConformanceResult::new();

    // Discover transitions from log
    let log_transitions = discover_transitions_from_log(log);

    // Check for missing transitions
    for transition in model_transitions {
        result.total_checks += 1;
        if !log_transitions.contains(transition) {
            result.deviations += 1;
            result.missing_transitions.push(transition.clone());
        }
    }

    // Check for extra transitions
    for transition in &log_transitions {
        result.total_checks += 1;
        if !model_transitions.contains(transition) {
            result.deviations += 1;
            result.extra_transitions.push(transition.clone());
        }
    }

    result.calculate_fitness();
    result
}

/// Discover transitions from event log
fn discover_transitions_from_log(log: &EventLog) -> HashSet<(String, String)> {
    let mut transitions: HashSet<(String, String)> = HashSet::new();

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            let from = &window[0].activity;
            let to = &window[1].activity;
            transitions.insert((from.clone(), to.clone()));
        }
    }

    transitions
}
