/// Inductive Miner - A recursive process discovery algorithm
///
/// pm4py is the **only** implementation.  If Python or pm4py are unavailable
/// the process must crash loudly — never swallow the error and return a stub.
use crate::log::EventLog;
use crate::models::PetriNet;
use crate::observability::{SpanContext, Tracing};
use crate::semconv::process_mining_attributes::{
    process_mining_algorithm, PROCESS_MINING_ALGORITHM, PROCESS_MINING_CASE_COUNT,
};
use crate::semconv::process_mining_span_names::PROCESS_MINING_INDUCTIVE_MINE_SPAN;
use std::collections::HashMap;

pub struct InductiveMiner {
    pub min_support: f64,
}

impl InductiveMiner {
    pub fn new() -> Self {
        Self { min_support: 0.0 }
    }

    pub fn with_min_support(mut self, support: f64) -> Self {
        self.min_support = support;
        self
    }

    /// Discover a Petri net via `pm4py.discover_petri_net_inductive`.
    ///
    /// # Panics
    ///
    /// Panics if Python or pm4py are unavailable.  This is intentional: pm4py
    /// is the only implementation and there is no safe fallback.
    pub fn discover(&self, log: &EventLog) -> PetriNet {
        use crate::python::generated::discovery::call_discover_petri_net_inductive;
        use pyo3::Python;
        Python::with_gil(|py| call_discover_petri_net_inductive(py, log)).expect(
            "pm4py not available — ensure Python and pm4py are installed (pip install pm4py)",
        )
    }

    /// Discover a Petri net and emit an OTEL span named `process.mining.inductive.mine`.
    ///
    /// Returns `(PetriNet, Vec<SpanContext>)`. Span carries:
    /// - `process.mining.algorithm` = `"inductive_miner"`
    /// - `process.mining.case_count` = number of traces
    ///
    /// Armstrong rule: no try/rescue — crash propagates to supervisor.
    pub fn discover_with_tracing(&self, log: &EventLog) -> (PetriNet, Vec<SpanContext>) {
        let tracing = Tracing::new();
        let case_count = log.traces.len();

        let mut attrs = HashMap::new();
        attrs.insert(
            PROCESS_MINING_ALGORITHM.to_string(),
            process_mining_algorithm::INDUCTIVE_MINER.to_string(),
        );
        attrs.insert(
            PROCESS_MINING_CASE_COUNT.to_string(),
            case_count.to_string(),
        );

        let mut span = tracing
            .start_span(PROCESS_MINING_INDUCTIVE_MINE_SPAN, attrs, None)
            .expect("Tracing::start_span must not fail");

        let net = self.discover(log);

        tracing
            .end_span(&mut span, "ok", None)
            .expect("Tracing::end_span must not fail");

        let spans = tracing.get_spans();
        (net, spans)
    }

    /// Discover a process tree from the event log via pm4py inductive miner.
    ///
    /// Delegates to `call_discover_petri_net_inductive` and reconstructs
    /// the tree from the discovered net's visible transitions.
    ///
    /// # Panics
    ///
    /// Panics if Python or pm4py are unavailable.
    pub fn discover_tree(&self, log: &EventLog) -> crate::models::ProcessTree {
        use crate::python::generated::discovery::call_discover_petri_net_inductive;
        use pyo3::Python;

        let net = Python::with_gil(|py| call_discover_petri_net_inductive(py, log)).expect(
            "pm4py not available — ensure Python and pm4py are installed (pip install pm4py)",
        );

        let mut activities: Vec<String> = net
            .transitions
            .iter()
            .filter_map(|t| t.label.clone())
            .collect();
        activities.sort();
        activities.dedup();

        match activities.len() {
            0 => crate::models::ProcessTree::new(crate::models::ProcessTreeNode::activity("SKIP")),
            1 => crate::models::ProcessTree::new(crate::models::ProcessTreeNode::activity(
                &activities[0],
            )),
            _ => crate::models::ProcessTree::new(crate::models::ProcessTreeNode::sequence(
                activities
                    .iter()
                    .map(|a| crate::models::ProcessTreeNode::activity(a.as_str()))
                    .collect(),
            )),
        }
    }
}

impl Default for InductiveMiner {
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

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now));
        log.add_trace(trace);

        log
    }

    #[test]
    fn test_inductive_miner() {
        let log = create_test_log();
        let miner = InductiveMiner::new();
        let net = miner.discover(&log);

        assert!(!net.transitions.is_empty());
        assert!(!net.places.is_empty());
    }
}
