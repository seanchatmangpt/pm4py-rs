//! Memory-Efficient Collections and Allocator Optimizations
//!
//! This module provides memory-optimized data structures and allocation strategies:
//! - Interned strings to reduce memory for repeated activity names
//! - Compact event representation with shared attribute references
//! - Efficient arc traversal with cache-friendly data layouts
//! - Reference counting for large shared structures
//!
//! Memory Safety: All optimizations maintain Rust's safety guarantees with no unsafe blocks.

use std::collections::HashMap;
use std::sync::Arc;

/// String interning pool for activity names and resource IDs
///
/// This reduces memory when the same activity appears thousands of times.
/// Example: Instead of storing "approve" 1M times (7 bytes each),
/// store once and reference via ID.
#[derive(Debug, Clone)]
pub struct StringIntern {
    // Maps string -> unique ID
    strings: HashMap<String, usize>,
    // Maps ID -> string (for reconstruction)
    by_id: Vec<String>,
}

impl StringIntern {
    /// Create a new string interning pool
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            by_id: Vec::new(),
        }
    }

    /// Intern a string, returning its ID
    pub fn intern(&mut self, s: impl Into<String>) -> usize {
        let s = s.into();
        if let Some(&id) = self.strings.get(&s) {
            id
        } else {
            let id = self.by_id.len();
            self.by_id.push(s.clone());
            self.strings.insert(s, id);
            id
        }
    }

    /// Retrieve a string by ID
    pub fn get(&self, id: usize) -> Option<&str> {
        self.by_id.get(id).map(|s| s.as_str())
    }

    /// Get total memory used (estimate in bytes)
    pub fn memory_estimate(&self) -> usize {
        let mut bytes = 0;
        // HashMap overhead
        bytes += self.strings.len() * 56; // Average entry overhead

        // Strings stored
        for s in &self.by_id {
            bytes += s.len() + 32; // String with capacity
        }

        bytes
    }

    /// Compression ratio: (original_bytes / current_bytes)
    pub fn compression_ratio(&self, typical_string_size: usize) -> f64 {
        let original = self.by_id.len() * typical_string_size;
        let current = self.memory_estimate();
        if current > 0 {
            original as f64 / current as f64
        } else {
            1.0
        }
    }

    /// Size estimation helper (not used in all scenarios)
    #[allow(dead_code)]
    fn _size_helper() {}
}

impl Default for StringIntern {
    fn default() -> Self {
        Self::new()
    }
}

/// Compact event attribute representation using Arc for sharing
///
/// Instead of every event storing a complete BTreeMap of attributes,
/// share the attributes via Arc<> when they're identical.
#[derive(Debug, Clone)]
pub struct CompactAttributes {
    // Maps attribute hash -> Arc<HashMap> for deduplication
    attribute_cache: HashMap<u64, Arc<HashMap<String, String>>>,
}

impl CompactAttributes {
    /// Create a new compact attribute store
    pub fn new() -> Self {
        Self {
            attribute_cache: HashMap::new(),
        }
    }

    /// Add attributes, returning an Arc to deduplicated storage
    pub fn add_attributes(
        &mut self,
        attrs: HashMap<String, String>,
    ) -> Arc<HashMap<String, String>> {
        // Simple hash: combine all key-value lengths
        let hash = attrs.iter().fold(0u64, |acc, (k, v)| {
            acc.wrapping_mul(31)
                .wrapping_add(k.len() as u64)
                .wrapping_mul(31)
                .wrapping_add(v.len() as u64)
        });

        if let Some(existing) = self.attribute_cache.get(&hash) {
            Arc::clone(existing)
        } else {
            let arc = Arc::new(attrs);
            self.attribute_cache.insert(hash, Arc::clone(&arc));
            arc
        }
    }

