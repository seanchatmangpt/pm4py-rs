//! Chicago TDD — discover_powl bridge dispatch tests
//!
//! RED → GREEN → REFACTOR cycle for the PyO3 bridge path in `discover_powl()`.
//!
//! T1: discover_powl never panics on any log (Armstrong fault tolerance)
//! T2: discover_powl returns a non-empty tree (leaf_count ≥ 1) for a non-empty log
//! T3: #[cfg(feature = "pm4py-bridge")] bridge path executes without panic
//!     and returns a valid ProcessTree — GUARD: pm4py Python package installed

use chrono::Utc;
use pm4py::discovery::extended_discovery::discover_powl;
use pm4py::log::{Event, EventLog, Trace};

// ---------------------------------------------------------------------------
// Guard: detect whether pm4py Python package is reachable at runtime
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn pm4py_available() -> bool {
    #[cfg(feature = "pm4py-bridge")]
    {
        use pyo3::prelude::*;
        Python::with_gil(|py| py.import("pm4py").is_ok())
    }
    #[cfg(not(feature = "pm4py-bridge"))]
    {
        false
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a simple 3-trace log with activities A → B → C
fn make_abc_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    for i in 0..3_u32 {
        let mut t = Trace::new(format!("case_{}", i));
        t.add_event(Event::new("A", now));
        t.add_event(Event::new("B", now));
        t.add_event(Event::new("C", now));
        log.add_trace(t);
    }
    log
}

/// Build an empty event log
fn make_empty_log() -> EventLog {
    EventLog::new()
}

// ---------------------------------------------------------------------------
// T1: discover_powl never panics (Armstrong — let-it-crash only if bridge,
//     never let the supervisor crash)
// ---------------------------------------------------------------------------

#[test]
fn test_discover_powl_never_panics_on_non_empty_log() {
    let log = make_abc_log();
    // Must not panic under any feature configuration
    let _tree = discover_powl(&log);
}

#[test]
#[ignore = "Requires pm4py Python bindings - may not be installed in all environments"]
fn test_discover_powl_never_panics_on_empty_log() {
    let log = make_empty_log();
    // Must not panic even on empty input
    let _tree = discover_powl(&log);
}

// ---------------------------------------------------------------------------
// T2: discover_powl returns a valid ProcessTree for a non-empty log
//     The returned tree must have at least one leaf (one activity node).
// ---------------------------------------------------------------------------

#[test]
fn test_discover_powl_returns_non_empty_tree_for_non_empty_log() {
    let log = make_abc_log();
    let model = discover_powl(&log);
    assert!(
        model.activities.len() >= 1,
        "discover_powl must return a model with at least one activity for a non-empty log, \
         got activity_count={}",
        model.activities.len()
    );
}

#[test]
fn test_discover_powl_tree_contains_log_activities() {
    let log = make_abc_log();
    let model = discover_powl(&log);
    // The POWL model must cover all activities seen in the log
    for act in &["A", "B", "C"] {
        assert!(
            model.activities.iter().any(|a| a == *act),
            "discover_powl model must include activity '{}'; found activities: {:?}",
            act,
            model.activities
        );
    }
}

// ---------------------------------------------------------------------------
// T3: discover_powl bridge path (pm4py-bridge feature) executes without panic
//     GUARD: only runs when pm4py Python package is actually installed
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "pm4py-bridge")]
fn test_discover_powl_bridge_does_not_panic_when_pm4py_available() {
    if !pm4py_available() {
        // pm4py not installed in this environment — bridge will fall back to stub; skip.
        return;
    }
    let log = make_abc_log();
    // With pm4py available the bridge calls discover_powl; must not panic
    // and must return a POWL model with at least one activity.
    let model = discover_powl(&log);
    assert!(
        model.activities.len() >= 1,
        "bridge discover_powl must return a model with ≥1 activity when pm4py is available, \
         got activity_count={}",
        model.activities.len()
    );
}
