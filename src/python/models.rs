//! Python bindings for process models (Petri nets, process trees, etc.)

use crate::models::PetriNet;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Petri Net representation
#[pyclass]
pub struct PyPetriNet {
    inner: PetriNet,
}

#[pymethods]
impl PyPetriNet {
    #[new]
    fn new() -> Self {
        PyPetriNet {
            inner: PetriNet::new(),
        }
    }

    #[getter]
    fn places_count(&self) -> usize {
        self.inner.places.len()
    }

    #[getter]
    fn transitions_count(&self) -> usize {
        self.inner.transitions.len()
    }

    #[getter]
    fn arcs_count(&self) -> usize {
        self.inner.arcs.len()
    }

    fn places(&self) -> PyResult<Py<PyList>> {
        Python::with_gil(|py| {
            let list = PyList::new(py, Vec::<&str>::new());
            for place in &self.inner.places {
                let dict = PyDict::new(py);
                dict.set_item("id", place.id.as_str())?;
                dict.set_item("name", place.name.as_str())?;
                list.append(dict)?;
            }
            Ok(list.into())
        })
    }

    fn transitions(&self) -> PyResult<Py<PyList>> {
        Python::with_gil(|py| {
            let list = PyList::new(py, Vec::<&str>::new());
            for transition in &self.inner.transitions {
                let dict = PyDict::new(py);
                dict.set_item("id", transition.id.as_str())?;
                dict.set_item("name", transition.name.as_str())?;
                dict.set_item("label", transition.label.as_deref().unwrap_or(""))?;
                dict.set_item("is_silent", transition.label.is_none())?;
                list.append(dict)?;
            }
            Ok(list.into())
        })
    }

    fn arcs(&self) -> PyResult<Py<PyList>> {
        Python::with_gil(|py| {
            let list = PyList::new(py, Vec::<&str>::new());
            for arc in &self.inner.arcs {
                let dict = PyDict::new(py);
                dict.set_item("from", arc.from.as_str())?;
                dict.set_item("to", arc.to.as_str())?;
                list.append(dict)?;
            }
            Ok(list.into())
        })
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "JSON serialization error: {}",
                e
            ))
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "PetriNet(places={}, transitions={}, arcs={})",
            self.places_count(),
            self.transitions_count(),
            self.arcs_count()
        )
    }
}

impl PyPetriNet {
    pub fn from_inner(inner: PetriNet) -> Self {
        PyPetriNet { inner }
    }

    pub fn inner(&self) -> &PetriNet {
        &self.inner
    }
}

/// Process Tree representation
#[pyclass]
pub struct PyProcessTree {
    inner: crate::models::ProcessTree,
}

#[pymethods]
impl PyProcessTree {
    #[new]
    fn new() -> Self {
        // Create a default SKIP process tree as the root
        PyProcessTree {
            inner: crate::models::ProcessTree::new(crate::models::ProcessTreeNode::activity(
                "SKIP",
            )),
        }
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "JSON serialization error: {}",
                e
            ))
        })
    }

    fn __repr__(&self) -> String {
        "ProcessTree()".to_string()
    }
}

impl PyProcessTree {
    pub fn from_inner(inner: crate::models::ProcessTree) -> Self {
        PyProcessTree { inner }
    }

    pub fn inner(&self) -> &crate::models::ProcessTree {
        &self.inner
    }
}
