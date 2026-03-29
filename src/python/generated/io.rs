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

/// Bridge to `pm4py.read_bpmn()`
///
/// Call style: `io_read` → convert: `event_log`
pub fn call_read_bpmn(py: Python<'_>, path: &str) -> PyResult<EventLog> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_bpmn", (path,))?;
    super::helpers::pm4py_to_eventlog(py, result)
}

/// Bridge to `pm4py.read_dfg()`
///
/// Call style: `io_read` → convert: `event_log`
pub fn call_read_dfg(py: Python<'_>, path: &str) -> PyResult<EventLog> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_dfg", (path,))?;
    super::helpers::pm4py_to_eventlog(py, result)
}

/// Bridge to `pm4py.read_ocel()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel2()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel2(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel2", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel2_json()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel2_json(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel2_json", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel2_sqlite()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel2_sqlite(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel2_sqlite", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel2_xml()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel2_xml(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel2_xml", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel_csv()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel_csv(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel_csv", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel_json()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel_json(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel_json", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel_sqlite()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel_sqlite(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel_sqlite", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_ocel_xml()`
///
/// Call style: `io_read` → convert: `ocel`
pub fn call_read_ocel_xml(py: Python<'_>, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ocel_xml", (path,))?;
    let _ = result;
    Ok(())
}

/// Bridge to `pm4py.read_pnml()`
///
/// Call style: `io_read` → convert: `event_log`
pub fn call_read_pnml(py: Python<'_>, path: &str) -> PyResult<EventLog> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_pnml", (path,))?;
    super::helpers::pm4py_to_eventlog(py, result)
}

/// Bridge to `pm4py.read_ptml()`
///
/// Call style: `io_read` → convert: `event_log`
pub fn call_read_ptml(py: Python<'_>, path: &str) -> PyResult<EventLog> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_ptml", (path,))?;
    super::helpers::pm4py_to_eventlog(py, result)
}

/// Bridge to `pm4py.read_xes()`
///
/// Call style: `io_read` → convert: `event_log`
pub fn call_read_xes(py: Python<'_>, path: &str) -> PyResult<EventLog> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("read_xes", (path,))?;
    super::helpers::pm4py_to_eventlog(py, result)
}

/// Bridge to `pm4py.write_bpmn()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_bpmn(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_bpmn", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_dfg()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_dfg(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_dfg", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel2()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel2(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel2", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel2_json()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel2_json(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel2_json", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel2_sqlite()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel2_sqlite(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel2_sqlite", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel2_xml()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel2_xml(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel2_xml", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel_csv()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel_csv(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel_csv", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel_json()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel_json(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel_json", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel_sqlite()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel_sqlite(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel_sqlite", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ocel_xml()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ocel_xml(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ocel_xml", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_pnml()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_pnml(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_pnml", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_ptml()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_ptml(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_ptml", (py_log, path))?;
    Ok(())
}

/// Bridge to `pm4py.write_xes()`
///
/// Call style: `io_write` → convert: `unit`
pub fn call_write_xes(py: Python<'_>, log: &EventLog, path: &str) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    pm4py.call_method1("write_xes", (py_log, path))?;
    Ok(())
}
