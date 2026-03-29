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







/// Bridge to `pm4py.filter_activities_rework()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_activities_rework(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_activities_rework", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_activity_done_different_resources()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_activity_done_different_resources(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_activity_done_different_resources", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_between()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_between(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_between", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_case_performance()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_case_performance(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_case_performance", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_case_size()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_case_size(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_case_size", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_dfg_activities_percentage()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_dfg_activities_percentage(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_dfg_activities_percentage", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_dfg_paths_percentage()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_dfg_paths_percentage(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_dfg_paths_percentage", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_directly_follows_relation()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_directly_follows_relation(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_directly_follows_relation", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_end_activities()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_end_activities(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_end_activities", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_event_attribute_values()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_event_attribute_values(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_event_attribute_values", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_eventually_follows_relation()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_eventually_follows_relation(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_eventually_follows_relation", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_four_eyes_principle()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_four_eyes_principle(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_four_eyes_principle", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_log_relative_occurrence_event_attribute()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_log_relative_occurrence_event_attribute(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_log_relative_occurrence_event_attribute", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_activities_connected_object_type()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_activities_connected_object_type(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_activities_connected_object_type", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_cc_activity()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_cc_activity(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_cc_activity", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_cc_length()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_cc_length(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_cc_length", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_cc_object()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_cc_object(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_cc_object", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_cc_otype()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_cc_otype(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_cc_otype", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_end_events_per_object_type()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_end_events_per_object_type(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_end_events_per_object_type", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_event_attribute()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_event_attribute(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_event_attribute", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_events()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_events(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_events", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_events_timestamp()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_events_timestamp(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_events_timestamp", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_object_attribute()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_object_attribute(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_object_attribute", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_object_per_type_count()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_object_per_type_count(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_object_per_type_count", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_object_types()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_object_types(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_object_types", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_object_types_allowed_activities()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_object_types_allowed_activities(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_object_types_allowed_activities", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_objects()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_objects(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_objects", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_ocel_start_events_per_object_type()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_filter_ocel_start_events_per_object_type(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("filter_ocel_start_events_per_object_type", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_paths_performance()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_paths_performance(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_paths_performance", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_prefixes()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_prefixes(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_prefixes", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_start_activities()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_start_activities(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_start_activities", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_suffixes()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_suffixes(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_suffixes", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_time_range()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_time_range(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_time_range", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_trace_attribute_values()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_trace_attribute_values(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_trace_attribute_values", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_trace_segments()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_trace_segments(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_trace_segments", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_variants()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_variants(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_variants", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_variants_by_coverage_percentage()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_variants_by_coverage_percentage(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_variants_by_coverage_percentage", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.filter_variants_top_k()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_filter_variants_top_k(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("filter_variants_top_k", (py_log,))?;
    drop(result);
    Ok(())
}

