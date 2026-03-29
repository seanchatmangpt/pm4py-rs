//! Python bindings for statistics and analysis

use super::event_log::PyEventLog;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Log Statistics
#[pyclass]
pub struct PyLogStatistics;

#[pymethods]
impl PyLogStatistics {
    #[new]
    fn new() -> Self {
        PyLogStatistics
    }

    /// Calculate basic statistics about an event log
    fn basic_stats(&self, log: &PyEventLog) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);

            let num_traces = log.inner().traces.len();
            let total_events: usize = log.inner().traces.iter().map(|t| t.len()).sum();
            let avg_trace_length = if num_traces > 0 {
                total_events as f64 / num_traces as f64
            } else {
                0.0
            };

            // Count unique variants
            let mut variants = std::collections::HashSet::new();
            for trace in &log.inner().traces {
                let key: Vec<_> = trace.events.iter().map(|e| e.activity.clone()).collect();
                variants.insert(key);
            }
            let num_variants = variants.len();

            dict.set_item("num_traces", num_traces)?;
            dict.set_item("num_events", total_events)?;
            dict.set_item("num_variants", num_variants)?;
            dict.set_item("avg_trace_length", avg_trace_length)?;

            // Calculate min/max trace length
            if num_traces > 0 {
                let min_length = log
                    .inner()
                    .traces
                    .iter()
                    .map(|t| t.len())
                    .min()
                    .unwrap_or(0);
                let max_length = log
                    .inner()
                    .traces
                    .iter()
                    .map(|t| t.len())
                    .max()
                    .unwrap_or(0);
                dict.set_item("min_trace_length", min_length)?;
                dict.set_item("max_trace_length", max_length)?;
            }

            Ok(dict.into())
        })
    }

    /// Get unique activities in the log
    fn get_activities(&self, log: &PyEventLog) -> PyResult<Py<PyList>> {
        Python::with_gil(|py| {
            let mut activities = std::collections::HashSet::new();
            for trace in &log.inner().traces {
                for event in &trace.events {
                    activities.insert(event.activity.clone());
                }
            }

            let mut sorted_acts: Vec<_> = activities.into_iter().collect();
            sorted_acts.sort();

            let list = PyList::new(py, sorted_acts);
            Ok(list.into())
        })
    }

    /// Get activity frequencies
    fn get_activity_frequencies(&self, log: &PyEventLog) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let mut frequencies = std::collections::HashMap::new();
            for trace in &log.inner().traces {
                for event in &trace.events {
                    *frequencies.entry(event.activity.clone()).or_insert(0) += 1;
                }
            }

            let dict = PyDict::new(py);
            for (activity, count) in frequencies {
                dict.set_item(&activity, count)?;
            }
            Ok(dict.into())
        })
    }

    /// Get variant frequency
    fn get_variants(&self, log: &PyEventLog) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let mut variants = std::collections::HashMap::new();
            for trace in &log.inner().traces {
                let key: Vec<_> = trace.events.iter().map(|e| e.activity.clone()).collect();
                *variants.entry(key).or_insert(0) += 1;
            }

            let dict = PyDict::new(py);
            for (variant, count) in variants {
                let variant_str = variant.join(",");
                dict.set_item(&variant_str, count)?;
            }
            Ok(dict.into())
        })
    }

    fn __repr__(&self) -> String {
        "LogStatistics()".to_string()
    }
}
