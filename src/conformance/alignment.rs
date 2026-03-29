use super::{ConformanceChecker, ConformanceResult};
/// Alignment-based Conformance Checking
use crate::log::EventLog;
use crate::models::PetriNet;
use crate::observability::Tracing;

#[derive(Debug, Clone)]
pub struct Alignment {
    pub trace_moves: Vec<String>,
    pub model_moves: Vec<String>,
    pub synchronous_moves: Vec<String>,
    pub cost: usize,
}

pub struct AlignmentChecker {
    pub max_cost: usize,
}

impl AlignmentChecker {
    pub fn new() -> Self {
        Self { max_cost: 100 }
    }

    pub fn with_max_cost(mut self, cost: usize) -> Self {
        self.max_cost = cost;
        self
    }

    pub fn check(&self, log: &EventLog, net: &PetriNet) -> ConformanceResult {
        use crate::conformance::alignments::conformance_alignments;
        let result = conformance_alignments(log, net);
        ConformanceResult {
            is_conformant: result.average_fitness >= 0.9,
            fitness: result.average_fitness,
            precision: 0.0,
            generalization: 0.0,
        }
    }
}

impl Default for AlignmentChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl ConformanceChecker for AlignmentChecker {
    fn check(&self, log: &EventLog, net: &PetriNet) -> ConformanceResult {
        self.check(log, net)
    }
}

/// Run alignment-based conformance checking and emit a `conformance.check` span
/// into `tracing` with fitness, trace_count, and algorithm attributes.
///
/// Armstrong rule: Tracing methods panic on lock poison — crash and let the
/// supervisor restart rather than hiding the failure with try/rescue.
///
/// Span attributes emitted:
/// - `conformance.algorithm` = `"alignment"`
/// - `conformance.trace_count` = number of traces in the log
/// - `conformance.fitness` = fitness score [0.0, 1.0]
///
/// Returns the `ConformanceResult` so the caller can act on the fitness score.
pub fn check_with_tracing(log: &EventLog, net: &PetriNet, tracing: &Tracing) -> ConformanceResult {
    use crate::semconv::conformance_attributes::CONFORMANCE_FITNESS;
    use crate::semconv::spans::CONFORMANCE_CHECK;

    let trace_count = log.traces.len();
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("conformance.algorithm".to_string(), "alignment".to_string());
    attrs.insert(
        "conformance.trace_count".to_string(),
        trace_count.to_string(),
    );

    let mut span = tracing
        .start_span(CONFORMANCE_CHECK, attrs, None)
        .expect("Tracing::start_span must not fail — if it does, crash and let supervisor restart");

    let result = AlignmentChecker::new().check(log, net);

    span.attributes
        .insert(CONFORMANCE_FITNESS.to_string(), result.fitness.to_string());

    tracing
        .end_span(&mut span, "ok", None)
        .expect("Tracing::end_span must not fail");

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::AlphaMiner;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_trace() -> Trace {
        let mut trace = Trace::new("case_1");
        let now = Utc::now();
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now));
        trace
    }

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        log.add_trace(create_test_trace());
        log
    }

    #[test]
    fn test_alignment_checker() {
        let log = create_test_log();
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        let checker = AlignmentChecker::new();
        let result = checker.check(&log, &net);

        assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
    }
}
