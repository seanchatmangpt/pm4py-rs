//! PyO3 bridge to the Python `pm4py` library.
//!
//! # DEPRECATED
//!
//! Use `crate::python::generated::*` instead.
//! Remaining caller: `src/conformance/alignments.rs` (tracked for future migration).
//! New callers should use the generated bridge in `src/python/generated/`.
//!
//! Functions here are only compiled when the `pm4py-bridge` feature is active.
//! They translate between Rust `EventLog`/`PetriNet`/`ProcessTree` types and
//! their Python pm4py equivalents using PyO3's GIL-based FFI.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::log::EventLog;
use crate::models::{Arc, PetriNet, Place, Transition};

// ─── EventLog → Python pm4py EventLog ────────────────────────────────────────

/// Convert a Rust `EventLog` to a Python pm4py `EventLog` object.
///
/// Builds a pandas `DataFrame` with the standard pm4py columns, then calls
/// `pm4py.format_dataframe` and `pm4py.convert_to_event_log`.
pub fn eventlog_to_pm4py(py: Python<'_>, log: &EventLog) -> PyResult<PyObject> {
    let pd = py.import("pandas")?;
    let pm4py = py.import("pm4py")?;

    // Build list of row dicts
    let rows = PyList::empty(py);
    for trace in &log.traces {
        for event in &trace.events {
            let row = PyDict::new(py);
            row.set_item("case:concept:name", trace.id.as_str())?;
            row.set_item("concept:name", event.activity.as_str())?;
            row.set_item("time:timestamp", event.timestamp.to_rfc3339())?;
            if let Some(ref resource) = event.resource {
                row.set_item("org:resource", resource.as_str())?;
            }
            rows.append(row)?;
        }
    }

    // pandas.DataFrame(rows)
    let df = pd.call_method1("DataFrame", (rows,))?;

    // pm4py.format_dataframe(df, case_id='case:concept:name',
    //                         activity_key='concept:name',
    //                         timestamp_key='time:timestamp')
    let kwargs = PyDict::new(py);
    kwargs.set_item("case_id", "case:concept:name")?;
    kwargs.set_item("activity_key", "concept:name")?;
    kwargs.set_item("timestamp_key", "time:timestamp")?;
    let df_formatted = pm4py.call_method("format_dataframe", (df,), Some(kwargs))?;

    // pm4py.convert_to_event_log(df_formatted)
    let py_log = pm4py.call_method1("convert_to_event_log", (df_formatted,))?;
    Ok(py_log.into())
}

// ─── Python PetriNet → Rust PetriNet ─────────────────────────────────────────

/// Convert a Python pm4py `PetriNet` (plus initial/final markings) to a Rust
/// `PetriNet`.
///
/// `py_net`  — the `pm4py.objects.petri_net.obj.PetriNet` object
/// `py_im`   — the initial marking dict (`{Place: tokens}`)
/// `py_fm`   — the final marking dict (`{Place: tokens}`)
pub fn pm4py_petri_to_rust(
    _py: Python<'_>,
    py_net: &PyAny,
    py_im: &PyAny,
    py_fm: &PyAny,
) -> PyResult<PetriNet> {
    let mut net = PetriNet::new();

    // — Places ——————————————————————————————————————————————————————————————
    // py_net.places is a frozenset of Place objects; each has `.name`
    let places_set = py_net.getattr("places")?;
    let place_list: Vec<&PyAny> = places_set.iter()?.collect::<PyResult<_>>()?;

    // Collect initial / final place names from markings
    let im_keys: Vec<String> = py_im
        .call_method0("keys")?
        .iter()?
        .map(|p| p.and_then(|p| p.getattr("name")?.extract::<String>()))
        .collect::<PyResult<_>>()?;
    let fm_keys: Vec<String> = py_fm
        .call_method0("keys")?
        .iter()?
        .map(|p| p.and_then(|p| p.getattr("name")?.extract::<String>()))
        .collect::<PyResult<_>>()?;

    // Map Python place id (str(place)) → Rust Place.id (use place.name)
    for py_place in &place_list {
        let name: String = py_place.getattr("name")?.extract()?;
        let mut place = Place::new(name.clone());
        // Use name as stable id for arc matching
        place.id = name.clone();
        if im_keys.contains(&name) {
            place.initial_marking = 1;
        }
        net.add_place(place);
    }
    if let Some(init_name) = im_keys.first() {
        net.set_initial_place(init_name.clone());
    }
    if let Some(final_name) = fm_keys.first() {
        net.set_final_place(final_name.clone());
    }

    // — Transitions ————————————————————————————————————————————————————————
    // py_net.transitions: frozenset; each has `.name` (label) and `.label`
    let transitions_set = py_net.getattr("transitions")?;
    let trans_list: Vec<&PyAny> = transitions_set.iter()?.collect::<PyResult<_>>()?;

    for py_trans in &trans_list {
        let name: String = py_trans.getattr("name")?.extract()?;
        let label_obj = py_trans.getattr("label")?;
        let mut trans = Transition::new(name.clone());
        trans.id = name.clone();
        if !label_obj.is_none() {
            let label: String = label_obj.extract()?;
            trans = trans.with_label(label);
        }
        net.add_transition(trans);
    }

    // — Arcs ———————————————————————————————————————————————————————————————
    // py_net.arcs: frozenset of Arc objects; each has `.source` and `.target`
    let arcs_set = py_net.getattr("arcs")?;
    for py_arc in arcs_set.iter()? {
        let py_arc = py_arc?;
        let source = py_arc.getattr("source")?;
        let target = py_arc.getattr("target")?;

        // Determine whether source/target is Place or Transition by checking
        // for the `.label` attribute (transitions have it, places don't).
        let src_name: String = source.getattr("name")?.extract()?;
        let tgt_name: String = target.getattr("name")?.extract()?;

        let weight: usize = py_arc
            .getattr("weight")
            .and_then(|w| w.extract())
            .unwrap_or(1);

        net.add_arc(Arc::new(src_name, tgt_name).with_weight(weight));
    }

    Ok(net)
}

