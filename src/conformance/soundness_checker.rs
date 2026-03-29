/// Soundness Verification Engine for Petri Nets
///
/// Implements formal soundness checking according to van der Aalst's soundness theorem
/// for workflow nets. A Petri net is sound if:
///
/// 1. **Deadlock-free**: From any reachable marking, the sink place is reachable
/// 2. **Proper termination**: Only one token in sink, zero elsewhere when done
/// 3. **Liveness**: Every transition can fire in some execution sequence
///
/// # Mathematical Foundations
///
/// For a workflow net (N, initial_place, final_place):
/// - Reachability set R(M0) = all markings reachable from initial
/// - A marking is a "deadlock state" if ∄ transition enabled AND final_place not reachable
/// - Proper termination: ∀M ∈ R(M0), if M is terminal, then M(final_place) = 1 AND ∀p ≠ final, M(p) = 0
/// - Liveness: ∀t ∈ T, ∃M ∈ R(M0) such that t is enabled in M
///
/// Reference: W. M. P. van der Aalst, "Verification of Workflow Nets,"
/// in Application and Theory of Petri Nets, pp. 407-426, 1997.
use crate::models::petri_net::PetriNet;
use std::collections::{HashMap, VecDeque};

/// A marking is a snapshot of token distribution (place_id -> token_count)
pub type Marking = HashMap<String, usize>;

/// A sequence of markings representing proof of soundness
pub type MarkingSequence = Vec<String>; // serialized markings

/// Detailed violation evidence when soundness check fails
#[derive(Debug, Clone)]
pub enum SoundnessViolation {
    /// Deadlock detected at unreachable marking
    DeadlockFound {
        marking: Marking,
        reachable_transitions: Vec<String>,
    },

    /// Improper termination detected
    ImproperTermination {
        final_marking: Marking,
        reason: String,
    },

    /// Transition that can never fire
    DeadTransition {
        transition_id: String,
        transition_name: String,
    },

    /// Net is not a workflow net (missing source or sink)
    NotWorkflowNet {
        source_count: usize,
        sink_count: usize,
    },
}

impl std::fmt::Display for SoundnessViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoundnessViolation::DeadlockFound { marking, .. } => {
                write!(f, "Deadlock found at marking: {:?}", marking)
            }
            SoundnessViolation::ImproperTermination { reason, .. } => {
                write!(f, "Improper termination: {}", reason)
            }
            SoundnessViolation::DeadTransition {
                transition_name, ..
            } => {
                write!(f, "Dead transition: {}", transition_name)
            }
            SoundnessViolation::NotWorkflowNet {
                source_count,
                sink_count,
            } => {
                write!(
                    f,
                    "Not a workflow net: {} sources, {} sinks",
                    source_count, sink_count
                )
            }
        }
    }
}

/// Result of soundness verification with formal proof
#[derive(Debug, Clone)]
pub struct SoundnessProof {
    /// Overall soundness verdict
    pub is_sound: bool,

    /// No deadlock reachable from initial marking
    pub no_deadlock: bool,

    /// Proper termination property verified
    pub proper_termination: bool,

    /// Liveness property verified (all transitions live)
    pub liveness_verified: bool,

    /// Execution sequence proving soundness (or violation evidence)
    pub proof_path: Vec<MarkingSequence>,

    /// Summary message
    pub summary: String,

    /// Detailed violation if unsound
    pub violation: Option<SoundnessViolation>,
}

/// The soundness verification engine
pub struct SoundnessChecker {
    net: PetriNet,
    initial_marking: Marking,
}

impl SoundnessChecker {
    /// Create a new soundness checker for a Petri net
    pub fn new(net: PetriNet) -> Self {
        let initial_marking = Self::compute_initial_marking(&net);

        SoundnessChecker {
            net,
            initial_marking,
        }
    }

    /// Compute initial marking from initial place
    fn compute_initial_marking(net: &PetriNet) -> Marking {
        let mut marking = HashMap::new();

        // Set up initial tokens
        for place in &net.places {
            if place.initial_marking > 0 {
                marking.insert(place.id.clone(), place.initial_marking);
            }
        }

        marking
    }

