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







/// Bridge to `pm4py.convert_log_to_networkx()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_log_to_networkx(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_log_to_networkx", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_log_to_ocel()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_log_to_ocel(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_log_to_ocel", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_log_to_time_intervals()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_log_to_time_intervals(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_log_to_time_intervals", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_ocel_to_networkx()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_convert_ocel_to_networkx(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("convert_ocel_to_networkx", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_petri_net_to_networkx()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_petri_net_to_networkx(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_petri_net_to_networkx", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_petri_net_type()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_petri_net_type(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_petri_net_type", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_bpmn()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_bpmn(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_bpmn", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_dataframe()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_dataframe(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_dataframe", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_event_log()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_event_log(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_event_log", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_event_stream()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_event_stream(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_event_stream", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_petri_net()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_petri_net(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_petri_net", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_powl()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_powl(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_powl", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_process_tree()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_process_tree(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_process_tree", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.convert_to_reachability_graph()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_convert_to_reachability_graph(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("convert_to_reachability_graph", (py_log,))?;
    drop(result);
    Ok(())
}