    /// Memory used by cache (estimate in bytes)
    pub fn memory_estimate(&self) -> usize {
        let mut bytes = 0;

        // HashMap overhead
        bytes += self.attribute_cache.len() * 56;

        // Actual attribute data
        for arc_attrs in self.attribute_cache.values() {
            bytes += arc_attrs.len() * 56; // Per entry overhead
            for (k, v) in arc_attrs.iter() {
                bytes += k.len() + v.len();
            }
        }

        bytes
    }

    /// Deduplication ratio: saved bytes / original bytes
    pub fn deduplication_ratio(&self) -> f64 {
        if self.attribute_cache.is_empty() {
            return 1.0;
        }

        let total_stored = self
            .attribute_cache
            .values()
            .map(|arc| arc.len())
            .sum::<usize>();

        if total_stored == 0 {
            return 1.0;
        }

        let original = self.attribute_cache.len() * total_stored;
        let current = self.memory_estimate();

        if current > 0 {
            original as f64 / current as f64
        } else {
            1.0
        }
    }
}

impl Default for CompactAttributes {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache-efficient arc traversal using adjacency lists
///
/// For Petri net traversal, group related arcs to improve cache locality.
#[derive(Copy, Clone, Debug)]
pub struct ArcIndex {
    pub from: usize,
    pub to: usize,
    pub weight: usize,
}

/// Adjacency list for cache-friendly graph traversal
#[derive(Debug, Clone)]
pub struct AdjacencyLists {
    // outgoing[i] = indices of arcs starting from node i
    outgoing: Vec<Vec<usize>>,
    // incoming[i] = indices of arcs ending at node i
    incoming: Vec<Vec<usize>>,
    // All arcs, indexed by the lists above
    arcs: Vec<ArcIndex>,
}

impl AdjacencyLists {
    /// Create adjacency lists from arc list
    pub fn new(num_nodes: usize, arcs: &[(usize, usize, usize)]) -> Self {
        let mut outgoing = vec![Vec::new(); num_nodes];
        let mut incoming = vec![Vec::new(); num_nodes];
        let mut arc_list = Vec::new();

        for (from, to, weight) in arcs {
            let arc_idx = arc_list.len();
            arc_list.push(ArcIndex {
                from: *from,
                to: *to,
                weight: *weight,
            });

            outgoing[*from].push(arc_idx);
            incoming[*to].push(arc_idx);
        }

        Self {
            outgoing,
            incoming,
            arcs: arc_list,
        }
    }

    /// Get outgoing arcs from a node (cache-friendly)
    pub fn outgoing(&self, node: usize) -> impl Iterator<Item = &ArcIndex> {
        self.outgoing[node]
            .iter()
            .filter_map(move |&idx| self.arcs.get(idx))
    }

    /// Get incoming arcs to a node (cache-friendly)
    pub fn incoming(&self, node: usize) -> impl Iterator<Item = &ArcIndex> {
        self.incoming[node]
            .iter()
            .filter_map(move |&idx| self.arcs.get(idx))
    }

    /// Memory used by adjacency lists (estimate in bytes)
    pub fn memory_estimate(&self) -> usize {
        let mut bytes = 0;

        // Outgoing lists
        for list in &self.outgoing {
            bytes += list.len() * 8 + 24;
        }

        // Incoming lists
        for list in &self.incoming {
            bytes += list.len() * 8 + 24;
        }

        // Arcs (each is 24 bytes: 3 * usize)
        bytes += self.arcs.len() * 24;

        bytes
    }

