/// ILP Miner - Integer Linear Programming based Petri Net Discovery
///
/// pm4py is the **only** implementation.  If Python or pm4py are unavailable
/// the process must crash loudly — never swallow the error and return a stub.
use crate::log::EventLog;
use crate::models::PetriNet;

/// ILP Miner configuration
#[derive(Debug, Clone)]
pub struct ILPMinerConfig {
    /// Use decomposition for large logs
    pub use_decomposition: bool,
    /// Minimum fraction of traces to cover
    pub min_trace_coverage: f64,
    /// Maximum number of places in result
    pub max_places: usize,
}

impl Default for ILPMinerConfig {
    fn default() -> Self {
        Self {
            use_decomposition: false,
            min_trace_coverage: 0.95,
            max_places: 100,
        }
    }
}

pub struct ILPMiner {
    pub config: ILPMinerConfig,
}

impl ILPMiner {
    pub fn new() -> Self {
        Self {
            config: ILPMinerConfig::default(),
        }
    }

    pub fn with_config(config: ILPMinerConfig) -> Self {
        Self { config }
    }

    pub fn with_decomposition(mut self, use_it: bool) -> Self {
        self.config.use_decomposition = use_it;
        self
    }

    pub fn with_min_coverage(mut self, coverage: f64) -> Self {
        self.config.min_trace_coverage = coverage.clamp(0.0, 1.0);
        self
    }

    /// Discover a Petri net via `pm4py.discover_petri_net_ilp`.
    ///
    /// # Panics
    ///
    /// Panics if Python or pm4py are unavailable.  This is intentional: pm4py
    /// is the only implementation and there is no safe fallback.
    pub fn discover(&self, log: &EventLog) -> PetriNet {
        use crate::python::generated::discovery::call_discover_petri_net_ilp;
        use pyo3::Python;
        Python::with_gil(|py| call_discover_petri_net_ilp(py, log)).expect(
            "pm4py not available — ensure Python and pm4py are installed (pip install pm4py)",
        )
    }
}

impl Default for ILPMiner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_simple_log() -> EventLog {
        let mut log = EventLog::new();
        let mut trace = crate::log::Trace::new("case1".to_string());

        trace.add_event(crate::log::Event::new("A", Utc::now()));
        trace.add_event(crate::log::Event::new("B", Utc::now()));
        trace.add_event(crate::log::Event::new("C", Utc::now()));

        log.add_trace(trace);

        let mut trace2 = crate::log::Trace::new("case2".to_string());
        trace2.add_event(crate::log::Event::new("A", Utc::now()));
        trace2.add_event(crate::log::Event::new("C", Utc::now()));
        log.add_trace(trace2);

        log
    }

    #[test]
    fn test_ilp_miner_creation() {
        let miner = ILPMiner::new();
        assert_eq!(miner.config.min_trace_coverage, 0.95);
    }

    #[test]
    fn test_ilp_miner_with_config() {
        let config = ILPMinerConfig {
            use_decomposition: true,
            min_trace_coverage: 0.90,
            max_places: 50,
        };
        let miner = ILPMiner::with_config(config);
        assert!(miner.config.use_decomposition);
        assert_eq!(miner.config.min_trace_coverage, 0.90);
    }

    #[test]
    fn test_ilp_config_builder() {
        let miner = ILPMiner::new()
            .with_decomposition(true)
            .with_min_coverage(0.85);

        assert!(miner.config.use_decomposition);
        assert_eq!(miner.config.min_trace_coverage, 0.85);
    }

    #[test]
    fn test_ilp_miner_simple_discovery() {
        let log = create_simple_log();
        let miner = ILPMiner::new();
        let net = miner.discover(&log);

        // Should have transitions for each activity
        assert!(net.transitions.len() >= 3);
        // Should have places
        assert!(!net.places.is_empty());
    }
}