// ─── Discover via bridge ──────────────────────────────────────────────────────

/// Call `pm4py.discover_petri_net_inductive(log)` and return the Rust PetriNet.
pub fn call_discover_inductive(py: Python<'_>, log: &EventLog) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_inductive", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}

/// Call `pm4py.discover_petri_net_ilp(log)` and return the Rust PetriNet.
pub fn call_discover_ilp(py: Python<'_>, log: &EventLog) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_ilp", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}

/// Call `pm4py.discover_petri_net_heuristics(log)` and return the Rust PetriNet.
pub fn call_discover_heuristics(py: Python<'_>, log: &EventLog) -> PyResult<PetriNet> {
    let pm4py = py.import("pm4py")?;
    let py_log = eventlog_to_pm4py(py, log)?;
    let result = pm4py.call_method1("discover_petri_net_heuristics", (py_log,))?;
    let net_obj: &PyAny = result.get_item(0)?;
    let im_obj: &PyAny = result.get_item(1)?;
    let fm_obj: &PyAny = result.get_item(2)?;
    pm4py_petri_to_rust(py, net_obj, im_obj, fm_obj)
}

// ─── Conformance via bridge ───────────────────────────────────────────────────

/// Call `pm4py.conformance_diagnostics_token_based_replay(log, net, im, fm)`
/// and return average fitness.
///
/// Guards at 1 000 traces to bound runtime for large logs.
#[deprecated(
    note = "Use `crate::python::generated::conformance::call_conformance_diagnostics_alignments` instead"
)]
pub fn call_conformance_alignments(
    py: Python<'_>,
    log: &EventLog,
    net: &PetriNet,
) -> PyResult<f64> {
    const MAX_TRACES: usize = 1_000;

    let pm4py = py.import("pm4py")?;

    // Truncate log if needed
    let truncated: EventLog;
    let use_log = if log.traces.len() > MAX_TRACES {
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

    // Rebuild a fresh Python PetriNet from the Rust net
    let (py_net, py_im, py_fm) = rust_petri_to_pm4py(py, net)?;

    let result = pm4py.call_method1(
        "conformance_diagnostics_token_based_replay",
        (py_log, py_net, py_im, py_fm),
    )?;

    // result is a list of dicts; average the 'trace_fitness' field
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
        })
        .sum();
    Ok(total / diagnostics.len() as f64)
}

// ─── Rust PetriNet → Python PetriNet ─────────────────────────────────────────

/// Convert a Rust `PetriNet` to a Python pm4py `(PetriNet, im, fm)` triple.
fn rust_petri_to_pm4py(py: Python<'_>, net: &PetriNet) -> PyResult<(PyObject, PyObject, PyObject)> {
    let pn_module = py.import("pm4py.objects.petri_net.obj")?;
    let marking_module = py.import("pm4py.objects.petri_net.obj")?;

    let py_net = pn_module.call_method0("PetriNet")?;
    let py_im = marking_module.call_method0("Marking")?;
    let py_fm = marking_module.call_method0("Marking")?;

    // Add places
    let mut place_map: std::collections::HashMap<&str, PyObject> = std::collections::HashMap::new();
    for place in &net.places {
        let py_place = pn_module
            .getattr("PetriNet")?
            .getattr("Place")?
            .call1((place.name.as_str(),))?;
        py_net.getattr("places")?.call_method1("add", (py_place,))?;
        if net.initial_place.as_deref() == Some(place.id.as_str()) {
            py_im.set_item(py_place, 1)?;
        }
        if net.final_place.as_deref() == Some(place.id.as_str()) {
            py_fm.set_item(py_place, 1)?;
        }
        place_map.insert(place.id.as_str(), py_place.into());
    }

    // Add transitions
    let mut trans_map: std::collections::HashMap<&str, PyObject> = std::collections::HashMap::new();
    for trans in &net.transitions {
        let label = trans.label.as_deref().into_py(py);
        let py_trans = pn_module
            .getattr("PetriNet")?
            .getattr("Transition")?
            .call1((trans.name.as_str(), label))?;
        py_net
            .getattr("transitions")?
            .call_method1("add", (py_trans,))?;
        trans_map.insert(trans.id.as_str(), py_trans.into());
    }

    // Add arcs
    let utils = py.import("pm4py.objects.petri_net.utils.petri_utils")?;
    for arc in &net.arcs {
        // Determine if arc.from is a place or transition by looking up both maps
        let (src, tgt): (PyObject, PyObject) = if let Some(p) = place_map.get(arc.from.as_str()) {
            if let Some(t) = trans_map.get(arc.to.as_str()) {
                (p.clone_ref(py), t.clone_ref(py))
            } else {
                continue; // dangling arc — skip
            }
        } else if let Some(t) = trans_map.get(arc.from.as_str()) {
            if let Some(p) = place_map.get(arc.to.as_str()) {
                (t.clone_ref(py), p.clone_ref(py))
            } else {
                continue;
            }
        } else {
            continue;
        };
        utils.call_method1("add_arc_from_to", (src, tgt, py_net))?;
    }

    Ok((py_net.into(), py_im.into(), py_fm.into()))
}
