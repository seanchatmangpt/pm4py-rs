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







/// Bridge to `pm4py.correlation_miner()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_correlation_miner(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("correlation_miner", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.derive_minimum_self_distance()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_derive_minimum_self_distance(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("derive_minimum_self_distance", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_batches()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_batches(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_batches", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_bpmn_inductive()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_bpmn_inductive(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_bpmn_inductive", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_declare()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_declare(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_declare", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_dfg()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_dfg(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_dfg", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_dfg_typed()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_dfg_typed(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_dfg_typed", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_directly_follows_graph()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_directly_follows_graph(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_directly_follows_graph", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_etot()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_discover_etot(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("discover_etot", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_eventually_follows_graph()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_eventually_follows_graph(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_eventually_follows_graph", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_footprints()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_footprints(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_footprints", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_heuristics_net()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_heuristics_net(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_heuristics_net", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_log_skeleton()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_log_skeleton(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_log_skeleton", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_otg()`
///
/// Call style: `ocel_only` → convert: `unit`
pub fn call_discover_otg(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("discover_otg", (ocel,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_performance_dfg()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_performance_dfg(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_performance_dfg", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_petri_net_alpha()`
///
/// Call style: `log_only` → convert: `petri_net_triple`
pub fn call_discover_petri_net_alpha(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_alpha", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}






/// Bridge to `pm4py.discover_petri_net_alpha_plus()`
///
/// Call style: `log_only` → convert: `petri_net_triple`
pub fn call_discover_petri_net_alpha_plus(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_alpha_plus", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}






/// Bridge to `pm4py.discover_petri_net_heuristics()`
///
/// Call style: `log_only` → convert: `petri_net_triple`
pub fn call_discover_petri_net_heuristics(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_heuristics", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}






/// Bridge to `pm4py.discover_petri_net_ilp()`
///
/// Call style: `log_only` → convert: `petri_net_triple`
pub fn call_discover_petri_net_ilp(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_ilp", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}






/// Bridge to `pm4py.discover_petri_net_inductive()`
///
/// Call style: `log_only` → convert: `petri_net_triple`
pub fn call_discover_petri_net_inductive(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_inductive", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}






/// Bridge to `pm4py.discover_powl()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_powl(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_powl", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_prefix_tree()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_prefix_tree(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_prefix_tree", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_process_tree_inductive()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_process_tree_inductive(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_process_tree_inductive", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_temporal_profile()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_temporal_profile(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_temporal_profile", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.discover_transition_system()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_discover_transition_system(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_transition_system", (py_log,))?;
    drop(result);
    Ok(())
}

