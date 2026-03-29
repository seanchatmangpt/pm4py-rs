/// DFG Miner - Directly-Follows Graph discovery
use crate::log::EventLog;
use crate::models::dfg::DirectlyFollowsGraph;
use crate::observability::{SpanContext, Tracing};
use crate::semconv::process_mining_attributes::{
    process_mining_algorithm, PROCESS_MINING_ALGORITHM, PROCESS_MINING_CASE_COUNT,
    PROCESS_MINING_DFG_EDGE_COUNT, PROCESS_MINING_DFG_NODE_COUNT,
};
use crate::semconv::process_mining_span_names::PROCESS_MINING_DFG_SPAN;
use std::collections::HashMap;

pub struct DFGMiner;

impl DFGMiner {
    pub fn new() -> Self {
        DFGMiner
    }

    pub fn discover(&self, log: &EventLog) -> DirectlyFollowsGraph {
        DirectlyFollowsGraph::from_log(log)
    }

    /// Discover a DFG and emit an OTEL span named `process.mining.dfg`.
    ///
    /// Returns `(DirectlyFollowsGraph, Vec<SpanContext>)`. Span carries:
    /// - `process.mining.algorithm` = `"directly_follows"`
    /// - `process.mining.case_count` = number of traces
    /// - `process.mining.dfg.node_count` = nodes in result DFG
    /// - `process.mining.dfg.edge_count` = edges in result DFG
    ///
    /// Armstrong rule: no try/rescue — crash propagates to supervisor.
    pub fn discover_with_tracing(
        &self,
        log: &EventLog,
    ) -> (DirectlyFollowsGraph, Vec<SpanContext>) {
        let tracing = Tracing::new();
        let case_count = log.traces.len();

        let mut attrs = HashMap::new();
        attrs.insert(
            PROCESS_MINING_ALGORITHM.to_string(),
            process_mining_algorithm::DIRECTLY_FOLLOWS.to_string(),
        );
        attrs.insert(
            PROCESS_MINING_CASE_COUNT.to_string(),
            case_count.to_string(),
        );

        let mut span = tracing
            .start_span(PROCESS_MINING_DFG_SPAN, attrs, None)
            .expect("Tracing::start_span must not fail");

        let dfg = self.discover(log);

        span.attributes.insert(
            PROCESS_MINING_DFG_NODE_COUNT.to_string(),
            dfg.nodes.len().to_string(),
        );
        span.attributes.insert(
            PROCESS_MINING_DFG_EDGE_COUNT.to_string(),
            dfg.edges.len().to_string(),
        );

        tracing
            .end_span(&mut span, "ok", None)
            .expect("Tracing::end_span must not fail");

        let spans = tracing.get_spans();
        (dfg, spans)
    }
}

impl Default for DFGMiner {
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
    fn test_dfg_miner() {
        let log = create_test_log();
        let miner = DFGMiner::new();
        let dfg = miner.discover(&log);

        assert!(!dfg.nodes.is_empty());
    }

    #[test]
    fn dfg_discovers_directly_follows_edges() {
        use crate::log::EventLog;

        let mut log = EventLog::new();
        let now = Utc::now();

        // Trace 1: a -> b -> c
        let mut t1 = Trace::new("case_1");
        t1.add_event(Event::new("a", now));
        t1.add_event(Event::new("b", now));
        t1.add_event(Event::new("c", now));
        log.add_trace(t1);

        // Trace 2: a -> b -> d
        let mut t2 = Trace::new("case_2");
        t2.add_event(Event::new("a", now));
        t2.add_event(Event::new("b", now));
        t2.add_event(Event::new("d", now));
        log.add_trace(t2);

        let dfg = DFGMiner::new().discover(&log);

        assert!(!dfg.edges.is_empty(), "DFG must have edges");
        assert!(dfg.nodes.contains(&"a".to_string()), "node 'a' must exist");
        assert!(dfg.nodes.contains(&"b".to_string()), "node 'b' must exist");

        // a->b should appear in both traces (frequency 2)
        let ab_edge = dfg.edges.iter().find(|e| e.from == "a" && e.to == "b");
        assert!(ab_edge.is_some(), "edge a->b must exist");
        assert_eq!(ab_edge.unwrap().frequency, 2, "a->b frequency must be 2");

        // b->c and b->d should each appear once
        assert!(
            dfg.edges.iter().any(|e| e.from == "b" && e.to == "c"),
            "edge b->c must exist"
        );
        assert!(
            dfg.edges.iter().any(|e| e.from == "b" && e.to == "d"),
            "edge b->d must exist"
        );
    }

    #[test]
    fn dfg_empty_log_produces_empty_graph() {
        use crate::log::EventLog;

        let log = EventLog::new();
        let dfg = DFGMiner::new().discover(&log);
        assert!(dfg.nodes.is_empty(), "empty log must produce no nodes");
        assert!(dfg.edges.is_empty(), "empty log must produce no edges");
    }
}
