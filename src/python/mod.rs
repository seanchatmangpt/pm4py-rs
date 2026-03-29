//! Python bindings for pm4py using PyO3
//!
//! This module provides high-performance Python bindings to key pm4py-rust components,
//! enabling Python code to use Rust implementations for better performance.
//!
//! # PyO3 API note
//!
//! This module uses PyO3 0.21 APIs (`PyDict::new`, `PyList::new`, `Python::import`,
//! `PyList::empty`).  PyO3 0.21 marks these as deprecated in favour of `*_bound`
//! variants that will be the default in 0.22.  The suppression below is intentional
//! and scoped to this module only; it should be removed when PyO3 is upgraded to 0.22.
#![allow(deprecated)]
//!
//! # Examples
//!
//! ```python
//! from pm4py_rust import EventLog, AlphaMiner, InductiveMiner
//!
//! # Create event log
//! log = EventLog()
//! trace = log.add_trace("case_1")
//! trace.add_event("A", "2024-01-01T00:00:00Z")
//! trace.add_event("B", "2024-01-01T01:00:00Z")
//!
//! # Discover process model
//! miner = AlphaMiner()
//! net = miner.apply(log)
//! print(f"Places: {net.places_count()}")
//! print(f"Transitions: {net.transitions_count()}")
//! ```

pub mod conformance;
pub mod discovery;
pub mod event_log;
pub mod models;
#[cfg(feature = "pm4py-bridge")]
pub mod pm4py_bridge;
pub mod statistics;

pub mod generated;

pub use conformance::PyFootprintsConformanceChecker;
pub use discovery::{PyAlphaMiner, PyHeuristicMiner, PyInductiveMiner};
pub use event_log::PyEventLog;
pub use models::{PyPetriNet, PyProcessTree};
pub use statistics::PyLogStatistics;
