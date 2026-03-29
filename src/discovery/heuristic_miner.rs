/// Heuristic Miner - A discovery algorithm based on heuristics
///
/// pm4py is the **only** implementation.  If Python or pm4py are unavailable
/// the process must crash loudly — never swallow the error and return a stub.
use crate::log::EventLog;
use crate::models::PetriNet;

pub struct HeuristicMiner {
    pub dependency_threshold: f64,
    pub loop_threshold: f64,
}

impl HeuristicMiner {
    pub fn new() -> Self {
        Self {
            dependency_threshold: 0.5,
            loop_threshold: 0.5,
        }
    }

    pub fn with_dependency_threshold(mut self, threshold: f64) -> Self {
        self.dependency_threshold = threshold;
        self
    }

    pub fn with_loop_threshold(mut self, threshold: f64) -> Self {
        self.loop_threshold = threshold;
        self
    }

    /// Discover a Petri net via `pm4py.discover_petri_net_heuristics`.
    ///
    /// # Panics
    ///
    /// Panics if Python or pm4py are unavailable.  This is intentional: pm4py
    /// is the only implementation and there is no safe fallback.
    pub fn discover(&self, log: &EventLog) -> PetriNet {
        use crate::python::generated::discovery::call_discover_petri_net_heuristics;
        use pyo3::Python;
        Python::with_gil(|py| call_discover_petri_net_heuristics(py, log)).expect(
            "pm4py not available — ensure Python and pm4py are installed (pip install pm4py)",
        )
    }
}

impl Default for HeuristicMiner {
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
    fn test_heuristic_miner() {
        let log = create_test_log();
        let miner = HeuristicMiner::new();
        let net = miner.discover(&log);

        assert!(!net.transitions.is_empty());
    }
}
