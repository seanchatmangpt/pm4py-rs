//! Process discovery algorithms
//!
//! This module provides algorithms for automatically discovering process models from event logs.
//! Each algorithm extracts a different type of process model (Petri net, process tree, DFG, etc.)
//! using different techniques.
//!
//! # Discovery Algorithms
//!
//! ## Alpha Miner
//!
//! A fast, simple algorithm that discovers Petri nets by analyzing causal relations.
//! Best for: Simple processes without noise.
//!
//! ```rust
//! use pm4py::{AlphaMiner, DiscoveryAlgorithm, EventLog};
//!
//! let event_log = EventLog::new();
//! let miner = AlphaMiner::new();
//! let petri_net = miner.discover(&event_log);
//! ```
//!
//! ## Inductive Miner
//!
//! A robust, recursive discovery algorithm that handles complex patterns (loops, concurrency).
//! The most widely used algorithm in practice.
//!
//! ```rust,no_run
//! use pm4py::{InductiveMiner, DiscoveryAlgorithm, EventLog};
//!
//! let event_log = EventLog::new();
//! let miner = InductiveMiner::new().with_min_support(0.1);
//! let petri_net = miner.discover(&event_log);
//! ```
//!
//! ## Heuristic Miner
//!
//! Frequency-based discovery that handles noise well. Uses dependency metrics to determine
//! which relationships are significant.
//!
//! ```rust,no_run
//! use pm4py::{HeuristicMiner, DiscoveryAlgorithm, EventLog};
//!
//! let event_log = EventLog::new();
//! let miner = HeuristicMiner::new()
//!     .with_dependency_threshold(0.9);
//! let petri_net = miner.discover(&event_log);
//! ```
//!
//! ## DFG Miner
//!
//! Discovers a Directly-Follows Graph (DFG) - the simplest process model showing
//! which activities directly follow each other.
//!
//! ```rust
//! use pm4py::{DFGMiner, EventLog};
//!
//! let event_log = EventLog::new();
//! let miner = DFGMiner::new();
//! let dfg = miner.discover(&event_log);
//! ```
//!
//! ## Split Miner
//!
//! Ultra-fast discovery algorithm designed for large event logs.
//! Uses an exponential time decay filter to handle noise.
//!
//! ## Log Skeleton
//!
//! Discovers a set of constraints that describe the behavior in the log.
//! Useful for conformance checking and query answering.
//!
//! ## Organizational Mining
//!
//! Algorithms for discovering organizational patterns:
//! - Handover of work networks
//! - Working together networks
//! - Organizational roles
//! - Subcontracting networks
//!
//! # Choosing an Algorithm
//!
//! | Algorithm | Speed | Noise Tolerance | Model Quality | Best For |
//! |-----------|-------|------------------|---------------|----------|
//! | Alpha Miner | ⚡ Fast | Low | Simple models | Teaching, simple logs |
//! | Inductive Miner | Medium | High | Sound models | Production use |
//! | Heuristic Miner | Medium | High | Realistic | Noisy logs |
//! | Split Miner | ⚡⚡ Very Fast | High | Sound models | Large logs |
//! | DFG Miner | ⚡⚡ Very Fast | Medium | Simple | Quick analysis |
//!
//! # Trait
//!
//! All discovery miners implement the [`DiscoveryAlgorithm`] trait:
//!
//! ```rust,ignore
//! pub trait DiscoveryAlgorithm {
//!     fn discover(&self, log: &EventLog) -> PetriNet;
//! }
//! ```

pub mod alpha_miner;
pub mod alpha_plus;
pub mod causal_net_miner;
pub mod decision_mining;
pub mod declare_miner;
pub mod dfg_miner;
pub mod dfg_miner_extended;
pub mod extended_discovery;
pub mod graphs;
pub mod heuristic_miner;
pub mod ilp_miner;
pub mod inductive_miner;
pub mod log_skeleton;
pub mod organizational;
pub mod prefix_tree;
pub mod split_miner;
pub mod streaming_miner;
pub mod token_miner;
pub mod transition_system;
pub mod tree_miner;
pub mod variants;

pub use alpha_miner::AlphaMiner;
pub use alpha_plus::AlphaPlusMiner;
pub use causal_net_miner::CausalNetMiner;
pub use decision_mining::{gini_impurity, mine_decision_rules, DecisionModel, DecisionRule};
pub use declare_miner::{
    conformance_declare, get_declare_constraint_templates, DeclareConstraint, DeclareMiner,
    DeclareModel,
};
pub use dfg_miner::DFGMiner;
pub use dfg_miner_extended::DFGMinerExtended;
pub use extended_discovery::*;
pub use graphs::{
    directly_follows_graph, discover_performance_dfg, eventually_follows_graph,
    EventuallyFollowsGraph, PerformanceDFG, PerformanceMetrics,
};
pub use heuristic_miner::HeuristicMiner;
pub use ilp_miner::ILPMiner;
pub use inductive_miner::InductiveMiner;
pub use log_skeleton::{
    conformance_log_skeleton, LogSkeleton, LogSkeletonConformanceResult, LogSkeletonMiner,
};
pub use organizational::{
    discover_activity_based_resource_similarity, discover_handover_of_work_network,
    discover_network_analysis, discover_organizational_roles, discover_subcontracting_network,
    discover_working_together_network, OrganizationalNetworkMetrics,
};
pub use prefix_tree::{
    discover_prefix_tree, filter_log_by_variants, get_variants_from_log, get_variants_top_k,
    PrefixTree, PrefixTreeNode,
};
pub use split_miner::SplitMiner;
pub use streaming_miner::StreamingMiner;
pub use token_miner::TokenMiner;
pub use transition_system::{
    discover_annotated_transition_system, discover_transition_system, AnnotatedTransitionSystem,
    TSState, TSTransition, TransitionSystem,
};
pub use tree_miner::TreeMiner;
pub use variants::{
    FilterStrategy, Variant, VariantAnalysis, VariantFilter, VariantFingerprint, VariantInfo,
    VariantMetrics, VariantSimilarity,
};

use crate::log::EventLog;
use crate::models::PetriNet;

/// Trait for process discovery algorithms
///
/// All discovery miners implement this trait, providing a unified interface
/// for discovering process models from event logs.
///
/// # Example
///
/// ```rust
/// use pm4py::{DiscoveryAlgorithm, AlphaMiner, EventLog};
///
/// fn discover_and_analyze(event_log: &EventLog) {
///     let miner = AlphaMiner::new();
///     let model = miner.discover(event_log);
///     // Analyze the model...
/// }
/// ```
///
/// # Implementations
///
/// - [`AlphaMiner`]: Fast, simple discovery
/// - [`InductiveMiner`]: Robust, sound discovery
/// - [`HeuristicMiner`]: Noise-tolerant discovery
/// - [`SplitMiner`]: Ultra-fast discovery
pub trait DiscoveryAlgorithm {
    /// Discover a process model from an event log
    ///
    /// # Parameters
    ///
    /// - `log`: The event log to analyze
    ///
    /// # Returns
    ///
    /// A discovered Petri net model
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pm4py::{DiscoveryAlgorithm, InductiveMiner, EventLog};
    ///
    /// let event_log = EventLog::new();
    /// let miner = InductiveMiner::new();
    /// let petri_net = miner.discover(&event_log);
    /// ```
    fn discover(&self, log: &EventLog) -> PetriNet;
}
