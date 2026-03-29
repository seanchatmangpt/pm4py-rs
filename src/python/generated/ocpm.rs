//! PyO3 bridge — auto-generated, DO NOT EDIT
//! Source: pm4py-rust/ggen/ontology/pm4py-api.ttl
//! Regenerate: cd pm4py-rust/ggen && ggen sync --from . --to ../src/python/generated
//!
//! Each function delegates to the Python pm4py library over PyO3.
//! Failures propagate as PyResult::Err — callers decide whether to fallback.
//!
//! WvdA soundness: conformance functions cap log at 1,000 traces to bound runtime.
//! Armstrong: let-it-crash — no silent fallbacks inside this module.

#![allow(dead_code)]
#![allow(unused_imports)]

use pyo3::prelude::*;
use pyo3::types::PyAny;

use super::helpers::{eventlog_to_pm4py, pm4py_petri_to_rust, rust_petri_to_pm4py};
use crate::log::EventLog;
use crate::models::PetriNet;

/// Bridge to `pm4py.cluster_equivalent_ocel()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_cluster_equivalent_ocel(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("cluster_equivalent_ocel", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.discover_objects_graph()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_discover_objects_graph(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("discover_objects_graph", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.discover_oc_petri_net()`
///
/// Call style: `ocel_only` → convert: `petri_net_triple`
pub fn call_discover_oc_petri_net(py: Python<'_>, ocel: &PyAny) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("discover_oc_petri_net", (ocel,))?;
    // discover_oc_petri_net returns dict keyed by object type, not a tuple
    let pns: &PyAny = result.get_item("petri_nets")?;
    let first_ot = pns.call_method0("keys")?.iter()?.next().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("no object types in OC Petri net")
    })??;
    let triple: &PyAny = pns.get_item(first_ot)?;
    let net_obj: &PyAny = triple.get_item(0)?;
    let im_obj: &PyAny = triple.get_item(1)?;
    let fm_obj: &PyAny = triple.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}

/// Bridge to `pm4py.discover_ocdfg()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_discover_ocdfg(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("discover_ocdfg", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_add_index_based_timedelta()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_add_index_based_timedelta(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_add_index_based_timedelta", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_drop_duplicates()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_drop_duplicates(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_drop_duplicates", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_e2o_lifecycle_enrichment()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_e2o_lifecycle_enrichment(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_e2o_lifecycle_enrichment", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_flattening()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_flattening(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_flattening", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_get_attribute_names()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_get_attribute_names(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_get_attribute_names", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_get_object_types()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_get_object_types(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_get_object_types", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_merge_duplicates()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_merge_duplicates(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_merge_duplicates", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_o2o_enrichment()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_o2o_enrichment(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_o2o_enrichment", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_object_type_activities()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_object_type_activities(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_object_type_activities", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_objects_interactions_summary()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_objects_interactions_summary(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_objects_interactions_summary", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_objects_ot_count()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_objects_ot_count(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_objects_ot_count", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_objects_summary()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_objects_summary(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_objects_summary", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_sort_by_additional_column()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_sort_by_additional_column(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_sort_by_additional_column", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.ocel_temporal_summary()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_ocel_temporal_summary(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("ocel_temporal_summary", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.sample_ocel_connected_components()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_sample_ocel_connected_components(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("sample_ocel_connected_components", (ocel,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.sample_ocel_objects()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_sample_ocel_objects(py: Python<'_>, ocel: &PyAny) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("sample_ocel_objects", (ocel,))?;
    let _ = result;
    Ok(())
}
