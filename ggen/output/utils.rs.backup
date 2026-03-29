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







/// Bridge to `pm4py.deserialize()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_deserialize(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("deserialize", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_activity_based_resource_similarity()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_activity_based_resource_similarity(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_activity_based_resource_similarity", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_handover_of_work_network()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_handover_of_work_network(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_handover_of_work_network", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_network_analysis()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_network_analysis(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_network_analysis", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_organizational_roles()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_organizational_roles(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_organizational_roles", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_subcontracting_network()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_subcontracting_network(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_subcontracting_network", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_working_together_network()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_working_together_network(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_working_together_network", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.extract_features_dataframe()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_extract_features_dataframe(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("extract_features_dataframe", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.extract_ocel_features()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_extract_ocel_features(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("extract_ocel_features", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.extract_outcome_enriched_dataframe()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_extract_outcome_enriched_dataframe(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("extract_outcome_enriched_dataframe", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.extract_target_vector()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_extract_target_vector(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("extract_target_vector", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.extract_temporal_features_dataframe()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_extract_temporal_features_dataframe(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("extract_temporal_features_dataframe", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.format_dataframe()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_format_dataframe(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("format_dataframe", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.generate_process_tree()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_generate_process_tree(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("generate_process_tree", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.get_prefixes_from_log()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_get_prefixes_from_log(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_prefixes_from_log", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.parse_event_log_string()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_parse_event_log_string(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("parse_event_log_string", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.parse_powl_model_string()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_parse_powl_model_string(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("parse_powl_model_string", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.parse_process_tree()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_parse_process_tree(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("parse_process_tree", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.play_out()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_play_out(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("play_out", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.project_on_event_attribute()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_project_on_event_attribute(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("project_on_event_attribute", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.rebase()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_rebase(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("rebase", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.sample_cases()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_sample_cases(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("sample_cases", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.sample_events()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_sample_events(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("sample_events", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.serialize()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_serialize(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("serialize", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.set_classifier()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_set_classifier(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("set_classifier", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.split_train_test()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_split_train_test(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("split_train_test", (py_log,))?;
    drop(result);
    Ok(())
}

