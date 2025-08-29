use crate::arb::util::structs::persistent_cache::PersistentCache;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// Example: Custom cache with in-memory HashMap as backing store
pub async fn example_custom_backing_store() {
    // Custom backing store - could be any database or storage
    let backing_store: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    let backing_store_read = backing_store.clone();
    let backing_store_write = backing_store.clone();

    // Create cache with custom save/read functions
    let cache: PersistentCache<String, String> = PersistentCache::new_with_custom_db(
        100,
        Duration::from_secs(300),
        // Loader function - called when not in cache or backing store
        |key: &String| {
            let key = key.clone();
            async move {
                println!("Loading fresh data for key: {}", key);
                Some(format!("Fresh value for {}", key))
            }
        },
        // Custom save_to_db - saves to our HashMap
        move |key: String, value: String, _ttl: Duration| {
            let store = backing_store_write.clone();
            async move {
                let mut store = store.write().await;
                store.insert(key, value);
                println!("Saved to custom backing store");
            }
        },
        // Custom read_from_db - reads from our HashMap
        move |key: &String| {
            let store = backing_store_read.clone();
            let key = key.clone();
            async move {
                let store = store.read().await;
                let value = store.get(&key).cloned();
                if value.is_some() {
                    println!("Found in custom backing store");
                }
                value
            }
        },
    );

    // First access - will call loader and save to backing store
    let value1 = cache.get(&"key1".to_string()).await;
    println!("First access: {:?}", value1);

    // Second access - will retrieve from in-memory cache
    let value2 = cache.get(&"key1".to_string()).await;
    println!("Second access (from memory): {:?}", value2);

    // Manually put a value
    cache
        .put("key2".to_string(), "Manual value".to_string())
        .await;
}

// Example: Cache with no persistence (memory only)
pub async fn example_memory_only_cache() {
    let cache: PersistentCache<u32, String> = PersistentCache::new_with_custom_db(
        50,
        Duration::from_secs(60),
        // Loader function
        |key: &u32| {
            let key = *key;
            async move { Some(format!("Generated value for {}", key)) }
        },
        // No-op save function - memory only
        |_key: u32, _value: String, _ttl: Duration| async move {},
        // No-op read function - memory only (always returns None)
        |_key: &u32| async move { None },
    );

    let value = cache.get(&42).await;
    println!("Memory-only cache value: {:?}", value);
}
