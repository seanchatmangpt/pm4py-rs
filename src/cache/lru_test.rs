#[cfg(test)]
mod tests {
    use crate::cache::lru::LRUCache;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
    fn test_concurrent_access() {
        let cache = Arc::new(Mutex::new(LRUCache::new(1000)));
        let success_count = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        for thread_id in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let count_clone = Arc::clone(&success_count);

            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let key = format!("key_{}_{}", thread_id, i);
                    let value = format!("value_{}_{}", thread_id, i);

                    let mut cache = cache_clone.lock().expect("cache lock poisoned");
                    cache.insert(key.clone(), value.clone());

                    if let Some(retrieved) = cache.get(&key) {
                        if retrieved == value {
                            count_clone.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let cache_guard = cache.lock().expect("cache lock poisoned");
        let final_size = cache_guard.size();

        assert_eq!(success_count.load(Ordering::SeqCst), 1000);
        assert!(final_size <= 1000);
        assert!(final_size > 0);
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
    }
}
