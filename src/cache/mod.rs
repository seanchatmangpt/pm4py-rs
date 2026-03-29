//! Caching module for process mining results
//!
//! Provides LRU cache for storing and retrieving process mining results
//! with automatic eviction of least recently used entries.

pub mod lru;

pub use lru::LRUCache;
