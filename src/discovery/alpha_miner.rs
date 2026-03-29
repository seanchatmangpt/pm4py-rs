/// Alpha Miner - A fundamental process discovery algorithm
///
/// The Alpha Miner discovers a Petri net from an event log by analyzing
/// causal relations between activities.
use crate::log::{operations, EventLog};
use crate::models::petri_net::{Arc, Place, Transition};
use crate::models::PetriNet;
use crate::observability::{SpanContext, Tracing};
use crate::semconv::process_mining_attributes::{
    process_mining_algorithm, PROCESS_MINING_ALGORITHM, PROCESS_MINING_CASE_COUNT,
    PROCESS_MINING_PETRI_NET_PLACE_COUNT, PROCESS_MINING_PETRI_NET_TRANSITION_COUNT,
};
use crate::semconv::process_mining_span_names::PROCESS_MINING_DISCOVERY_SPAN;
use std::collections::HashMap;

pub struct AlphaMiner {
    pub noise_threshold: f64,
}

impl AlphaMiner {
    pub fn new() -> Self {
        Self {
            noise_threshold: 0.0,
        }
    }

    pub fn with_noise_threshold(mut self, threshold: f64) -> Self {
        self.noise_threshold = threshold;
        self
    }

    pub fn discover(&self, log: &EventLog) -> PetriNet {
        let mut net = PetriNet::new();

        // Get basic statistics
        let activities = log.activities();
        let start_acts = operations::start_activities(log);
        let end_acts = operations::end_activities(log);
        let df = operations::directly_follows(log);

        // Create transitions for all activities
        let mut trans_map: HashMap<String, String> = HashMap::new();
        for activity in &activities {
            let transition = Transition::new(activity).with_label(activity);
            let trans_id = transition.id.clone();
            trans_map.insert(activity.clone(), trans_id);
            net.add_transition(transition);
        }

        // ## Causal Dependency Detection
        // Identify causality relations: a →→ b only if a→b exists AND b↛a.
        // This asymmetry is essential: if both a→b and b→a exist, they're part of a loop,
        // not a causal sequence. The Alpha algorithm requires strictly one-directional deps.
        let mut causality = HashMap::new();
        for activity_a in &activities {
            for activity_b in &activities {
                if activity_a == activity_b {
                    continue;
                }

                let ab = df
                    .get(&(activity_a.clone(), activity_b.clone()))
                    .copied()
                    .unwrap_or(0);
                let ba = df
                    .get(&(activity_b.clone(), activity_a.clone()))
                    .copied()
                    .unwrap_or(0);

                // Causal relation: a -> b if a -> b exists and b -> a doesn't
                if ab > 0 && ba == 0 {
                    causality.insert((activity_a.clone(), activity_b.clone()), true);
                }
            }
        }

        // Create places for transitions
        let mut place_count = 0;

        // Initial place
        let initial_place = Place::new("source").with_initial_marking(1);
        let initial_place_id = initial_place.id.clone();
        net.add_place(initial_place);
        net.set_initial_place(initial_place_id.clone());

        // Connect initial place to start activities
        for activity in start_acts.keys() {
            if let Some(trans_id) = trans_map.get(activity) {
                net.add_arc(Arc::new(&initial_place_id, trans_id));
            }
        }

        // Final place
        let final_place = Place::new("sink");
        let final_place_id = final_place.id.clone();
        net.add_place(final_place);
        net.set_final_place(final_place_id.clone());

        // ## Place Creation for Causal Dependencies
        // For each causal relation a→b, create an implicit place (intermediate state).
        // In Petri nets, places model conditions; transitions model activities.
        // The pattern from_trans → place → to_trans enforces sequential ordering:
        // only when 'from' completes (token in place) can 'to' fire.
        for ((from, to), _) in causality {
            if let (Some(from_trans), Some(to_trans)) = (trans_map.get(&from), trans_map.get(&to)) {
                let place = Place::new(format!("p_{}_{}", place_count, place_count + 1));
                let place_id = place.id.clone();
                net.add_place(place);

                net.add_arc(Arc::new(from_trans, &place_id));
                net.add_arc(Arc::new(&place_id, to_trans));

                place_count += 1;
            }
        }

        // Connect end activities to final place
        for activity in end_acts.keys() {
            if let Some(trans_id) = trans_map.get(activity) {
                net.add_arc(Arc::new(trans_id, &final_place_id));
            }
        }

        net
    }

