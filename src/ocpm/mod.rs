//! Object-Centric Process Mining (OCPM)
//!
//! This module provides comprehensive object-centric process mining capabilities,
//! enabling the discovery, analysis, and validation of multi-dimensional process models
//! that capture the lifecycle and interactions of multiple object types.
//!
//! # What is Object-Centric Process Mining?
//!
//! Traditional process mining focuses on case-level processes where a single process instance
//! flows through a linear sequence of activities. Object-centric process mining extends this
//! to handle processes where multiple object types participate, each with their own lifecycle,
//! and where objects can interact in complex ways.
//!
//! # Key Concepts
//!
//! ## Objects and Object Types
//! - **Object Type**: A classification of objects (e.g., "order", "item", "invoice")
//! - **Object**: An individual instance of an object type with its own identity and lifecycle
//! - **Lifecycle**: The sequence of states and events an object goes through
//!
//! ## Object-Centric Event Log
//! An event log that explicitly captures:
//! - Which objects are involved in each event
//! - The role of each object in the event
//! - The lifecycle stage of each object
//! - Relationships between objects
//!
//! ## Discovery
//! Object-centric discovery extracts:
//! - Individual lifecycles per object type
//! - Object interaction patterns
//! - Multi-dimensional Petri net models
//!
//! ## Conformance Checking
//! Validates whether:
//! - Objects follow expected lifecycle patterns
//! - Object interactions are valid
//! - Relationships between objects are consistent

pub mod object_conformance;
pub mod object_log;
pub mod oc_dfg;
pub mod oc_performance;
pub mod ocel_filters;
pub mod ocel_utils;
pub mod ocpm_miner;

pub use object_conformance::{
    ObjectCentricConformanceResult, ObjectCentricTokenReplay, ObjectConformanceResult,
    ObjectRelationshipValidator,
};
pub use object_log::{
    EventToObjectMapping, Object, ObjectCentricEventLog, ObjectRelationship, ObjectType,
    OcelTypedValue,
};
pub use oc_dfg::{discover_ocdfg, ActivityStats, EdgePerformance, ObjectCentricDFG};
pub use oc_performance::{
    compute_activity_waiting_times, compute_throughput_by_object_type, detect_bottlenecks,
    ActivityWaitStats, BottleneckReport, ThroughputStats,
};
pub use ocel_filters::*;
pub use ocel_utils::*;
pub use ocpm_miner::{OCArc, OCPMDiscoveryMiner, OCPlace, OCTransition, ObjectCentricPetriNet};
