use std::collections::HashMap;
use std::fmt;

/// Statistics about cache performance
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub entries: usize,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
}

/// LRU (Least Recently Used) cache for process mining results
///
/// Maintains up to `capacity` entries, evicting the least recently used
/// entry when capacity is exceeded.
pub struct LRUCache {
    capacity: usize,
    cache: HashMap<String, String>,
    access_order: Vec<String>,
    stats: CacheStats,
}

impl LRUCache {
    /// Create a new LRU cache with the specified capacity
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "LRU cache capacity must be greater than 0. Tip: use LRUCache::new(100) or higher. See docs/TROUBLESHOOTING.md#cache-initialization for guidance");

        LRUCache {
            capacity,
            cache: HashMap::with_capacity(capacity),
            access_order: Vec::with_capacity(capacity),
            stats: CacheStats {
                entries: 0,
                capacity,
                hits: 0,
                misses: 0,
            },
        }
    }

    /// Get a value from the cache, updating its recency
    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.cache.get(key) {
            // Update access order - move to end (most recent)
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key.to_string());

            self.stats.hits += 1;
            Some(value.clone())
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a key-value pair into the cache
    pub fn insert(&mut self, key: String, value: String) {
        // If key already exists, update it and move to most recent
        if self.cache.contains_key(&key) {
            self.cache.insert(key.clone(), value);
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key);
            return;
        }

        // If at capacity, evict least recently used
        if self.cache.len() >= self.capacity {
            if let Some(lru_key) = self.access_order.first() {
                let lru_key = lru_key.clone();
                self.cache.remove(&lru_key);
                self.access_order.remove(0);
            }
        }

        // Insert new entry
        self.cache.insert(key.clone(), value);
        self.access_order.push(key);
        self.stats.entries = self.cache.len();
    }

    /// Remove a specific key from the cache
    pub fn remove(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.cache.remove(key) {
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.stats.entries = self.cache.len();
            Some(value)
        } else {
            None
        }
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.stats.entries = 0;
    }

    /// Get current number of entries in the cache
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache contains a key (without updating recency)
    pub fn contains_key(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            capacity: self.capacity,
            hits: self.stats.hits,
            misses: self.stats.misses,
        }
    }

    /// Get hit rate as a percentage (0.0 to 100.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total == 0 {
            0.0
        } else {
            (self.stats.hits as f64 / total as f64) * 100.0
        }
    }
}

impl fmt::Debug for LRUCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LRUCache")
            .field("capacity", &self.capacity)
            .field("size", &self.cache.len())
            .field("stats", &self.stats)
            .finish()
    }
}

impl Clone for LRUCache {
    fn clone(&self) -> Self {
        LRUCache {
            capacity: self.capacity,
            cache: self.cache.clone(),
            access_order: self.access_order.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_basic_operations() {
        let mut cache = LRUCache::new(3);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());

        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
        assert_eq!(cache.size(), 3);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = LRUCache::new(3);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());

        // Access key1 to mark it as recently used
        let _ = cache.get("key1");

        // Add new key, should evict key2 (least recently used)
        cache.insert("key4".to_string(), "value4".to_string());

        assert_eq!(cache.size(), 3);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), None);
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
        assert_eq!(cache.get("key4"), Some("value4".to_string()));
    }

    #[test]
    fn test_lru_capacity_boundary() {
        let mut cache = LRUCache::new(1000);

        // Fill to capacity
        for i in 0..1000 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }

        assert_eq!(cache.size(), 1000);

        // Add one more - should evict the least recently used
        cache.insert("key1000".to_string(), "value1000".to_string());
        assert_eq!(cache.size(), 1000);

        // The oldest key should be gone
        assert_eq!(cache.get("key0"), None);
        assert_eq!(cache.get("key1000"), Some("value1000".to_string()));
    }

    #[test]
    fn test_lru_lookup_updates_recency() {
        let mut cache = LRUCache::new(3);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());

        // Access key1 multiple times - should make it most recent
        for _ in 0..5 {
            let _ = cache.get("key1");
        }

        // Add new key - should evict key2 or key3, not key1
        cache.insert("key4".to_string(), "value4".to_string());

        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key4"), Some("value4".to_string()));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = LRUCache::new(10);

        for i in 0..5 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }

        // Hits
        for i in 0..5 {
            let _ = cache.get(&format!("key{}", i));
        }

        // Misses
        for i in 5..8 {
            let _ = cache.get(&format!("key{}", i));
        }

        let stats = cache.stats();
        assert_eq!(stats.entries, 5);
        assert!(stats.entries <= 10);
        assert!(stats.hits >= 5);
        assert!(stats.misses >= 3);
    }
}