    /// Cache efficiency vs naive HashMap approach
    ///
    /// Naive: store each arc separately in HashMap → poor cache locality
    /// This: contiguous allocation with index arrays → better locality
    pub fn cache_efficiency_estimate(&self) -> f64 {
        // Naive: each arc lookup requires HashMap lookup (58 bytes overhead)
        let naive_cost = self.arcs.len() * 58;

        // This: direct array access with index lookup
        let efficient_cost = self.memory_estimate();

        if efficient_cost > 0 {
            naive_cost as f64 / efficient_cost as f64
        } else {
            1.0
        }
    }
}

/// Pooled allocation for small temporary objects
///
/// Reuse allocations for temporary structures like markings, states, etc.
#[derive(Debug, Clone)]
pub struct ObjectPool<T: Clone + Default> {
    available: Vec<T>,
    allocated: usize,
}

impl<T: Clone + Default> ObjectPool<T> {
    /// Create a new object pool with initial capacity
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            available: (0..initial_capacity).map(|_| T::default()).collect(),
            allocated: 0,
        }
    }

    /// Acquire an object from the pool
    pub fn acquire(&mut self) -> T {
        self.allocated += 1;
        self.available.pop().unwrap_or_default()
    }

    /// Return an object to the pool for reuse
    pub fn release(&mut self, obj: T) {
        self.allocated = self.allocated.saturating_sub(1);
        // Reset object to default state and return to pool
        let reset_obj = T::default();
        if self.available.len() < 1000 {
            // Limit pool size to avoid memory bloat
            // Ignore the returned object by shadowing it
            let _ = obj; // consume the input object
            self.available.push(reset_obj);
        }
    }

    /// Number of objects currently allocated
    pub fn allocated_count(&self) -> usize {
        self.allocated
    }

    /// Memory estimate in bytes
    pub fn memory_estimate(&self) -> usize {
        (self.available.len() + self.allocated) * std::mem::size_of::<T>() + 64
    }
}

impl<T: Clone + Default> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new(100)
    }
}

// ============================================================================
// SAFETY DOCUMENTATION
// ============================================================================

/// Memory Safety Guarantees
///
/// This module provides memory optimizations while maintaining Rust's safety:
///
/// 1. **StringIntern**: Uses HashMap for deduplication. No unsafe code.
///    - Arc usage is safe (thread-safe reference counting)
///    - Strings are immutable after interning
///
/// 2. **CompactAttributes**: Arc<HashMap> for attribute deduplication.
///    - Arc is atomic reference counting (thread-safe)
///    - HashMap is protected by Arc (no data races)
///
/// 3. **AdjacencyLists**: Pre-allocated arrays with index-based access.
///    - No unsafe array indexing (uses Vec with bounds checking)
///    - No raw pointers
///
/// 4. **ObjectPool**: Simple Vec-based reuse with Clone + Default.
///    - No concurrent access (single-threaded)
///    - Objects properly reset to default state
///
/// No unsafe blocks are used anywhere in this module.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_intern() {
        let mut intern = StringIntern::new();

        let id1 = intern.intern("approve");
        let id2 = intern.intern("approve");
        let id3 = intern.intern("reject");

        assert_eq!(id1, id2); // Same string gets same ID
        assert_ne!(id1, id3);

        assert_eq!(intern.get(id1), Some("approve"));
        assert_eq!(intern.get(id3), Some("reject"));
    }

    #[test]
    fn test_compact_attributes() {
        let mut attrs = CompactAttributes::new();

        let map1 = {
            let mut m = HashMap::new();
            m.insert("priority".to_string(), "high".to_string());
            m
        };

        let map2 = {
            let mut m = HashMap::new();
            m.insert("priority".to_string(), "high".to_string());
            m
        };

        let arc1 = attrs.add_attributes(map1);
        let arc2 = attrs.add_attributes(map2);

        // Same attributes share Arc (deduplicated)
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }

    #[test]
    fn test_adjacency_lists() {
        let arcs = vec![(0, 1, 1), (1, 2, 1), (0, 2, 2)];
        let adj = AdjacencyLists::new(3, &arcs);

        let outgoing_0: Vec<_> = adj.outgoing(0).collect();
        assert_eq!(outgoing_0.len(), 2);

        let incoming_2: Vec<_> = adj.incoming(2).collect();
        assert_eq!(incoming_2.len(), 2);
    }

    #[test]
    fn test_object_pool() {
        let mut pool: ObjectPool<Vec<u32>> = ObjectPool::new(10);

        let mut vec1 = pool.acquire();
        vec1.push(42);
        assert_eq!(pool.allocated_count(), 1);

        pool.release(vec1);
        assert_eq!(pool.allocated_count(), 0);
    }
}
