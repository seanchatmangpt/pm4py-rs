//! Transition System Discovery
//!
//! A transition system is a simple state-based model where each state
//! represents a unique prefix of traces in the log.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Represents a state in the transition system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TSState {
    pub id: usize,
    pub prefix: Vec<String>,
}

impl TSState {
    pub fn new(id: usize, prefix: Vec<String>) -> Self {
        Self { id, prefix }
    }
}

/// Represents a transition in the transition system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TSTransition {
    pub from_state: usize,
    pub to_state: usize,
    pub activity: String,
}

impl TSTransition {
    pub fn new(from_state: usize, to_state: usize, activity: String) -> Self {
        Self {
            from_state,
            to_state,
            activity,
        }
    }
}

/// Transition system representation
#[derive(Debug, Clone)]
pub struct TransitionSystem {
    pub states: Vec<TSState>,
    pub transitions: Vec<TSTransition>,
    pub initial_state: usize,
}

impl TransitionSystem {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            transitions: Vec::new(),
            initial_state: 0,
        }
    }

    pub fn add_state(&mut self, prefix: Vec<String>) -> usize {
        let state_id = self.states.len();
        let state = TSState::new(state_id, prefix);
        self.states.push(state);
        state_id
    }

    pub fn add_transition(&mut self, from: usize, to: usize, activity: String) {
        let transition = TSTransition::new(from, to, activity);
        self.transitions.push(transition);
    }

    pub fn get_state_by_prefix(&self, prefix: &[String]) -> Option<usize> {
        self.states
            .iter()
            .find(|s| s.prefix.as_slice() == prefix)
            .map(|s| s.id)
    }
}

impl Default for TransitionSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover transition system from event log
pub fn discover_transition_system(log: &EventLog) -> TransitionSystem {
    let mut ts = TransitionSystem::new();

    // Add initial state (empty prefix)
    let empty_prefix: Vec<String> = Vec::new();
    ts.initial_state = ts.add_state(empty_prefix);

    // Track all prefixes
    let mut all_prefixes: HashSet<Vec<String>> = HashSet::new();

    for trace in &log.traces {
        let mut prefix = Vec::new();
        all_prefixes.insert(prefix.clone());

        for event in &trace.events {
            prefix.push(event.activity.clone());
            all_prefixes.insert(prefix.clone());
        }
    }

    // Add all states
    for prefix in &all_prefixes {
        ts.add_state(prefix.clone());
    }

    // Add transitions
    for trace in &log.traces {
        let mut current_prefix = Vec::new();

        for event in &trace.events {
            let from_state = ts.get_state_by_prefix(&current_prefix).unwrap();

            current_prefix.push(event.activity.clone());
            let to_state = ts.get_state_by_prefix(&current_prefix).unwrap();

            ts.add_transition(from_state, to_state, event.activity.clone());
        }
    }

    ts
}

/// Transition system with additional information
#[derive(Debug, Clone)]
pub struct AnnotatedTransitionSystem {
    pub ts: TransitionSystem,
    pub state_in_count: HashMap<usize, usize>,
    pub state_out_count: HashMap<usize, usize>,
}

/// Discover annotated transition system
pub fn discover_annotated_transition_system(log: &EventLog) -> AnnotatedTransitionSystem {
    let ts = discover_transition_system(log);
    let mut state_in_count: HashMap<usize, usize> = HashMap::new();
    let mut state_out_count: HashMap<usize, usize> = HashMap::new();

    for transition in &ts.transitions {
        *state_in_count.entry(transition.to_state).or_insert(0) += 1;
        *state_out_count.entry(transition.from_state).or_insert(0) += 1;
    }

    AnnotatedTransitionSystem {
        ts,
        state_in_count,
        state_out_count,
    }
}
