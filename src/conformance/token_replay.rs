use super::{ConformanceChecker, ConformanceResult};
/// Token Replay - A conformance checking algorithm
use crate::log::EventLog;
use crate::models::PetriNet;
use crate::observability::Tracing;
use std::collections::HashMap;

pub struct TokenReplay;

impl TokenReplay {
    pub fn new() -> Self {
        TokenReplay
    }

    pub fn check(&self, log: &EventLog, net: &PetriNet) -> ConformanceResult {
        let mut total_produced = 0.0;
        let mut total_missing = 0.0;
        let mut total_remaining = 0.0;

        for trace in &log.traces {
            let mut marking: HashMap<String, usize> = HashMap::new();

            // Initialize marking with initial places
            // Count initial tokens as "produced"
            let mut trace_produced = 0.0;
            if let Some(initial_id) = &net.initial_place {
                marking.insert(initial_id.clone(), 1);
                trace_produced = 1.0;
            }

            let mut trace_missing = 0.0;

            // Replay events using WvdA's token-based fitness.
            //
            // Standard WvdA algorithm (van der Aalst 2012):
            // - If a transition's input place lacks tokens, inject an artificial
            //   token (count it in both `m` and `p`), then fire.
            // - Count every output token produced by a fire in `p`.
            // This gives partial credit for non-conformant traces and correct
            // fitness = 1.0 for fully-conformant traces.
            for event in &trace.events {
                // Find transition with matching label
                let matching_trans = net.transitions.iter().find(|t| {
                    t.label
                        .as_ref()
                        .map(|l| l == &event.activity)
                        .unwrap_or(false)
                });

                if let Some(transition) = matching_trans {
                    // Inject artificial tokens for any unenabled input place
                    for arc in net.get_arcs_to(&transition.id) {
                        let available = marking.get(&arc.from).copied().unwrap_or(0);
                        if available < arc.weight {
                            let needed = (arc.weight - available) as f64;
                            trace_missing += needed;
                            trace_produced += needed; // artificial tokens count as produced
                            *marking.entry(arc.from.clone()).or_insert(0) += arc.weight - available;
                        }
                    }

                    // Fire the transition (now guaranteed to succeed after injection)
                    net.fire_transition(&transition.id, &mut marking);

                    // Count output tokens produced by this firing
                    trace_produced += net.get_arcs_from(&transition.id).len() as f64;
                }
                // No matching transition: skip (not counted as missing per WvdA)
            }

            // Count remaining tokens (tokens not reaching final place)
            let mut trace_remaining = 0.0;
            for (place_id, token_count) in &marking {
                // Only count tokens not in final place
                if let Some(final_id) = &net.final_place {
                    if place_id != final_id {
                        trace_remaining += *token_count as f64;
                    }
                } else {
                    // No final place defined, count all remaining
                    trace_remaining += *token_count as f64;
                }
            }

            // Accumulate for overall fitness (WvdA formula)
            // fitness = (produced - remaining - missing) / produced
            total_produced += trace_produced;
            total_missing += trace_missing;
            total_remaining += trace_remaining;
        }

        // Calculate fitness using WvdA's formula
        // fitness = (produced - remaining - missing) / produced
        let fitness = if total_produced > 0.0 {
            ((total_produced - total_remaining - total_missing) / total_produced).max(0.0)
        } else {
            0.0
        };

        ConformanceResult {
            is_conformant: fitness == 1.0,
            fitness,
            precision: 0.0,
            generalization: 0.0,
        }
    }
}

impl Default for TokenReplay {
    fn default() -> Self {
        Self::new()
    }
}

impl ConformanceChecker for TokenReplay {
    fn check(&self, log: &EventLog, net: &PetriNet) -> ConformanceResult {
        self.check(log, net)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::AlphaMiner;
    use crate::log::{Event, Trace};
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
    fn test_token_replay() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        let checker = TokenReplay::new();
        let result = checker.check(&log, &net);

        assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
    }
}

// ========================================================================
// MISSING WRAPPER FUNCTIONS FOR PYTHON PM4PY PARITY
// ========================================================================

/// Calculate precision via token replay using pm4py.
///
/// Calls `pm4py.precision_token_based_replay` through the auto-generated PyO3 bridge.
///
/// # Panics
///
/// Panics if Python or pm4py are unavailable.  This is intentional: pm4py is
/// the only implementation and there is no safe fallback.
pub fn precision_token_based_replay(log: &EventLog, net: &PetriNet) -> f64 {
    use crate::python::generated::conformance::call_precision_token_based_replay;
    use pyo3::Python;

    Python::with_gil(|py| call_precision_token_based_replay(py, log, net))
        .expect("pm4py not available — ensure Python and pm4py are installed (pip install pm4py)")
}

/// Get diagnostics from token-based replay, with precision from pm4py.
pub fn diagnostics_token_based_replay(log: &EventLog, net: &PetriNet) -> super::ConformanceResult {
    use super::ConformanceResult;
    use crate::python::generated::conformance::call_precision_token_based_replay;
    use pyo3::Python;
    let base = TokenReplay::new().check(log, net);
    let precision = Python::with_gil(|py| call_precision_token_based_replay(py, log, net))
        .expect("pm4py not available — ensure Python and pm4py are installed (pip install pm4py)");
    ConformanceResult { precision, ..base }
}

/// Run token-replay conformance checking and emit a `conformance.check` span
/// into `tracing` with fitness, trace_count, and algorithm attributes.
///
/// Armstrong rule: `Tracing` methods use expect/panic on lock poison — the
/// correct response to a poisoned mutex is to crash and let the supervisor
/// restart. No try/rescue here.
///
/// Span attributes emitted:
/// - `conformance.algorithm` = `"token_replay"`
/// - `conformance.trace_count` = number of traces in the log
/// - `conformance.fitness` = fitness score [0.0, 1.0]
///
/// Returns the `ConformanceResult` so the caller can act on the fitness score.
pub fn check_with_tracing(log: &EventLog, net: &PetriNet, tracing: &Tracing) -> ConformanceResult {
    use crate::semconv::conformance_attributes::CONFORMANCE_FITNESS;
    use crate::semconv::spans::CONFORMANCE_CHECK;

    let trace_count = log.traces.len();
    let mut attrs = std::collections::HashMap::new();
    attrs.insert(
        "conformance.algorithm".to_string(),
        "token_replay".to_string(),
    );
    attrs.insert(
        "conformance.trace_count".to_string(),
        trace_count.to_string(),
    );

    let mut span = tracing
        .start_span(CONFORMANCE_CHECK, attrs, None)
        .expect("Tracing::start_span must not fail — if it does, crash and let supervisor restart");

    let result = TokenReplay::new().check(log, net);

    span.attributes
        .insert(CONFORMANCE_FITNESS.to_string(), result.fitness.to_string());

    tracing
        .end_span(&mut span, "ok", None)
        .expect("Tracing::end_span must not fail");

    result
}
