use serde::{Deserialize, Serialize};
/// Petri net representation and operations
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a place in a Petri net
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub initial_marking: usize,
    pub final_marking: Option<usize>,
}

impl Place {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            initial_marking: 0,
            final_marking: None,
        }
    }

    pub fn with_initial_marking(mut self, marking: usize) -> Self {
        self.initial_marking = marking;
        self
    }

    pub fn with_final_marking(mut self, marking: usize) -> Self {
        self.final_marking = Some(marking);
        self
    }
}

/// Represents a transition in a Petri net
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Transition {
    pub id: String,
    pub label: Option<String>,
    pub name: String,
}

impl Transition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: None,
            name: name.into(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Check if this is an invisible transition (tau)
    pub fn is_invisible(&self) -> bool {
        self.label.is_none()
    }
}

/// Represents an arc (edge) in a Petri net
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Arc {
    pub from: String,
    pub to: String,
    pub weight: usize,
}

impl Arc {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            weight: 1,
        }
    }

    pub fn with_weight(mut self, weight: usize) -> Self {
        self.weight = weight;
        self
    }
}

/// Represents a complete Petri net
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
    pub initial_place: Option<String>,
    pub final_place: Option<String>,
}

impl PetriNet {
    pub fn new() -> Self {
        Self {
            places: Vec::new(),
            transitions: Vec::new(),
            arcs: Vec::new(),
            initial_place: None,
            final_place: None,
        }
    }

    /// Add a place to the net
    pub fn add_place(&mut self, place: Place) {
        self.places.push(place);
    }

    /// Add a transition to the net
    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    /// Add an arc to the net
    pub fn add_arc(&mut self, arc: Arc) {
        self.arcs.push(arc);
    }

    /// Get place by ID
    pub fn get_place(&self, id: &str) -> Option<&Place> {
        self.places.iter().find(|p| p.id == id)
    }

    /// Get transition by ID
    pub fn get_transition(&self, id: &str) -> Option<&Transition> {
        self.transitions.iter().find(|t| t.id == id)
    }

    /// Get arcs from a node
    pub fn get_arcs_from(&self, id: &str) -> Vec<&Arc> {
        self.arcs.iter().filter(|a| a.from == id).collect()
    }

    /// Get arcs to a node
    pub fn get_arcs_to(&self, id: &str) -> Vec<&Arc> {
        self.arcs.iter().filter(|a| a.to == id).collect()
    }

    /// Set initial place
    pub fn set_initial_place(&mut self, place_id: String) {
        self.initial_place = Some(place_id);
    }

    /// Set final place
    pub fn set_final_place(&mut self, place_id: String) {
        self.final_place = Some(place_id);
    }

    /// Get all visible transitions
    pub fn visible_transitions(&self) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| !t.is_invisible())
            .collect()
    }

    /// Get all invisible transitions
    pub fn invisible_transitions(&self) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| t.is_invisible())
            .collect()
    }

    /// Get source places (places with no incoming arcs)
    pub fn source_places(&self) -> Vec<&Place> {
        self.places
            .iter()
            .filter(|p| {
                !self
                    .get_arcs_to(&p.id)
                    .iter()
                    .any(|a| self.get_transition(&a.from).is_some())
            })
            .collect()
    }

    /// Get sink places (places with no outgoing arcs)
    pub fn sink_places(&self) -> Vec<&Place> {
        self.places
            .iter()
            .filter(|p| {
                !self
                    .get_arcs_from(&p.id)
                    .iter()
                    .any(|a| self.get_transition(&a.to).is_some())
            })
            .collect()
    }

    /// Check if net is a workflow net (single source, single sink)
    pub fn is_workflow_net(&self) -> bool {
        let sources = self.source_places();
        let sinks = self.sink_places();
        sources.len() == 1 && sinks.len() == 1
    }

    /// Check if transition is enabled (all input places have tokens)
    pub fn is_transition_enabled(
        &self,
        transition_id: &str,
        marking: &HashMap<String, usize>,
    ) -> bool {
        let input_arcs = self.get_arcs_to(transition_id);

        input_arcs.iter().all(|arc| {
            if let Some(place) = self.get_place(&arc.from) {
                marking.get(&place.id).copied().unwrap_or(0) >= arc.weight
            } else {
                false
            }
        })
    }

    /// Fire a transition (update marking)
    pub fn fire_transition(
        &self,
        transition_id: &str,
        marking: &mut HashMap<String, usize>,
    ) -> bool {
        if !self.is_transition_enabled(transition_id, marking) {
            return false;
        }

        // Remove tokens from input places
        for arc in self.get_arcs_to(transition_id) {
            *marking.entry(arc.from.clone()).or_insert(0) -= arc.weight;
        }

        // Add tokens to output places
        for arc in self.get_arcs_from(transition_id) {
            *marking.entry(arc.to.clone()).or_insert(0) += arc.weight;
        }

        // Prune zero-token entries so marking equality works correctly
        marking.retain(|_, v| *v > 0);

        true
    }

    /// Count reachable states (simplified)
    pub fn count_reachable_states(&self, initial_marking: &HashMap<String, usize>) -> usize {
        let mut visited_count = 0;
        let mut visited_markings: Vec<HashMap<String, usize>> = Vec::new();
        let mut queue = vec![initial_marking.clone()];

        while let Some(current) = queue.pop() {
            // Check if we've seen this marking before
            if visited_markings.iter().any(|m| m == &current) {
                continue;
            }

            visited_markings.push(current.clone());
            visited_count += 1;

            for transition in &self.transitions {
                let mut new_marking = current.clone();
                if self.fire_transition(&transition.id, &mut new_marking)
                    && !visited_markings.iter().any(|m| m == &new_marking)
                {
                    queue.push(new_marking);
                }
            }
        }

        visited_count
    }
}

impl Default for PetriNet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_petri_net_creation() {
        let mut net = PetriNet::new();
        let p1 = Place::new("p1").with_initial_marking(1);
        let t1 = Transition::new("t1");
        let p2 = Place::new("p2");

        net.add_place(p1);
        net.add_transition(t1);
        net.add_place(p2);

        assert_eq!(net.places.len(), 2);
        assert_eq!(net.transitions.len(), 1);
    }

    #[test]
    fn test_transition_firing() {
        let mut net = PetriNet::new();
        let p1 = Place::new("p1").with_initial_marking(1);
        let t1 = Transition::new("t1");
        let p2 = Place::new("p2");

        let p1_id = p1.id.clone();
        let t1_id = t1.id.clone();
        let p2_id = p2.id.clone();

        net.add_place(p1);
        net.add_transition(t1);
        net.add_place(p2);

        net.add_arc(Arc::new(&p1_id, &t1_id));
        net.add_arc(Arc::new(&t1_id, &p2_id));

        let mut marking = HashMap::new();
        marking.insert(p1_id.clone(), 1);

        assert!(net.fire_transition(&t1_id, &mut marking));
        assert_eq!(marking.get(&p1_id), None);
        assert_eq!(marking.get(&p2_id), Some(&1));
    }

    #[test]
    fn test_workflow_net_detection() {
        let mut net = PetriNet::new();
        let start = Place::new("start").with_initial_marking(1);
        let end = Place::new("end");
        let t1 = Transition::new("t1");

        let start_id = start.id.clone();
        let end_id = end.id.clone();
        let t1_id = t1.id.clone();

        net.add_place(start);
        net.add_transition(t1);
        net.add_place(end);

        net.add_arc(Arc::new(&start_id, &t1_id));
        net.add_arc(Arc::new(&t1_id, &end_id));

        assert!(net.is_workflow_net());
    }
}
