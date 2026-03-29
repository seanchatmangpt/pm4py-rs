//! Memory optimization module
//!
//! Provides memory-efficient data structures and allocation strategies:
//! - String interning for repeated activity/resource names
//! - Compact attribute storage with deduplication via Arc
//! - Cache-friendly adjacency lists for graph traversal
//! - Object pooling for temporary allocations
//!
//! All optimizations maintain Rust's memory safety guarantees with zero unsafe code.

pub mod allocator;

pub use allocator::{AdjacencyLists, ArcIndex, CompactAttributes, ObjectPool, StringIntern};
