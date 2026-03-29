//! PyO3 bridge helpers — auto-generated, DO NOT EDIT
//! Source: pm4py-rust/ggen/templates/_helpers.rs.tera
//! Regenerate: cd pm4py-rust/ggen && ggen sync --from . --to ../src/python/generated
//!
//! Shared conversion utilities between Rust and Python pm4py types.
//! These are used by all domain bridge modules.

#![allow(dead_code)]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use chrono::Utc;
use std::collections::HashMap;

use crate::log::{Event, EventLog, Trace};
use crate::models::{PetriNet, Place, Transition, Arc};

// ─── EventLog → Python pm4py EventLog ────────────────────────────────────────

/// Convert a Rust `EventLog` to a Python pm4py `EventLog` object.
///
/// Builds a pandas `DataFrame` with standard pm4py columns, then calls
/// `pm4py.format_dataframe` and `pm4py.convert_to_event_log`.
pub fn eventlog_to_pm4py(py: Python<'_>, log: &EventLog) -> PyResult<PyObject> {
    let pd   = py.import("pandas")?;
    let pm4py = py.import("pm4py")?;

    let rows = PyList::empty(py);
    for trace in &log.traces {
        for event in &trace.events {
            let row = PyDict::new(py);
            row.set_item("case:concept:name", trace.id.as_str())?;
            row.set_item("concept:name",      event.activity.as_str())?;
            row.set_item("time:timestamp",    event.timestamp.to_rfc3339())?;
            if let Some(ref resource) = event.resource {
                row.set_item("org:resource", resource.as_str())?;
            }
            rows.append(row)?;
        }
    }

    let df = pd.call_method1("DataFrame", (rows,))?;

    let kwargs = PyDict::new(py);
    kwargs.set_item("case_id",       "case:concept:name")?;
    kwargs.set_item("activity_key",  "concept:name")?;
    kwargs.set_item("timestamp_key", "time:timestamp")?;
    let df_fmt = pm4py.call_method("format_dataframe", (df,), Some(kwargs))?;

    let py_log = pm4py.call_method1("convert_to_event_log", (df_fmt,))?;
    Ok(py_log.into())
}

// ─── Python pm4py EventLog → Rust EventLog ───────────────────────────────────

/// Convert a Python pm4py EventLog or DataFrame back to a Rust `EventLog`.
///
/// Calls `pm4py.convert_to_dataframe()` to normalize, then extracts rows
/// grouped by `case:concept:name`.
pub fn pm4py_to_eventlog(py: Python<'_>, py_log: &PyAny) -> PyResult<EventLog> {
    let pm4py = py.import("pm4py")?;
    let df = pm4py.call_method1("convert_to_dataframe", (py_log,))?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("orient", "records")?;
    let records: Vec<&PyAny> = df.call_method("to_dict", (), Some(kwargs))?.extract()?;
    let mut cases: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for rec in &records {
        let case_id: String = rec.get_item("case:concept:name")?.extract()?;
        let activity: String = rec.get_item("concept:name")?.extract()?;
        let ts: String = rec.get_item("time:timestamp")?.call_method0("isoformat")?.extract()?;
        cases.entry(case_id).or_default().push((activity, ts));
    }
    let mut log = EventLog::new();
    for (case_id, events) in cases {
        let mut trace = Trace::new(&case_id);
        for (activity, ts_str) in events {
            let dt = chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            trace.events.push(Event::new(&activity, dt));
        }
        log.traces.push(trace);
    }
    Ok(log)
}

// ─── Python PetriNet → Rust PetriNet ─────────────────────────────────────────

/// Convert a Python pm4py `(PetriNet, im, fm)` triple to a Rust `PetriNet`.
///
/// `py_net` — pm4py PetriNet object
/// `py_im`  — initial marking dict ({Place: tokens})
/// `py_fm`  — final marking dict   ({Place: tokens})
pub fn pm4py_petri_to_rust(
    _py: Python<'_>,
    py_net: &PyAny,
    py_im:  &PyAny,
    py_fm:  &PyAny,
) -> PyResult<PetriNet> {
    let mut net = PetriNet::new();

    // Places
    let places_set = py_net.getattr("places")?;
    let place_list: Vec<&PyAny> = places_set.iter()?.collect::<PyResult<_>>()?;

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

    for py_place in &place_list {
        let name: String = py_place.getattr("name")?.extract()?;
        let mut place = Place::new(name.clone());
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

    // Transitions
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

    // Arcs
    let arcs_set = py_net.getattr("arcs")?;
    for py_arc in arcs_set.iter()? {
        let py_arc = py_arc?;
        let source = py_arc.getattr("source")?;
        let target = py_arc.getattr("target")?;
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

// ─── Rust PetriNet → Python PetriNet ─────────────────────────────────────────

/// Convert a Rust `PetriNet` to a Python pm4py `(PetriNet, im, fm)` triple.
///
/// Used by conformance functions (log_and_net call style).
pub fn rust_petri_to_pm4py(
    py: Python<'_>,
    net: &PetriNet,
) -> PyResult<(PyObject, PyObject, PyObject)> {
    let pn_mod = py.import("pm4py.objects.petri_net.obj")?;

    let py_net = pn_mod.call_method0("PetriNet")?;
    let py_im  = pn_mod.call_method0("Marking")?;
    let py_fm  = pn_mod.call_method0("Marking")?;

    // Add places
    let mut place_map: std::collections::HashMap<&str, PyObject> =
        std::collections::HashMap::new();
    for place in &net.places {
        let py_place = pn_mod
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
    let mut trans_map: std::collections::HashMap<&str, PyObject> =
        std::collections::HashMap::new();
    for trans in &net.transitions {
        let label = trans.label.as_deref().into_py(py);
        let py_trans = pn_mod
            .getattr("PetriNet")?
            .getattr("Transition")?
            .call1((trans.name.as_str(), label))?;
        py_net.getattr("transitions")?.call_method1("add", (py_trans,))?;
        trans_map.insert(trans.id.as_str(), py_trans.into());
    }

    // Add arcs
    let utils = py.import("pm4py.objects.petri_net.utils.petri_utils")?;
    for arc in &net.arcs {
        let (src, tgt): (PyObject, PyObject) =
            if let Some(p) = place_map.get(arc.from.as_str()) {
                if let Some(t) = trans_map.get(arc.to.as_str()) {
                    (p.clone_ref(py), t.clone_ref(py))
                } else {
                    continue;
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