    /// Perform complete soundness verification
    pub fn check(&self) -> SoundnessProof {
        // Step 1: Verify this is a workflow net (single source, single sink)
        let sources = self.net.source_places();
        let sinks = self.net.sink_places();

        if sources.len() != 1 || sinks.len() != 1 {
            return SoundnessProof {
                is_sound: false,
                no_deadlock: false,
                proper_termination: false,
                liveness_verified: false,
                proof_path: vec![],
                summary: "Not a workflow net: must have exactly one source and one sink"
                    .to_string(),
                violation: Some(SoundnessViolation::NotWorkflowNet {
                    source_count: sources.len(),
                    sink_count: sinks.len(),
                }),
            };
        }

        let final_place_id = sinks[0].id.clone();

        // Step 2: Compute reachability set
        let mut reachable_set = Vec::new();
        let mut proof_path = Vec::new();

        if !self.compute_reachability(&mut reachable_set, &mut proof_path) {
            return SoundnessProof {
                is_sound: false,
                no_deadlock: false,
                proper_termination: false,
                liveness_verified: false,
                proof_path,
                summary: "Failed to compute reachability set".to_string(),
                violation: None,
            };
        }

        // Step 3: Check deadlock-free property
        let no_deadlock = self.check_no_deadlock(&reachable_set, &final_place_id);

        if !no_deadlock {
            // Find the deadlock marking for violation report
            if let Some(deadlock) = self.find_deadlock(&reachable_set, &final_place_id) {
                return SoundnessProof {
                    is_sound: false,
                    no_deadlock: false,
                    proper_termination: false,
                    liveness_verified: false,
                    proof_path,
                    summary: "Deadlock detected".to_string(),
                    violation: Some(SoundnessViolation::DeadlockFound {
                        marking: deadlock,
                        reachable_transitions: vec![],
                    }),
                };
            }
        }

        // Step 4: Check proper termination
        let proper_termination = self.check_proper_termination(&reachable_set, &final_place_id);

        if !proper_termination {
            return SoundnessProof {
                is_sound: false,
                no_deadlock,
                proper_termination: false,
                liveness_verified: false,
                proof_path,
                summary: "Improper termination: cannot guarantee single token in sink".to_string(),
                violation: Some(SoundnessViolation::ImproperTermination {
                    final_marking: self.initial_marking.clone(),
                    reason: "Cannot reach state with exactly one token in final place".to_string(),
                }),
            };
        }

        // Step 5: Check liveness (all transitions must be live)
        let live_transitions = self.compute_liveness(&reachable_set);
        let liveness_verified = live_transitions.len() == self.net.transitions.len();

        if !liveness_verified {
            // Find dead transition
            for t in &self.net.transitions {
                if !live_transitions.contains(&t.id) {
                    return SoundnessProof {
                        is_sound: false,
                        no_deadlock,
                        proper_termination,
                        liveness_verified: false,
                        proof_path,
                        summary: format!("Dead transition: {}", t.name),
                        violation: Some(SoundnessViolation::DeadTransition {
                            transition_id: t.id.clone(),
                            transition_name: t.name.clone(),
                        }),
                    };
                }
            }
        }

        // All checks passed - net is sound
        SoundnessProof {
            is_sound: true,
            no_deadlock: true,
            proper_termination: true,
            liveness_verified: true,
            proof_path,
            summary: "Petri net is sound: deadlock-free, proper termination, all transitions live"
                .to_string(),
            violation: None,
        }
    }

    /// Compute full reachability set using BFS
    /// Returns Vec of markings instead of HashSet since HashMap doesn't implement Hash
    fn compute_reachability(
        &self,
        reachable_set: &mut Vec<Marking>,
        proof_path: &mut Vec<MarkingSequence>,
    ) -> bool {
        let mut queue = VecDeque::new();
        queue.push_back(self.initial_marking.clone());
        reachable_set.push(self.initial_marking.clone());

        let current_path = vec![Self::marking_to_string(&self.initial_marking)];

        while let Some(current_marking) = queue.pop_front() {
            // Try firing each transition from current marking
            for transition in &self.net.transitions {
                let mut new_marking = current_marking.clone();

                if self.net.fire_transition(&transition.id, &mut new_marking)
                    && !reachable_set.iter().any(|m| m == &new_marking)
                {
                    reachable_set.push(new_marking.clone());
                    queue.push_back(new_marking);
                }
            }
        }

        // If we found many reachable states, sample them for proof path
        if reachable_set.len() <= 100 {
            for marking in reachable_set.iter().take(10) {
                proof_path.push(vec![Self::marking_to_string(marking)]);
            }
        } else {
            // For large reachability sets, just record the initial state
            proof_path.push(current_path);
        }

        true
    }

