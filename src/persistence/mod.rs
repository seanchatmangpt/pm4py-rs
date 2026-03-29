//! Persistence layer for BOS ↔ BusinessOS synchronization
//!
//! This module provides PostgreSQL persistence capabilities for process mining models,
//! conformance results, and statistics. It enables bidirectional synchronization between
//! BOS (process mining engine) and BusinessOS (business logic layer).
//!
//! This module is only available when the `persistence` feature is enabled.

#[cfg(feature = "persistence")]
pub mod businessos_sync;

#[cfg(feature = "persistence")]
pub use businessos_sync::{
    ConformanceResult, DiscoverySession, PersistedModel, PersistenceClient, PersistenceError,
    PersistenceResult, ProcessStatistics,
};
