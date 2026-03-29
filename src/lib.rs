//! PM4Py - A Rust Process Mining Library
//!
//! A comprehensive Rust implementation of process mining algorithms, inspired by the Python pm4py library.
//! This library provides tools for discovering process models, checking conformance, analyzing performance,
//! and extracting insights from event logs.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use pm4py::{EventLog, AlphaMiner, TokenReplay};
//! use pm4py::io::CSVReader;
//! use std::path::Path;
//!
//! // Read an event log from CSV
//! let reader = CSVReader::new();
//! let log = reader.read(Path::new("event_log.csv")).unwrap();
//!
//! // Discover a process model using Alpha Miner
//! let miner = AlphaMiner::new();
//! let petri_net = miner.discover(&log);
//!
//! // Check conformance using token replay
//! let checker = TokenReplay::new();
//! let result = checker.check(&log, &petri_net);
//! println!("Fitness: {}", result.fitness);
//! ```
//!
//! # Core Concepts
//!
//! ## Event Logs
//!
//! Process mining starts with an **event log** - a collection of recorded executions of a business process.
//! Each execution is called a **trace** or **case**, and contains a sequence of **events**.
//!
//! ```rust
//! use pm4py::{Event, Trace, EventLog};
//! use chrono::Utc;
//!
//! // Create an event
//! let event = Event::new("Approve Request", Utc::now())
//!     .with_resource("John Doe")
//!     .with_attribute("outcome", "approved");
//!
//! // Create a trace (case)
//! let mut trace = Trace::new("case-001");
//! trace.add_event(event);
//!
//! // Create an event log
//! let mut log = EventLog::new();
//! log.add_trace(trace);
//! ```
//!
//! ## Process Discovery
//!
//! Discovery algorithms automatically extract a process model from an event log:
//!
//! - **Alpha Miner**: Fast, simple discovery for control-flow
//! - **Inductive Miner**: Robust discovery, handles complex patterns
//! - **Heuristic Miner**: Frequency-based discovery, handles noise
//! - **DFG Miner**: Directly-Follows Graph discovery
//! - **Split Miner**: Ultra-fast discovery for large logs
//!
//! ```rust,ignore
//! use pm4py::{InductiveMiner, DiscoveryAlgorithm, EventLog};
//!
//! let event_log = EventLog::new();
//! let miner = InductiveMiner::new();
//! let petri_net = miner.discover(&event_log);
//! ```
//!
//! ## Conformance Checking
//!
//! Conformance checking compares observed behavior (event log) with modeled behavior (process model):
//!
//! - **Token Replay**: Basic fitness checking
//! - **Alignments**: Optimal alignment-based conformance
//! - **Precision**: Measures model specificity
//! - **Generalization**: Measures model generalization
//! - **Four Spectrum**: Unified quality metric
//!
//! ## Organizational Mining
//!
//! Analyzes resource behavior and organizational patterns:
//!
//! - Handover of work networks
//! - Working together networks
//! - Organizational role discovery
//! - Resource similarity analysis
//!
//! ## Performance Analysis
//!
//! Extracts performance insights from event logs:
//!
//! - Cycle time analysis
//! - Bottleneck identification
//! - Resource utilization
//! - Case duration statistics
//!
//! # Modules
//!
//! - [`log`]: Event log structures and manipulation
//! - [`discovery`]: Process discovery algorithms (Alpha, Inductive, Heuristic, etc.)
//! - [`conformance`]: Conformance checking (Token Replay, Alignments, Precision, etc.)
//! - [`performance`]: Performance analysis utilities
//! - [`statistics`]: Statistical analysis of event logs
//! - [`predictive`]: Predictive process analytics
//! - [`io`]: Input/output for various log formats (XES, CSV, JSON, Parquet, OCEL)
//! - [`connectors`]: Enterprise data connectors (CSV, Webhook, SAP, Salesforce)
//! - [`ocpm`]: Object-Centric Process Mining
//! - [`visualization`]: Process model visualization (SVG, BPMN, Petri nets)
//! - [`monitoring`]: Drift detection and process monitoring
//! - [`models`]: Process model representations (Petri nets, Process trees, BPMN)
//! - [`utils`]: General utilities

/// A2A agent-to-agent protocol — exposes all 10 pm4py tools as A2A skills.
#[cfg(feature = "a2a")]
pub mod a2a;

pub mod audit;
pub mod board_kpis;
pub mod boardchair;
pub mod cache;
pub mod conformance;
pub mod connectors;
pub mod discovery;
pub mod errors;
pub mod http;
pub mod io;
// NOTE: middleware module requires actix-web, redis, dashmap dependencies
// Uncomment in lib.rs when these are added to Cargo.toml
// pub mod middleware;
pub mod jtbd;
pub mod llm;
pub mod log;
pub mod mcp;
pub mod memory;
pub mod metrics;
pub mod models;
pub mod monitoring;
pub mod observability;
pub mod ocpm;
pub mod optimization;
pub mod parity_verification;
pub mod performance;
#[cfg(feature = "persistence")]
pub mod persistence;
pub mod predictive;
pub mod python;
pub mod python_bridge;
pub mod remaining_parity;
pub mod semconv;
pub mod statistics;
pub mod telemetry;
pub mod tracing;
pub mod utils;
pub mod verification;
pub mod version;
pub mod visualization;
pub mod yawl;

pub use cache::LRUCache;
pub use conformance::{
    AlignmentChecker, ConformanceChecker, ConformanceResult, FootprintsConformanceChecker,
    FootprintsConformanceResult, Generalization, Precision, TokenReplay,
};
pub use discovery::{
    AlphaMiner, AlphaPlusMiner, DFGMiner, DiscoveryAlgorithm, FilterStrategy, HeuristicMiner,
    InductiveMiner, LogSkeleton, LogSkeletonMiner, TreeMiner, Variant, VariantAnalysis,
    VariantFilter, VariantFingerprint, VariantInfo, VariantMetrics, VariantSimilarity,
};
pub use log::{AdvancedFilter, Event, EventLog, FilterChain, FilterResult, Trace};
pub use models::causal_net::CausalNet;
pub use models::petri_net::PetriNet;
pub use models::{ActivityRelationship, BPMNDiagram, BPMNExecutor, BPMNXmlBuilder, Footprints};
pub use models::{ProcessTree, ProcessTreeNode, TreeOperator};
pub use observability::Tracing;
pub use ocpm::{
    OCPMDiscoveryMiner, Object, ObjectCentricConformanceResult, ObjectCentricEventLog,
    ObjectCentricPetriNet, ObjectCentricTokenReplay, ObjectType,
};
pub use predictive::{
    ActivityPrediction, CaseOutcome, NextActivityPredictor, OutcomePredictor,
    RemainingTimePrediction, RemainingTimePredictor, RiskAssessment,
};
pub use remaining_parity::*;
pub use statistics::{analyze_tree, TreeMetrics, TreePattern, TreeStatistics};
pub use version::{
    version_info, version_string, VERSION, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH,
};

/// Global configuration for pm4py-rust
///
/// # Example
///
/// ```rust
/// use pm4py::Config;
///
/// let config = Config::default();
/// ```
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Enable debug mode for verbose logging
    pub debug: bool,
}
