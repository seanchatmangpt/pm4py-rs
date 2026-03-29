/// Simplicity Metric - Model complexity assessment
///
/// Simplicity = 1 - (normalized complexity)
/// Measures: model size, cyclomatic complexity, structuredness
use crate::models::petri_net::PetriNet;
use std::collections::HashMap;

pub struct Simplicity;

impl Simplicity {
    /// Calculate simplicity (inverse of complexity) for a Petri net
    pub fn calculate(net: &PetriNet) -> f64 {
        if net.transitions.is_empty() {
            return 1.0;
        }

        let num_activities = net.visible_transitions().len();

        if num_activities == 0 {
            return 1.0;
        }

        let size_complexity = Self::calculate_size_complexity(net, num_activities);
        let cyclomatic_complexity = Self::calculate_cyclomatic_complexity(net);
        let structuredness = Self::calculate_structuredness(net);

        let normalized_complexity = size_complexity + cyclomatic_complexity * 0.5;
        let adjusted_complexity = normalized_complexity * (1.0 - structuredness * 0.3);

        1.0 / (1.0 + adjusted_complexity)
    }

    fn calculate_size_complexity(net: &PetriNet, num_activities: usize) -> f64 {
        let num_places = net.places.len();
        let num_transitions = net.transitions.len();
        let size = (num_places + num_transitions) as f64;

        let normalized_size = size / num_activities.max(1) as f64;

        (normalized_size - 1.0).max(0.0) / 5.0
    }

    fn calculate_cyclomatic_complexity(net: &PetriNet) -> f64 {
        let num_nodes = (net.places.len() + net.transitions.len()) as i32;
        let num_edges = net.arcs.len() as i32;

        let num_components = 1;

        let complexity = (num_edges - num_nodes + 2 * num_components).max(1) as f64;

        (complexity - 1.0).max(0.0) / 10.0
    }

    fn calculate_structuredness(net: &PetriNet) -> f64 {
        let mut score = 0.0;

        if net.is_workflow_net() {
            score += 0.3;
        }

        score += Self::check_branching_balance(net) * 0.2;
        score += Self::check_arc_weight_simplicity(net) * 0.2;

        let invisible_ratio =
            net.invisible_transitions().len() as f64 / net.transitions.len().max(1) as f64;
        score += (1.0 - invisible_ratio.min(1.0)) * 0.3;

        score.min(1.0)
    }

    fn check_branching_balance(net: &PetriNet) -> f64 {
        if net.transitions.is_empty() {
            return 1.0;
        }

        let mut in_degrees = HashMap::new();
        let mut out_degrees = HashMap::new();

        for arc in &net.arcs {
            *in_degrees.entry(&arc.to).or_insert(0) += 1;
            *out_degrees.entry(&arc.from).or_insert(0) += 1;
        }

        let mut imbalances = 0;
        let mut max_degree = 0;

        for degree in in_degrees.values().chain(out_degrees.values()) {
            max_degree = max_degree.max(*degree);
            if *degree > 2 {
                imbalances += 1;
            }
        }

        if max_degree <= 2 {
            1.0
        } else {
            1.0 / (1.0 + imbalances as f64 / 10.0)
        }
    }

    fn check_arc_weight_simplicity(net: &PetriNet) -> f64 {
        if net.arcs.is_empty() {
            return 1.0;
        }

        let complex_weights = net.arcs.iter().filter(|a| a.weight > 1).count();

        1.0 / (1.0 + complex_weights as f64 / net.arcs.len() as f64)
    }

    pub fn detailed_analysis(net: &PetriNet) -> ComplexityBreakdown {
        let num_places = net.places.len();
        let num_transitions = net.transitions.len();
        let num_arcs = net.arcs.len();
        let num_activities = net.visible_transitions().len();
        let num_invisible = net.invisible_transitions().len();
        let is_workflow_net = net.is_workflow_net();

        let cyclomatic = Self::calculate_cyclomatic_complexity(net);
        let structuredness = Self::calculate_structuredness(net);
        let simplicity = Self::calculate(net);

        ComplexityBreakdown {
            num_places,
            num_transitions,
            num_arcs,
            num_activities,
            num_invisible_transitions: num_invisible,
            is_workflow_net,
            cyclomatic_complexity: cyclomatic,
            structuredness_score: structuredness,
            simplicity_score: simplicity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComplexityBreakdown {
    pub num_places: usize,
    pub num_transitions: usize,
    pub num_arcs: usize,
    pub num_activities: usize,
    pub num_invisible_transitions: usize,
    pub is_workflow_net: bool,
    pub cyclomatic_complexity: f64,
    pub structuredness_score: f64,
    pub simplicity_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::petri_net::{Arc, Place, Transition};

    fn create_simple_net() -> PetriNet {
        let mut net = PetriNet::new();

        let p1 = Place::new("p1").with_initial_marking(1);
        let t1 = Transition::new("t1").with_label("a");
        let p2 = Place::new("p2");
        let t2 = Transition::new("t2").with_label("b");
        let p3 = Place::new("p3").with_final_marking(1);

        let ids = (
            p1.id.clone(),
            t1.id.clone(),
            p2.id.clone(),
            t2.id.clone(),
            p3.id.clone(),
        );

        net.add_place(p1);
        net.add_transition(t1);
        net.add_place(p2);
        net.add_transition(t2);
        net.add_place(p3);

        net.add_arc(Arc::new(&ids.0, &ids.1));
        net.add_arc(Arc::new(&ids.1, &ids.2));
        net.add_arc(Arc::new(&ids.2, &ids.3));
        net.add_arc(Arc::new(&ids.3, &ids.4));

        net.set_initial_place(ids.0);
        net.set_final_place(ids.4);

        net
    }

    #[test]
    fn test_simplicity_calculation() {
        let net = create_simple_net();
        let simplicity = Simplicity::calculate(&net);

        assert!(simplicity > 0.0 && simplicity <= 1.0);
    }

    #[test]
    fn test_empty_net_simplicity() {
        let net = PetriNet::new();
        let simplicity = Simplicity::calculate(&net);

        assert_eq!(simplicity, 1.0);
    }

    #[test]
    fn test_detailed_analysis() {
        let net = create_simple_net();
        let analysis = Simplicity::detailed_analysis(&net);

        assert!(analysis.num_places > 0);
        assert!(analysis.num_transitions > 0);
        assert!(analysis.is_workflow_net);
        assert!(analysis.simplicity_score > 0.0 && analysis.simplicity_score <= 1.0);
    }
}
