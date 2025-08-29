#[cfg(test)]
mod tests {
    use crate::arb::util::structs::cache_type::CacheType;
    use crate::arb::util::structs::persistent_cache::PersistentCache;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_persistent_cache_basic() {
        let load_count = Arc::new(AtomicUsize::new(0));
        let count_clone = load_count.clone();
        
        let cache = PersistentCache::new(
            CacheType::Custom("test_cache".to_string()),
            10,
            Duration::from_secs(60),
            move |key: &String| {
                let count = count_clone.clone();
                let key = key.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Some(format!("value_for_{}", key))
                }
            },
        );
        
        let value1 = cache.get(&"key1".to_string()).await;
        assert_eq!(value1, Some("value_for_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);
        
        let value1_again = cache.get(&"key1".to_string()).await;
        assert_eq!(value1_again, Some("value_for_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);
    }
    
    #[tokio::test]
    async fn test_persistent_cache_put_and_evict() {
        let cache = PersistentCache::new(
            CacheType::Custom("test_put_cache".to_string()),
            10,
            Duration::from_secs(60),
            |_key: &String| async { None },
        );
        
        cache.put("manual_key".to_string(), "manual_value".to_string()).await;
        
        let value = cache.get(&"manual_key".to_string()).await;
        assert_eq!(value, Some("manual_value".to_string()));
        
        cache.evict(&"manual_key".to_string()).await;
        
        let value_after_evict = cache.get(&"manual_key".to_string()).await;
        assert_eq!(value_after_evict, None);
    }
    
    #[tokio::test]
    async fn test_persistent_cache_with_ttl() {
        let cache = PersistentCache::new(
            CacheType::Custom("test_ttl_cache".to_string()),
            10,
            Duration::from_secs(60),
            |_key: &String| async { None },
        );
        
        cache.put_with_ttl(
            "short_ttl".to_string(), 
            "expires_soon".to_string(),
            Duration::from_millis(100)
        ).await;
        
        let value = cache.get(&"short_ttl".to_string()).await;
        assert_eq!(value, Some("expires_soon".to_string()));
        
        sleep(Duration::from_millis(150)).await;
        
        let value_after_expiry = cache.get(&"short_ttl".to_string()).await;
        assert_eq!(value_after_expiry, None);
    }
}