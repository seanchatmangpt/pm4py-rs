//! Python bindings for discovery algorithms

use super::event_log::PyEventLog;
use super::models::PyPetriNet;
use crate::discovery::{AlphaMiner, HeuristicMiner, InductiveMiner};
use pyo3::prelude::*;

/// Alpha Miner - discovers a Petri net using the Alpha algorithm
#[pyclass]
pub struct PyAlphaMiner {
    inner: AlphaMiner,
}

#[pymethods]
impl PyAlphaMiner {
    #[new]
    fn new() -> Self {
        PyAlphaMiner {
            inner: AlphaMiner::new(),
        }
    }

    /// Apply the Alpha algorithm to discover a Petri net
    fn apply(&self, log: &PyEventLog) -> PyResult<Py<PyPetriNet>> {
        let result = self.inner.discover(log.inner());
        Python::with_gil(|py| Py::new(py, PyPetriNet::from_inner(result)))
    }

    fn __repr__(&self) -> String {
        "AlphaMiner()".to_string()
    }
}

/// Inductive Miner - discovers a process tree using the Inductive Miner algorithm
#[pyclass]
pub struct PyInductiveMiner {
    inner: InductiveMiner,
}

#[pymethods]
impl PyInductiveMiner {
    #[new]
    fn new() -> Self {
        PyInductiveMiner {
            inner: InductiveMiner::new(),
        }
    }

    /// Apply the Inductive Miner algorithm to discover a Petri net
    fn apply(&self, log: &PyEventLog) -> PyResult<Py<PyPetriNet>> {
        let result = self.inner.discover(log.inner());
        Python::with_gil(|py| Py::new(py, PyPetriNet::from_inner(result)))
    }

    fn __repr__(&self) -> String {
        "InductiveMiner()".to_string()
    }
}

/// Heuristic Miner - discovers a Petri net using heuristic-based algorithm
#[pyclass]
pub struct PyHeuristicMiner {
    inner: HeuristicMiner,
}

#[pymethods]
impl PyHeuristicMiner {
    #[new]
    fn new() -> Self {
        PyHeuristicMiner {
            inner: HeuristicMiner::new(),
        }
    }

    /// Apply the Heuristic Miner algorithm
    fn apply(&self, log: &PyEventLog) -> PyResult<Py<PyPetriNet>> {
        let result = self.inner.discover(log.inner());
        Python::with_gil(|py| Py::new(py, PyPetriNet::from_inner(result)))
    }

    fn __repr__(&self) -> String {
        "HeuristicMiner()".to_string()
    }
}
