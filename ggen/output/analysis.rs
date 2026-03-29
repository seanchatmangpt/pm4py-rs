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

use crate::log::EventLog;
use crate::models::PetriNet;
use super::helpers::{eventlog_to_pm4py, pm4py_petri_to_rust, rust_petri_to_pm4py};







/// Bridge to `pm4py.behavioral_similarity()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_behavioral_similarity(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("behavioral_similarity", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.check_is_workflow_net()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_check_is_workflow_net(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("check_is_workflow_net", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.check_soundness()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_check_soundness(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("check_soundness", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.cluster_log()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_cluster_log(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("cluster_log", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.compute_emd()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_compute_emd(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("compute_emd", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.construct_synchronous_product_net()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_construct_synchronous_product_net(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("construct_synchronous_product_net", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.embeddings_similarity()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_embeddings_similarity(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("embeddings_similarity", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.generate_marking()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_generate_marking(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("generate_marking", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.get_activity_labels()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_activity_labels(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_activity_labels", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.get_enabled_transitions()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_get_enabled_transitions(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_enabled_transitions", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.insert_artificial_start_end()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_insert_artificial_start_end(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("insert_artificial_start_end", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.insert_case_arrival_finish_rate()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_insert_case_arrival_finish_rate(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("insert_case_arrival_finish_rate", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.insert_case_service_waiting_time()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_insert_case_service_waiting_time(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("insert_case_service_waiting_time", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.label_sets_similarity()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_label_sets_similarity(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("label_sets_similarity", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.map_labels_from_second_model()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_map_labels_from_second_model(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("map_labels_from_second_model", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.maximal_decomposition()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_maximal_decomposition(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("maximal_decomposition", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.reduce_petri_net_implicit_places()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_reduce_petri_net_implicit_places(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("reduce_petri_net_implicit_places", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.reduce_petri_net_invisibles()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_reduce_petri_net_invisibles(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("reduce_petri_net_invisibles", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.replace_activity_labels()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_replace_activity_labels(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("replace_activity_labels", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.simplicity_petri_net()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_simplicity_petri_net(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("simplicity_petri_net", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.solve_extended_marking_equation()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_solve_extended_marking_equation(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("solve_extended_marking_equation", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.solve_marking_equation()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_solve_marking_equation(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("solve_marking_equation", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.structural_similarity()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_structural_similarity(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("structural_similarity", (py_log,))?;
    drop(result);
    Ok(())
}

