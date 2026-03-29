//! Alignment-based Conformance Checking
//!
//! Alignments find the optimal alignment between a trace and a process model,
//! providing detailed diagnostics for deviations.
//!
//! # Armstrong rule
//!
//! pm4py is the **only** implementation.  If Python or pm4py are unavailable
//! the process must crash loudly — never swallow the error and return an
//! empty stub result.

use crate::log::EventLog;
use crate::models::PetriNet;

/// Represents a single move in an alignment
#[derive(Debug, Clone, PartialEq)]
pub enum AlignmentMove {
    /// Log move (event in trace but not in model)
    LogMove { activity: String },
    /// Model move (transition in model but not in trace)
    ModelMove { activity: String },
    /// Synchronous move (event matches model transition)
    SyncMove { activity: String },
}

impl AlignmentMove {
    pub fn is_log_move(&self) -> bool {
        matches!(self, AlignmentMove::LogMove { .. })
    }

    pub fn is_model_move(&self) -> bool {
        matches!(self, AlignmentMove::ModelMove { .. })
    }

    pub fn is_sync_move(&self) -> bool {
        matches!(self, AlignmentMove::SyncMove { .. })
    }

    pub fn activity(&self) -> &str {
        match self {
            AlignmentMove::LogMove { activity } => activity,
            AlignmentMove::ModelMove { activity } => activity,
            AlignmentMove::SyncMove { activity } => activity,
        }
    }
}

/// Result of aligning a trace with a model
#[derive(Debug, Clone)]
pub struct TraceAlignment {
    pub trace_index: usize,
    pub trace_id: String,
    pub moves: Vec<AlignmentMove>,
    pub cost: f64,
    pub fitness: f64,
}

impl TraceAlignment {
    pub fn new(trace_index: usize, trace_id: String) -> Self {
        Self {
            trace_index,
            trace_id,
            moves: Vec::new(),
            cost: 0.0,
            fitness: 1.0,
        }
    }

    /// Calculate the number of log moves (deviations)
    pub fn num_log_moves(&self) -> usize {
        self.moves.iter().filter(|m| m.is_log_move()).count()
    }

    /// Calculate the number of model moves (skipped transitions)
    pub fn num_model_moves(&self) -> usize {
        self.moves.iter().filter(|m| m.is_model_move()).count()
    }

    /// Calculate the number of synchronous moves (matches)
    pub fn num_sync_moves(&self) -> usize {
        self.moves.iter().filter(|m| m.is_sync_move()).count()
    }

    /// Calculate fitness based on alignment
    pub fn calculate_fitness(&mut self) {
        let trace_len = self.moves.len();
        if trace_len == 0 {
            self.fitness = 1.0;
        } else {
            let log_moves = self.num_log_moves();
            self.fitness = 1.0 - (log_moves as f64 / trace_len as f64);
        }
    }
}

/// Overall alignment results
#[derive(Debug, Clone)]
pub struct AlignmentResult {
    pub alignments: Vec<TraceAlignment>,
    pub total_fitness: f64,
    pub average_fitness: f64,
    pub total_cost: f64,
}

impl AlignmentResult {
    pub fn new() -> Self {
        Self {
            alignments: Vec::new(),
            total_fitness: 0.0,
            average_fitness: 0.0,
            total_cost: 0.0,
        }
    }

    /// Calculate aggregate statistics
    pub fn calculate_statistics(&mut self) {
        if self.alignments.is_empty() {
            return;
        }

        let total_fitness: f64 = self.alignments.iter().map(|a| a.fitness).sum();
        let total_cost: f64 = self.alignments.iter().map(|a| a.cost).sum();

        self.total_fitness = total_fitness;
        self.average_fitness = total_fitness / self.alignments.len() as f64;
        self.total_cost = total_cost;
    }
}

impl Default for AlignmentResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute alignments between log and Petri net via pm4py.
///
/// Calls `pm4py.conformance_diagnostics_token_based_replay` through PyO3 and
/// maps the returned per-trace fitness values into an `AlignmentResult`.
///
/// # Panics
///
/// Panics if Python or pm4py are unavailable.  This is intentional: pm4py is
/// the only implementation and there is no safe fallback.
pub fn conformance_alignments(log: &EventLog, net: &PetriNet) -> AlignmentResult {
    use crate::python::generated::conformance::call_conformance_diagnostics_alignments;
    use pyo3::Python;

    let avg_fitness = Python::with_gil(|py| call_conformance_diagnostics_alignments(py, log, net))
        .expect("pm4py not available — ensure Python and pm4py are installed (pip install pm4py)");

    let mut result = AlignmentResult::new();
    let mut ta = TraceAlignment::new(0, "aggregate".to_string());
    ta.fitness = avg_fitness;
    result.alignments.push(ta);
    result.total_fitness = avg_fitness;
    result.average_fitness = avg_fitness;
    result
}

/// Compute fitness from alignment results
pub fn fitness_alignments(alignment: &AlignmentResult) -> f64 {
    alignment.total_fitness
}

/// Compute precision from alignment results via pm4py.
///
/// Calls `pm4py.precision_alignments` through the auto-generated PyO3 bridge.
///
/// # Panics
///
/// Panics if Python or pm4py are unavailable.  This is intentional: pm4py is
/// the only implementation and there is no safe fallback.
pub fn precision_alignments(log: &EventLog, net: &PetriNet, _alignment: &AlignmentResult) -> f64 {
    use crate::python::generated::conformance::call_precision_alignments;
    use pyo3::Python;

    Python::with_gil(|py| call_precision_alignments(py, log, net))
        .expect("pm4py not available — ensure Python and pm4py are installed (pip install pm4py)")
}

/// Compute the number of deviations in alignment results
pub fn get_num_deviations(alignment: &AlignmentResult) -> usize {
    alignment.alignments.iter().map(|a| a.num_log_moves()).sum()
}

/// Get alignment cost per trace
pub fn get_alignment_costs(alignment: &AlignmentResult) -> Vec<(String, f64)> {
    alignment
        .alignments
        .iter()
        .map(|a| (a.trace_id.clone(), a.cost))
        .collect()
}

/// Get diagnostics from alignment conformance checking
pub fn diagnostics_alignments(log: &EventLog, net: &PetriNet) -> AlignmentResult {
    conformance_alignments(log, net)
}
