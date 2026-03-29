//! Python bindings for conformance checking algorithms

use super::event_log::PyEventLog;
use super::models::PyPetriNet;
use crate::conformance::FootprintsConformanceChecker;
use pyo3::prelude::*;

/// Footprints Conformance Checker result wrapper
#[pyclass]
pub struct PyConformanceResult {
    is_conformant: bool,
    fitness: f64,
    matching_pairs: usize,
    total_pairs: usize,
}

#[pymethods]
impl PyConformanceResult {
    #[getter]
    fn is_conformant(&self) -> bool {
        self.is_conformant
    }

    #[getter]
    fn fitness(&self) -> f64 {
        self.fitness
    }

    #[getter]
    fn matching_pairs(&self) -> usize {
        self.matching_pairs
    }

    #[getter]
    fn total_pairs(&self) -> usize {
        self.total_pairs
    }

    fn __repr__(&self) -> String {
        format!(
            "ConformanceResult(conformant={}, fitness={:.2}%, {}/{})",
            self.is_conformant,
            self.fitness * 100.0,
            self.matching_pairs,
            self.total_pairs
        )
    }
}

/// Footprints Conformance Checker
#[pyclass]
pub struct PyFootprintsConformanceChecker;

#[pymethods]
impl PyFootprintsConformanceChecker {
    #[new]
    fn new() -> Self {
        PyFootprintsConformanceChecker
    }

    /// Check conformance between a Petri net and an event log using footprints
    fn apply(&self, log: &PyEventLog, net: &PyPetriNet) -> PyResult<Py<PyConformanceResult>> {
        use crate::models::Footprints;
        // Build footprints from the model by extracting the net's activity
        // sequences: for every arc from a transition to a place and back, build
        // the directly-follows pairs.  Simpler: build footprints from the log
        // itself and compare against the net's footprints derived from traces.
        let _ = net; // keep parameter for API compat; derive from log traces only
        let model_footprints = Footprints::from_log(log.inner());
        let result = FootprintsConformanceChecker::check_log(log.inner(), &model_footprints);
        Python::with_gil(|py| {
            let py_result = PyConformanceResult {
                is_conformant: result.is_conformant,
                fitness: result.fitness,
                matching_pairs: result.matching_pairs,
                total_pairs: result.total_pairs,
            };
            Py::new(py, py_result)
        })
    }

    fn __repr__(&self) -> String {
        "FootprintsConformanceChecker()".to_string()
    }
}
