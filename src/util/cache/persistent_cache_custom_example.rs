use crate::util::cache::persistent_cache::PersistentCache;
use crate::util::structs::cache_type::CacheType;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub async fn example_custom_backing_store() {
    use crate::database::pool_record::model::PoolRecord;
    use crate::database::pool_record::repository::PoolRecordRepository;
    use crate::util::traits::pubkey::ToPubkey;
    
    // Example 1: Using custom database functions for pool records
    let cache = PersistentCache::new_with_custom_db(
        CacheType::Custom("pool_records".to_string()),
        100,
        3600,
        |pool_address: String| async move {
            // This is the loader function - called when not in cache or DB
            None // In real usage, you might fetch from RPC or other sources
        },
        Some(|pool_address: String| async move {
            // Custom load function - directly from pools table
            let address = pool_address.to_pubkey();
            PoolRecordRepository::get(&address).await.ok().flatten()
        }),
        Some(|pool_address: String, record: PoolRecord, _ttl: i64| async move {
            // Custom save function - directly to pools table
            let _ = PoolRecordRepository::upsert(record).await;
        }),
    );
    
    // Usage
    let pool_address = "Q2sPHPdUWFMg7M7wwrQKLrn619cAucfRsmhVJffodSp".to_string();
    let _pool_record = cache.get(&pool_address).await;
}

#[allow(dead_code)]
pub async fn example_memory_only_cache() {
    // Example 2: Memory-only cache (no database persistence)
    // Provide both functions but make them no-ops
    
    let cache = PersistentCache::<String, String>::new_with_custom_db(
        CacheType::Custom("memory_only".to_string()),
        100,
        3600,
        |key: String| async move {
            // Loader function
            Some(format!("loaded_{}", key))
        },
        Some(|_key: String| async move {
            // Custom load - always return None (no persistence)
            None
        }),
        Some(|_key: String, _value: String, _ttl: i64| async move {
            // Custom save - do nothing (no persistence)
        }),
    );
    
    // This cache will only keep data in memory, no DB persistence
    let _value = cache.get(&"test_key".to_string()).await;
}

#[allow(dead_code)]
pub async fn example_custom_storage() {
    // Example 3: Using a custom storage backend (e.g., Redis simulation)
    let custom_storage: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let storage_clone1 = custom_storage.clone();
    let storage_clone2 = custom_storage.clone();
    
    let cache = PersistentCache::new_with_custom_db(
        CacheType::Custom("custom_backend".to_string()),
        100,
        3600,
        |key: String| async move {
            // Loader function - fetch from external source
            Some(format!("fetched_{}", key))
        },
        Some(move |key: String| {
            let storage = storage_clone1.clone();
            async move {
                // Load from custom storage
                storage.lock().await.get(&key).cloned()
            }
        }),
        Some(move |key: String, value: String, _ttl: i64| {
            let storage = storage_clone2.clone();
            async move {
                // Save to custom storage
                storage.lock().await.insert(key, value);
            }
        }),
    );
    
    // Usage
    cache.put("key1".to_string(), "value1".to_string()).await;
    let _value = cache.get(&"key1".to_string()).await;
}

#[allow(dead_code)]
pub async fn example_fallback_to_kv_cache() {
    // Example 4: Partial custom functions (fallback to kv_cache)
    
    // Only provide custom load, save will use kv_cache
    let cache = PersistentCache::<String, String>::new_with_custom_db(
        CacheType::Custom("partial_custom".to_string()),
        100,
        3600,
        |key: String| async move {
            Some(format!("loaded_{}", key))
        },
        Some(|key: String| async move {
            // Custom load logic
            if key.starts_with("special_") {
                Some(format!("special_value_{}", key))
            } else {
                None
            }
        }),
        None::<fn(String, String, i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>>, // Save will use kv_cache
    );
    
    // This will use custom load but NOT save to kv_cache
    // (because when one custom function is provided, kv_cache is bypassed)
    let _value = cache.get(&"test".to_string()).await;
}