    /// Discover a Petri net and emit an OTEL span capturing discovery metrics.
    ///
    /// Returns `(PetriNet, Vec<SpanContext>)`. The span is named
    /// `process.mining.discovery` and carries:
    /// - `process.mining.algorithm` = `"alpha_miner"`
    /// - `process.mining.case_count` = number of traces
    /// - `process.mining.petri_net.place_count` = places in result
    /// - `process.mining.petri_net.transition_count` = transitions in result
    ///
    /// Armstrong rule: Tracing::new() never panics. If it did, let it crash —
    /// the caller's supervisor restarts; no try/rescue.
    pub fn discover_with_tracing(&self, log: &EventLog) -> (PetriNet, Vec<SpanContext>) {
        let tracing = Tracing::new();
        let case_count = log.traces.len();

        let mut attrs = HashMap::new();
        attrs.insert(
            PROCESS_MINING_ALGORITHM.to_string(),
            process_mining_algorithm::ALPHA_MINER.to_string(),
        );
        attrs.insert(
            PROCESS_MINING_CASE_COUNT.to_string(),
            case_count.to_string(),
        );

        let mut span = tracing
            .start_span(PROCESS_MINING_DISCOVERY_SPAN, attrs, None)
            .expect(
                "Tracing::start_span must not fail — if it does, crash and let supervisor restart",
            );

        let net = self.discover(log);

        span.attributes.insert(
            PROCESS_MINING_PETRI_NET_PLACE_COUNT.to_string(),
            net.places.len().to_string(),
        );
        span.attributes.insert(
            PROCESS_MINING_PETRI_NET_TRANSITION_COUNT.to_string(),
            net.transitions.len().to_string(),
        );

        tracing
            .end_span(&mut span, "ok", None)
            .expect("Tracing::end_span must not fail");

        let spans = tracing.get_spans();
        (net, spans)
    }
}

impl Default for AlphaMiner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        for case_id in 1..=3 {
            let mut trace = Trace::new(format!("case_{}", case_id));
            trace.add_event(Event::new("a", now));
            trace.add_event(Event::new("b", now));
            trace.add_event(Event::new("c", now));
            log.add_trace(trace);
        }

        log
    }

    #[test]
    fn test_alpha_miner_discovery() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        assert!(!net.transitions.is_empty());
        assert!(!net.places.is_empty());
    }

    #[test]
    fn test_alpha_miner_creates_valid_net() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        // Check that net has initial and final places
        assert!(net.initial_place.is_some());
        assert!(net.final_place.is_some());

        // Check that we have at least 3 transitions (a, b, c)
        assert!(net.transitions.len() >= 3);
    }

    #[test]
    fn test_alpha_miner_linear_sequence_5_activities() {
        // Test with 5-activity linear sequence: wake -> check_tasks -> execute -> delegate -> sleep
        let mut log = EventLog::new();
        let now = Utc::now();

        for case_id in 1..=3 {
            let mut trace = Trace::new(format!("case_{}", case_id));
            trace.add_event(Event::new("wake", now));
            trace.add_event(Event::new("check_tasks", now));
            trace.add_event(Event::new("execute", now));
            trace.add_event(Event::new("delegate", now));
            trace.add_event(Event::new("sleep", now));
            log.add_trace(trace);
        }

        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        // For N=5 activities in linear sequence:
        // - Should have exactly 5 transitions (one per activity)
        // - Should have 6 places (1 source + 4 intermediate + 1 sink) = N+1
        // - Should have 10 arcs (5 from transitions to places + 5 from places to transitions)

        println!("=== LINEAR SEQUENCE TEST (5 ACTIVITIES) ===");
        println!("Transitions: {} (expected 5)", net.transitions.len());
        println!("Places: {} (expected 6, N+1)", net.places.len());
        println!("Arcs: {} (expected 10)", net.arcs.len());

        // Print detailed structure
        println!("\nTransitions:");
        for t in &net.transitions {
            let incoming = net.get_arcs_to(&t.id).len();
            let outgoing = net.get_arcs_from(&t.id).len();
            println!(
                "  {} (label: {:?}): {} incoming, {} outgoing",
                t.id, t.label, incoming, outgoing
            );
        }

        println!("\nPlaces:");
        for p in &net.places {
            let incoming = net.get_arcs_to(&p.id).len();
            let outgoing = net.get_arcs_from(&p.id).len();
            println!(
                "  {} ({}): {} incoming, {} outgoing",
                p.id, p.name, incoming, outgoing
            );
        }

        println!("\nArcs: {}", net.arcs.len());
        for arc in &net.arcs {
            println!("  {} -> {}", arc.from, arc.to);
        }

        assert_eq!(
            net.transitions.len(),
            5,
            "Should have exactly 5 transitions"
        );
        assert_eq!(
            net.places.len(),
            6,
            "Should have exactly 6 places (1 source + 4 intermediate + 1 sink, N+1)"
        );
        assert_eq!(net.arcs.len(), 10, "Should have exactly 10 arcs");

        // Check that every transition has both incoming and outgoing arcs
        for transition in &net.transitions {
            let incoming = net.get_arcs_to(&transition.id);
            let outgoing = net.get_arcs_from(&transition.id);
            assert!(
                !incoming.is_empty(),
                "Transition {} missing incoming arc",
                transition.label.as_ref().unwrap_or(&transition.id)
            );
            assert!(
                !outgoing.is_empty(),
                "Transition {} missing outgoing arc",
                transition.label.as_ref().unwrap_or(&transition.id)
            );
        }

        // Verify workflow net property
        assert!(net.is_workflow_net(), "Should be a valid workflow net");
    }
}
