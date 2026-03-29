#![allow(dropping_references, clippy::redundant_closure_call)]
//! pm4py OCEL function tests — auto-generated, DO NOT EDIT
//! Source: pm4py-rust/ggen/ontology/pm4py-api.ttl
//! Regenerate: cd pm4py-rust/ggen && /Users/sac/ggen/target/release/ggen sync --manifest ggen.toml
//!
//! Uses a synthetic OCEL created via pm4py.convert_log_to_ocel() from the
//! running-example XES fixture.  Skips gracefully when pm4py is unavailable.

#![cfg(test)]
#![allow(deprecated, dead_code, unused_variables, unused_imports)]

use pm4py::log::{EventLog, Trace, Event};
use pm4py::models::{PetriNet, Place, Transition, Arc};
use chrono::Utc;

// ── Runtime guard ────────────────────────────────────────────────────────────

#[allow(deprecated)]
fn pm4py_available() -> bool {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        py.import("pm4py").is_ok() && py.import("pandas").is_ok()
    })
}

// ── Test data paths ───────────────────────────────────────────────────────────

const XES_PATH: &str = "test_data/running-example.xes";

// ── OCEL fixture ─────────────────────────────────────────────────────────────

/// Create a synthetic OCEL from the running-example XES log.
/// Uses pm4py.convert_log_to_ocel() to produce a real pm4py OCEL object.
#[allow(deprecated)]
fn synthetic_ocel_object(py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    let pm4py_mod = py.import("pm4py")?;
    let log = pm4py_mod.call_method1("read_xes", (XES_PATH,))?;
    let ocel = pm4py_mod.call_method1("convert_log_to_ocel", (log,))?;
    Ok(ocel.into())
}

// ── Generated OCEL tests ─────────────────────────────────────────────────────







#[test]
fn call_discover_objects_graph_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_objects_graph_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_discover_objects_graph(py, ocel)
    });
    result.expect("discover_objects_graph failed");
}






#[test]
fn call_discover_oc_petri_net_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_oc_petri_net_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_discover_oc_petri_net(py, ocel)
    });
    let discovered = result.expect("discover_oc_petri_net failed");
    assert!(!discovered.places.is_empty(),
        "discover_oc_petri_net: PetriNet must have places");
    assert!(!discovered.transitions.is_empty(),
        "discover_oc_petri_net: PetriNet must have transitions");
}






#[test]
fn call_discover_ocdfg_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_ocdfg_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_discover_ocdfg(py, ocel)
    });
    result.expect("discover_ocdfg failed");
}






#[test]
fn call_ocel_add_index_based_timedelta_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_add_index_based_timedelta_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_add_index_based_timedelta(py, ocel)
    });
    result.expect("ocel_add_index_based_timedelta failed");
}






#[test]
fn call_ocel_drop_duplicates_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_drop_duplicates_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_drop_duplicates(py, ocel)
    });
    result.expect("ocel_drop_duplicates failed");
}






#[test]
fn call_ocel_e2o_lifecycle_enrichment_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_e2o_lifecycle_enrichment_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_e2o_lifecycle_enrichment(py, ocel)
    });
    result.expect("ocel_e2o_lifecycle_enrichment failed");
}






#[test]
fn call_ocel_get_attribute_names_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_get_attribute_names_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_get_attribute_names(py, ocel)
    });
    result.expect("ocel_get_attribute_names failed");
}






#[test]
fn call_ocel_get_object_types_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_get_object_types_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_get_object_types(py, ocel)
    });
    result.expect("ocel_get_object_types failed");
}






#[test]
fn call_ocel_merge_duplicates_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_merge_duplicates_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_merge_duplicates(py, ocel)
    });
    result.expect("ocel_merge_duplicates failed");
}






#[test]
fn call_ocel_o2o_enrichment_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_o2o_enrichment_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_o2o_enrichment(py, ocel)
    });
    result.expect("ocel_o2o_enrichment failed");
}






#[test]
fn call_ocel_object_type_activities_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_object_type_activities_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_object_type_activities(py, ocel)
    });
    result.expect("ocel_object_type_activities failed");
}






#[test]
fn call_ocel_objects_interactions_summary_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_objects_interactions_summary_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_objects_interactions_summary(py, ocel)
    });
    result.expect("ocel_objects_interactions_summary failed");
}






#[test]
fn call_ocel_objects_ot_count_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_objects_ot_count_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_objects_ot_count(py, ocel)
    });
    result.expect("ocel_objects_ot_count failed");
}






#[test]
fn call_ocel_objects_summary_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_objects_summary_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_objects_summary(py, ocel)
    });
    result.expect("ocel_objects_summary failed");
}






#[test]
fn call_ocel_temporal_summary_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_ocel_temporal_summary_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_ocel_temporal_summary(py, ocel)
    });
    result.expect("ocel_temporal_summary failed");
}






#[test]
fn call_sample_ocel_connected_components_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_sample_ocel_connected_components_ocel_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        let ocel_obj = synthetic_ocel_object(py)?;
        let ocel: &pyo3::types::PyAny = ocel_obj.as_ref(py);
        pm4py::python::generated::ocpm::call_sample_ocel_connected_components(py, ocel)
    });
    result.expect("sample_ocel_connected_components failed");
}

