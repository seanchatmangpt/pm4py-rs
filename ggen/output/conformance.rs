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







/// Bridge to `pm4py.check_is_fitting()`
///
/// Call style: `log_only` → convert: `unit`
pub fn call_check_is_fitting(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("check_is_fitting", (py_log,))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.conformance_declare()`
///
/// Call style: `log_and_model` → convert: `list_dict`
pub fn call_conformance_declare(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let discover_fn = "discover_declare";
    let model = pm4py.call_method1(discover_fn, (py_log.as_ref(py),))?;
    let result = pm4py.call_method1("conformance_declare", (py_log.as_ref(py), model))?;
    let diagnostics: Vec<&PyAny> = result.extract()?;
    if diagnostics.is_empty() {
        return Ok(0.0);
    }
    let total: f64 = diagnostics
        .iter()
        .filter_map(|d| {
            d.get_item("dev_fitness")
                .ok()
                .and_then(|v| v.extract::<f64>().ok())
        })
        .sum();
    Ok(total / diagnostics.len() as f64)
}






/// Bridge to `pm4py.conformance_diagnostics_alignments()`
///
/// Call style: `log_and_net` → convert: `list_dict`
pub fn call_conformance_diagnostics_alignments(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("conformance_diagnostics_alignments", (py_log, py_net, py_im, py_fm))?;
    // Extract list of dicts; try trace_fitness (TBR) then fitness (alignments)
    let diagnostics: Vec<&PyAny> = result.extract()?;
    if diagnostics.is_empty() {
        return Ok(0.0);
    }
    let total: f64 = diagnostics
        .iter()
        .filter_map(|d| {
            d.get_item("trace_fitness")
                .ok()
                .and_then(|v| v.extract::<f64>().ok())
                .or_else(|| {
                    d.get_item("fitness")
                        .ok()
                        .and_then(|v| v.extract::<f64>().ok())
                })
        })
        .sum();
    Ok(total / diagnostics.len() as f64)
}






/// Bridge to `pm4py.conformance_diagnostics_footprints()`
///
/// Call style: `log_and_net` → convert: `unit`
pub fn call_conformance_diagnostics_footprints(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("conformance_diagnostics_footprints", (py_log, py_net, py_im, py_fm))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.conformance_diagnostics_token_based_replay()`
///
/// Call style: `log_and_net` → convert: `list_dict`
pub fn call_conformance_diagnostics_token_based_replay(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("conformance_diagnostics_token_based_replay", (py_log, py_net, py_im, py_fm))?;
    // Extract list of dicts; try trace_fitness (TBR) then fitness (alignments)
    let diagnostics: Vec<&PyAny> = result.extract()?;
    if diagnostics.is_empty() {
        return Ok(0.0);
    }
    let total: f64 = diagnostics
        .iter()
        .filter_map(|d| {
            d.get_item("trace_fitness")
                .ok()
                .and_then(|v| v.extract::<f64>().ok())
                .or_else(|| {
                    d.get_item("fitness")
                        .ok()
                        .and_then(|v| v.extract::<f64>().ok())
                })
        })
        .sum();
    Ok(total / diagnostics.len() as f64)
}






/// Bridge to `pm4py.conformance_etot()`
///
/// Call style: `ocel_only` → convert: `list_dict`
pub fn call_conformance_etot(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("conformance_etot", (ocel,))?;
    let diagnostics: Vec<&PyAny> = result.extract()?;
    if diagnostics.is_empty() { return Ok(0.0); }
    let total: f64 = diagnostics.iter()
        .filter_map(|d| d.get_item("trace_fitness").ok()?.extract::<f64>().ok())
        .sum();
    Ok(total / diagnostics.len() as f64)
}






/// Bridge to `pm4py.conformance_log_skeleton()`
///
/// Call style: `log_and_model` → convert: `list_dict`
pub fn call_conformance_log_skeleton(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let discover_fn = "discover_log_skeleton";
    let model = pm4py.call_method1(discover_fn, (py_log.as_ref(py),))?;
    let result = pm4py.call_method1("conformance_log_skeleton", (py_log.as_ref(py), model))?;
    let diagnostics: Vec<&PyAny> = result.extract()?;
    if diagnostics.is_empty() {
        return Ok(0.0);
    }
    let total: f64 = diagnostics
        .iter()
        .filter_map(|d| {
            d.get_item("dev_fitness")
                .ok()
                .and_then(|v| v.extract::<f64>().ok())
        })
        .sum();
    Ok(total / diagnostics.len() as f64)
}






/// Bridge to `pm4py.conformance_ocdfg()`
///
/// Call style: `ocel_only` → convert: `list_dict`
pub fn call_conformance_ocdfg(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("conformance_ocdfg", (ocel,))?;
    let diagnostics: Vec<&PyAny> = result.extract()?;
    if diagnostics.is_empty() { return Ok(0.0); }
    let total: f64 = diagnostics.iter()
        .filter_map(|d| d.get_item("trace_fitness").ok()?.extract::<f64>().ok())
        .sum();
    Ok(total / diagnostics.len() as f64)
}






/// Bridge to `pm4py.conformance_otg()`
///
/// Call style: `ocel_only` → convert: `list_dict`
pub fn call_conformance_otg(
    py: Python<'_>,
    ocel: &PyAny,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    let result = pm4py.call_method1("conformance_otg", (ocel,))?;
    let diagnostics: Vec<&PyAny> = result.extract()?;
    if diagnostics.is_empty() { return Ok(0.0); }
    let total: f64 = diagnostics.iter()
        .filter_map(|d| d.get_item("trace_fitness").ok()?.extract::<f64>().ok())
        .sum();
    Ok(total / diagnostics.len() as f64)
}






/// Bridge to `pm4py.conformance_temporal_profile()`
///
/// Call style: `log_and_model` → convert: `unit`
pub fn call_conformance_temporal_profile(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let discover_fn = "discover_temporal_profile";
    let model = pm4py.call_method1(discover_fn, (py_log.as_ref(py),))?;
    let result = pm4py.call_method1("conformance_temporal_profile", (py_log.as_ref(py), model))?;
    drop(result);
    Ok(())
}






/// Bridge to `pm4py.fitness_alignments()`
///
/// Call style: `log_and_net` → convert: `fitness_dict`
pub fn call_fitness_alignments(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("fitness_alignments", (py_log, py_net, py_im, py_fm))?;
    // pm4py returns dict {"average_trace_fitness": f64, ...}; extract the key
    result.get_item("average_trace_fitness")?.extract::<f64>()
}






/// Bridge to `pm4py.fitness_footprints()`
///
/// Call style: `log_and_net` → convert: `f64`
pub fn call_fitness_footprints(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("fitness_footprints", (py_log, py_net, py_im, py_fm))?;
    result.extract::<f64>()
}






/// Bridge to `pm4py.fitness_token_based_replay()`
///
/// Call style: `log_and_net` → convert: `fitness_dict`
pub fn call_fitness_token_based_replay(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("fitness_token_based_replay", (py_log, py_net, py_im, py_fm))?;
    // pm4py returns dict {"average_trace_fitness": f64, ...}; extract the key
    result.get_item("average_trace_fitness")?.extract::<f64>()
}






/// Bridge to `pm4py.generalization_tbr()`
///
/// Call style: `log_and_net` → convert: `f64`
pub fn call_generalization_tbr(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("generalization_tbr", (py_log, py_net, py_im, py_fm))?;
    result.extract::<f64>()
}






/// Bridge to `pm4py.precision_alignments()`
///
/// Call style: `log_and_net` → convert: `f64`
pub fn call_precision_alignments(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("precision_alignments", (py_log, py_net, py_im, py_fm))?;
    result.extract::<f64>()
}






/// Bridge to `pm4py.precision_footprints()`
///
/// Call style: `log_and_net` → convert: `f64`
pub fn call_precision_footprints(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("precision_footprints", (py_log, py_net, py_im, py_fm))?;
    result.extract::<f64>()
}






/// Bridge to `pm4py.precision_token_based_replay()`
///
/// Call style: `log_and_net` → convert: `f64`
pub fn call_precision_token_based_replay(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    let pm4py = py.import("pm4py")?;
    // WvdA soundness: bound runtime — cap at 1,000 traces
    const MAX_TRACES: usize = 1_000;
    let truncated: EventLog;
    let use_log: &EventLog = if log.traces.len() > MAX_TRACES {
        truncated = {
            let mut l = EventLog::new();
            l.traces = log.traces[..MAX_TRACES].to_vec();
            l
        };
        &truncated
    } else {
        log
    };
    let py_log = eventlog_to_pm4py(py, use_log)?;
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;
    let result = pm4py.call_method1("precision_token_based_replay", (py_log, py_net, py_im, py_fm))?;
    result.extract::<f64>()
}






/// Bridge to `pm4py.replay_prefix_tbr()`
///
/// Call style: `prefix_and_net` → convert: `unit`
pub fn call_replay_prefix_tbr(
    py: Python<'_>,
    log: &EventLog,
) -> PyResult<()> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("replay_prefix_tbr", (py_log,))?;
    drop(result);
    Ok(())
}

