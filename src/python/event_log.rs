//! Python bindings for EventLog structures

use crate::log::{Event, EventLog, Trace};
use chrono::Utc;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Python wrapper for Event
#[pyclass]
pub struct PyEvent {
    inner: Event,
}

#[pymethods]
impl PyEvent {
    #[new]
    fn new(activity: String, timestamp: String) -> PyResult<Self> {
        let ts = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid timestamp: {}", e))
            })?
            .with_timezone(&Utc);

        Ok(PyEvent {
            inner: Event::new(activity, ts),
        })
    }

    #[getter]
    fn activity(&self) -> String {
        self.inner.activity.clone()
    }

    #[setter]
    fn set_activity(&mut self, activity: String) {
        self.inner.activity = activity;
    }

    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    #[getter]
    fn resource(&self) -> Option<String> {
        self.inner.resource.clone()
    }

    fn set_resource(&mut self, resource: String) {
        self.inner.resource = Some(resource);
    }

    fn add_attribute(&mut self, key: String, value: String) {
        self.inner.attributes.insert(key, value);
    }

    fn get_attribute(&self, key: &str) -> Option<String> {
        self.inner.attributes.get(key).cloned()
    }

    fn attributes(&self) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (k, v) in &self.inner.attributes {
                dict.set_item(k, v)?;
            }
            Ok(dict.into())
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "Event(activity={}, timestamp={}, resource={:?})",
            self.inner.activity,
            self.inner.timestamp.to_rfc3339(),
            self.inner.resource
        )
    }
}

/// Python wrapper for Trace
#[pyclass]
pub struct PyTrace {
    inner: Trace,
}

#[pymethods]
impl PyTrace {
    #[new]
    fn new(case_id: String) -> Self {
        PyTrace {
            inner: Trace::new(case_id),
        }
    }

    #[getter]
    fn case_id(&self) -> String {
        self.inner.id.clone()
    }

    fn add_event(&mut self, activity: String, timestamp: String) -> PyResult<()> {
        let ts = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid timestamp: {}", e))
            })?
            .with_timezone(&Utc);

        self.inner.add_event(Event::new(activity, ts));
        Ok(())
    }

    fn add_event_with_resource(
        &mut self,
        activity: String,
        timestamp: String,
        resource: String,
    ) -> PyResult<()> {
        let ts = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid timestamp: {}", e))
            })?
            .with_timezone(&Utc);

        self.inner
            .add_event(Event::new(activity, ts).with_resource(resource));
        Ok(())
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn events(&self) -> PyResult<Py<PyList>> {
        Python::with_gil(|py| {
            let list = PyList::empty(py);
            for event in &self.inner.events {
                let py_event = PyEvent {
                    inner: event.clone(),
                };
                list.append(py_event.into_py(py))?;
            }
            Ok(list.into())
        })
    }

    fn __repr__(&self) -> String {
        format!("Trace(case_id={}, len={})", self.inner.id, self.inner.len())
    }
}

/// Python wrapper for EventLog
#[pyclass]
pub struct PyEventLog {
    inner: EventLog,
}

#[pymethods]
impl PyEventLog {
    #[new]
    fn new() -> Self {
        PyEventLog {
            inner: EventLog::new(),
        }
    }

    fn add_trace(&mut self, case_id: String) {
        let trace = Trace::new(case_id);
        self.inner.add_trace(trace);
    }

    fn add_trace_obj(&mut self, trace: &PyTrace) {
        self.inner.add_trace(trace.inner.clone());
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn traces(&self) -> PyResult<Py<PyList>> {
        Python::with_gil(|py| {
            let list = PyList::empty(py);
            for trace in &self.inner.traces {
                let py_trace = PyTrace {
                    inner: trace.clone(),
                };
                list.append(py_trace.into_py(py))?;
            }
            Ok(list.into())
        })
    }

    fn load_json(&mut self, json_str: String) -> PyResult<()> {
        let parsed: EventLog = serde_json::from_str(&json_str).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid JSON: {}", e))
        })?;
        self.inner = parsed;
        Ok(())
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "JSON serialization error: {}",
                e
            ))
        })
    }

    fn variant_count(&self) -> usize {
        let mut variants = std::collections::HashMap::new();
        for trace in &self.inner.traces {
            let key: Vec<_> = trace.events.iter().map(|e| e.activity.clone()).collect();
            *variants.entry(key).or_insert(0) += 1;
        }
        variants.len()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "EventLog(traces={}, variants={})",
            self.len(),
            self.variant_count()
        )
    }
}

impl PyEventLog {
    pub fn inner(&self) -> &EventLog {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut EventLog {
        &mut self.inner
    }
}