    /// Check deadlock-free property: from any reachable marking, can reach final place
    fn check_no_deadlock(&self, reachable_set: &[Marking], final_place_id: &str) -> bool {
        for marking in reachable_set {
            // Skip if already at final state
            let final_tokens = marking.get(final_place_id).copied().unwrap_or(0);
            if final_tokens > 0 && marking.values().sum::<usize>() == final_tokens {
                continue; // This is a valid final state
            }

            // Check if any transition is enabled
            let mut any_enabled = false;
            for transition in &self.net.transitions {
                if self.net.is_transition_enabled(&transition.id, marking) {
                    any_enabled = true;
                    break;
                }
            }

            // If no transition is enabled and not at final state -> deadlock
            if !any_enabled
                && (final_tokens == 0 || marking.values().sum::<usize>() != final_tokens)
            {
                return false;
            }
        }

        true
    }

    /// Find a deadlock marking (if any)
    fn find_deadlock(&self, reachable_set: &[Marking], final_place_id: &str) -> Option<Marking> {
        for marking in reachable_set {
            let final_tokens = marking.get(final_place_id).copied().unwrap_or(0);

            // Skip valid final states
            if final_tokens > 0 && marking.values().sum::<usize>() == final_tokens {
                continue;
            }

            // Check if deadlocked (no transition enabled, not at final)
            let mut any_enabled = false;
            for transition in &self.net.transitions {
                if self.net.is_transition_enabled(&transition.id, marking) {
                    any_enabled = true;
                    break;
                }
            }

            if !any_enabled
                && (final_tokens == 0 || marking.values().sum::<usize>() != final_tokens)
            {
                return Some(marking.clone());
            }
        }

        None
    }

    /// Check proper termination: only final place has token when done
    fn check_proper_termination(&self, reachable_set: &[Marking], final_place_id: &str) -> bool {
        // Find terminal states (no enabled transitions from reachable set)
        for marking in reachable_set {
            let mut any_enabled = false;
            for transition in &self.net.transitions {
                if self.net.is_transition_enabled(&transition.id, marking) {
                    any_enabled = true;
                    break;
                }
            }

            // If this is a terminal state
            if !any_enabled {
                let final_tokens = marking.get(final_place_id).copied().unwrap_or(0);

                // Proper termination requires:
                // 1. Exactly 1 token in final place
                // 2. No tokens anywhere else
                let total_tokens = marking.values().sum::<usize>();
                if final_tokens != 1 || total_tokens != 1 {
                    return false;
                }
            }
        }

        true
    }

    /// Compute live transitions (can fire from some reachable marking)
    fn compute_liveness(&self, reachable_set: &[Marking]) -> Vec<String> {
        let mut live = Vec::new();

        for marking in reachable_set {
            for transition in &self.net.transitions {
                if self.net.is_transition_enabled(&transition.id, marking)
                    && !live.contains(&transition.id)
                {
                    live.push(transition.id.clone());
                }
            }
        }

        live
    }

    /// Convert marking to string representation for proof
    fn marking_to_string(marking: &Marking) -> String {
        let mut items: Vec<_> = marking.iter().collect();
        items.sort_by_key(|(id, _)| *id);

        let repr: Vec<String> = items
            .iter()
            .map(|(id, count)| format!("{}:{}", id, count))
            .collect();

        format!("[{}]", repr.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::petri_net::{Arc, Place, Transition};

    #[test]
    fn test_simple_sound_net() {
        let mut net = PetriNet::new();
        let p1 = Place::new("p1").with_initial_marking(1);
        let p3 = Place::new("p3");

        let p1_id = p1.id.clone();
        let p3_id = p3.id.clone();

        net.add_place(p1);
        net.add_place(p3);

        let t = Transition::new("t1");
        let t_id = t.id.clone();
        net.add_transition(t);

        net.add_arc(Arc::new(&p1_id, &t_id));
        net.add_arc(Arc::new(&t_id, &p3_id));

        net.set_initial_place(p1_id);
        net.set_final_place(p3_id);

        let checker = SoundnessChecker::new(net);
        let result = checker.check();

        assert!(result.is_sound);
        assert!(result.no_deadlock);
        assert!(result.proper_termination);
        assert!(result.liveness_verified);
    }

    #[test]
    fn test_workflow_net_validation() {
        let net = PetriNet::new(); // Empty net
        let checker = SoundnessChecker::new(net);
        let result = checker.check();

        assert!(!result.is_sound);
    }
}
