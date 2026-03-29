use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a state in a transition system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct State {
    pub id: String,
    pub label: Option<String>,
    pub is_initial: bool,
    pub is_final: bool,
}

impl State {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: Some(label.into()),
            is_initial: false,
            is_final: false,
        }
    }

    pub fn with_initial(mut self, is_initial: bool) -> Self {
        self.is_initial = is_initial;
        self
    }

    pub fn with_final(mut self, is_final: bool) -> Self {
        self.is_final = is_final;
        self
    }
}

/// Represents a transition in a transition system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemTransition {
    pub from_state: String,
    pub to_state: String,
    pub label: String,
}

impl SystemTransition {
    pub fn new(from: impl Into<String>, to: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            from_state: from.into(),
            to_state: to.into(),
            label: label.into(),
        }
    }
}

/// Represents a transition system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionSystem {
    pub states: Vec<State>,
    pub transitions: Vec<SystemTransition>,
}

impl TransitionSystem {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn add_state(&mut self, state: State) {
        self.states.push(state);
    }

    pub fn add_transition(&mut self, transition: SystemTransition) {
        self.transitions.push(transition);
    }

    pub fn get_state(&self, id: &str) -> Option<&State> {
        self.states.iter().find(|s| s.id == id)
    }

    pub fn get_transitions_from(&self, state_id: &str) -> Vec<&SystemTransition> {
        self.transitions
            .iter()
            .filter(|t| t.from_state == state_id)
            .collect()
    }

    pub fn get_transitions_to(&self, state_id: &str) -> Vec<&SystemTransition> {
        self.transitions
            .iter()
            .filter(|t| t.to_state == state_id)
            .collect()
    }

    pub fn initial_states(&self) -> Vec<&State> {
        self.states.iter().filter(|s| s.is_initial).collect()
    }

    pub fn final_states(&self) -> Vec<&State> {
        self.states.iter().filter(|s| s.is_final).collect()
    }

    /// Get all traces accepted by the transition system
    pub fn get_traces(&self) -> Vec<Vec<String>> {
        let mut traces = Vec::new();

        for initial in self.initial_states() {
            self.dfs(&initial.id, Vec::new(), &mut traces);
        }

        traces
    }

    fn dfs(&self, state_id: &str, path: Vec<String>, traces: &mut Vec<Vec<String>>) {
        if let Some(state) = self.get_state(state_id) {
            if state.is_final {
                traces.push(path.clone());
            }

            let transitions = self.get_transitions_from(state_id);
            if transitions.is_empty() && state.is_final {
                return;
            }

            for transition in transitions {
                let mut new_path = path.clone();
                new_path.push(transition.label.clone());
                self.dfs(&transition.to_state, new_path, traces);
            }
        }
    }

    /// Check if a trace is accepted by the transition system
    pub fn accepts_trace(&self, trace: &[&str]) -> bool {
        let initial_states = self.initial_states();

        for initial_state in initial_states {
            if self.trace_accepted_from(&initial_state.id, trace) {
                return true;
            }
        }

        false
    }

    fn trace_accepted_from(&self, state_id: &str, trace: &[&str]) -> bool {
        if trace.is_empty() {
            if let Some(state) = self.get_state(state_id) {
                return state.is_final;
            }
            return false;
        }

        let transitions = self.get_transitions_from(state_id);

        for transition in transitions {
            if transition.label == trace[0]
                && self.trace_accepted_from(&transition.to_state, &trace[1..])
            {
                return true;
            }
        }

        false
    }
}

impl Default for TransitionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_system_creation() {
        let mut ts = TransitionSystem::new();
        let s1 = State::new("s1").with_initial(true);
        let s2 = State::new("s2").with_final(true);

        ts.add_state(s1);
        ts.add_state(s2);

        assert_eq!(ts.states.len(), 2);
    }

    #[test]
    fn test_trace_acceptance() {
        let mut ts = TransitionSystem::new();
        let s1 = State::new("s1").with_initial(true);
        let s2 = State::new("s2").with_final(true);

        let s1_id = s1.id.clone();
        let s2_id = s2.id.clone();

        ts.add_state(s1);
        ts.add_state(s2);
        ts.add_transition(SystemTransition::new(&s1_id, &s2_id, "a"));

        assert!(ts.accepts_trace(&["a"]));
        assert!(!ts.accepts_trace(&["b"]));
    }
}
