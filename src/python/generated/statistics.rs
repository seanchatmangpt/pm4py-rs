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

/// Bridge to `pm4py.get_activity_position_summary()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_activity_position_summary(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_activity_position_summary", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_all_case_durations()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_all_case_durations(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_all_case_durations", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_case_arrival_average()`
///
/// Call style: `log_only` → convert: `f64`
pub fn call_get_case_arrival_average(py: Python<'_>, log: &EventLog) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_case_arrival_average", (py_log,))?;
    result.extract::<f64>()
}

/// Bridge to `pm4py.get_case_duration()`
///
/// Call style: `log_only` → convert: `f64`
pub fn call_get_case_duration(py: Python<'_>, log: &EventLog) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_case_duration", (py_log,))?;
    result.extract::<f64>()
}

/// Bridge to `pm4py.get_case_overlap()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_case_overlap(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_case_overlap", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_cycle_time()`
///
/// Call style: `log_only` → convert: `f64`
pub fn call_get_cycle_time(py: Python<'_>, log: &EventLog) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_cycle_time", (py_log,))?;
    result.extract::<f64>()
}

/// Bridge to `pm4py.get_end_activities()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_end_activities(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_end_activities", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_event_attribute_values()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_event_attribute_values(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_event_attribute_values", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_event_attributes()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_event_attributes(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_event_attributes", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_frequent_trace_segments()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_frequent_trace_segments(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_frequent_trace_segments", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_minimum_self_distance_witnesses()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_minimum_self_distance_witnesses(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_minimum_self_distance_witnesses", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_minimum_self_distances()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_minimum_self_distances(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_minimum_self_distances", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_process_cube()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_process_cube(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_process_cube", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_rework_cases_per_activity()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_rework_cases_per_activity(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_rework_cases_per_activity", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_service_time()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_service_time(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_service_time", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_start_activities()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_start_activities(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_start_activities", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_stochastic_language()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_stochastic_language(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_stochastic_language", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_trace_attribute_values()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_trace_attribute_values(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_trace_attribute_values", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_trace_attributes()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_trace_attributes(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_trace_attributes", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_variants()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_variants(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_variants", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_variants_as_tuples()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_variants_as_tuples(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_variants_as_tuples", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.get_variants_paths_duration()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_get_variants_paths_duration(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("get_variants_paths_duration", (py_log,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.split_by_process_variant()`
///
/// Call style: `log_only` → convert: `dict`
pub fn call_split_by_process_variant(py: Python<'_>, log: &EventLog) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("split_by_process_variant", (py_log,))?;
    let _ = result;
    Ok(())
}
