use crate::database::kv_cache::repository::KvCacheRepository;
use crate::util::cache::loading_cache::LoadingCache;
use crate::util::structs::cache_type::CacheType;
use chrono::{Duration, Utc};
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;

type LoadFromDbFn<K, V> = Arc<dyn Fn(K) -> Pin<Box<dyn Future<Output = Option<V>> + Send>> + Send + Sync>;
type SaveToDbFn<K, V> = Arc<dyn Fn(K, V, i64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct PersistentCache<K, V> {
    cache_type: CacheType,
    loading_cache: LoadingCache<K, Arc<Option<V>>>,
    ttl_seconds: i64,
    load_from_db_fn: Option<LoadFromDbFn<K, V>>,
    save_to_db_fn: Option<SaveToDbFn<K, V>>,
}

impl<K, V> PersistentCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static + ToString,
    V: Clone + Send + Sync + 'static + Serialize + DeserializeOwned,
{
    async fn kv_cache_load(cache_type: &CacheType, key: &str) -> Option<V> {
        if let Ok(Some(kv_cache)) = KvCacheRepository::get(cache_type.clone(), key).await {
            if let Ok(value) = serde_json::from_value::<V>(kv_cache.value) {
                return Some(value);
            }
        }
        None
    }

    async fn kv_cache_save(cache_type: &CacheType, key: &str, value: &V, ttl_seconds: i64) {
        let valid_until = Utc::now() + Duration::seconds(ttl_seconds);
        if let Ok(json_value) = serde_json::to_value(value) {
            let _ = KvCacheRepository::put(
                cache_type.clone(),
                key.to_string(),
                json_value,
                valid_until,
            )
            .await;
        }
    }

    fn new_internal<F, Fut>(
        cache_type: CacheType,
        max_capacity: u64,
        ttl_seconds: i64,
        loader: F,
        load_from_db_fn: Option<LoadFromDbFn<K, V>>,
        save_to_db_fn: Option<SaveToDbFn<K, V>>,
    ) -> Self
    where
        F: Fn(K) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        let cache_type_for_closure = cache_type.clone();
        let load_from_db_fn_clone = load_from_db_fn.clone();
        let save_to_db_fn_clone = save_to_db_fn.clone();

        let loading_cache = LoadingCache::new(max_capacity, move |key: &K| {
            let cache_type = cache_type_for_closure.clone();
            let key_str = key.to_string();
            let key_clone = key.clone();
            let loader = loader.clone();
            let load_fn = load_from_db_fn_clone.clone();
            let save_fn = save_to_db_fn_clone.clone();
            
            async move {
                // Try to load from database first
                let db_value = if let Some(ref load_fn) = load_fn {
                    load_fn(key_clone.clone()).await
                } else {
                    Self::kv_cache_load(&cache_type, &key_str).await
                };

                if let Some(value) = db_value {
                    return Some(Arc::new(Some(value)));
                }
                
                // If not in database, call the loader
                let loaded_value = loader(key_clone.clone()).await;
                
                // Save to database if we got a value
                if let Some(ref value) = loaded_value {
                    if let Some(ref save_fn) = save_fn {
                        save_fn(key_clone, value.clone(), ttl_seconds).await;
                    } else if load_fn.is_none() {
                        // Only use kv_cache if neither custom function is provided
                        Self::kv_cache_save(&cache_type, &key_str, value, ttl_seconds).await;
                    }
                }
                
                Some(Arc::new(loaded_value))
            }
        });

        Self {
            cache_type,
            loading_cache,
            ttl_seconds,
            load_from_db_fn,
            save_to_db_fn,
        }
    }

    pub fn new<F, Fut>(
        cache_type: CacheType,
        max_capacity: u64,
        ttl_seconds: i64,
        loader: F,
    ) -> Self
    where
        F: Fn(K) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        Self::new_internal(cache_type, max_capacity, ttl_seconds, loader, None, None)
    }

    pub fn new_with_custom_db<F, Fut, LF, SF, LFut, SFut>(
        cache_type: CacheType,
        max_capacity: u64,
        ttl_seconds: i64,
        loader: F,
        load_from_db: LF,
        save_to_db: SF,
    ) -> Self
    where
        F: Fn(K) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Option<V>> + Send + 'static,
        LF: Fn(K) -> LFut + Send + Sync + 'static,
        LFut: Future<Output = Option<V>> + Send + 'static,
        SF: Fn(K, V, i64) -> SFut + Send + Sync + 'static,
        SFut: Future<Output = ()> + Send + 'static,
    {
        let load_from_db_fn: LoadFromDbFn<K, V> = Arc::new(move |key| 
            Box::pin(load_from_db(key)) as Pin<Box<dyn Future<Output = Option<V>> + Send>>
        );

        let save_to_db_fn: SaveToDbFn<K, V> = Arc::new(move |key, value, ttl| 
            Box::pin(save_to_db(key, value, ttl)) as Pin<Box<dyn Future<Output = ()> + Send>>
        );

        Self::new_internal(cache_type, max_capacity, ttl_seconds, loader, Some(load_from_db_fn), Some(save_to_db_fn))
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        self.loading_cache
            .get(key)
            .await
            .and_then(|arc| (*arc).clone())
    }

    pub async fn get_if_present(&self, key: &K) -> Option<V> {
        self.loading_cache
            .get_if_present(key)
            .await
            .and_then(|arc| (*arc).clone())
    }

    pub async fn put(&self, key: K, value: V) {
        // Save to database
        if let Some(ref save_fn) = self.save_to_db_fn {
            save_fn(key.clone(), value.clone(), self.ttl_seconds).await;
        } else if self.load_from_db_fn.is_none() {
            // Only use kv_cache if neither custom function is provided
            let key_str = key.to_string();
            Self::kv_cache_save(&self.cache_type, &key_str, &value, self.ttl_seconds).await;
        }
        
        // Update in-memory cache
        self.loading_cache.put(key, Arc::new(Some(value))).await;
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.loading_cache.contains_key(key)
    }

    pub fn entry_count(&self) -> u64 {
        self.loading_cache.entry_count()
    }

    pub async fn run_pending_tasks(&self) {
        self.loading_cache.run_pending_tasks().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global::client::db::must_init_db;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_basic_operations() {
        must_init_db().await;
        
        let load_count = Arc::new(AtomicUsize::new(0));
        let count_clone = load_count.clone();

        let cache = PersistentCache::new(
            CacheType::Custom("test_basic".to_string()),
            10,
            60,
            move |key: String| {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Some(format!("value_{}", key))
                }
            },
        );

        // First call should load from loader
        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        // Second call should use cached value
        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        // Test with a different key
        let val = cache.get(&"key2".to_string()).await;
        assert_eq!(val, Some("value_key2".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_null_results_caching() {
        must_init_db().await;
        
        let load_count = Arc::new(AtomicUsize::new(0));
        let count_clone = load_count.clone();

        let cache = PersistentCache::new(
            CacheType::Custom("test_null".to_string()),
            10,
            60,
            move |key: String| {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    if key == "exists" {
                        Some("value".to_string())
                    } else {
                        None
                    }
                }
            },
        );

        let val = cache.get(&"notexists".to_string()).await;
        assert_eq!(val, None);
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        // Second call should not increment load count because None is cached
        let val = cache.get(&"notexists".to_string()).await;
        assert_eq!(val, None);
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        let val = cache.get(&"exists".to_string()).await;
        assert_eq!(val, Some("value".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_explicit_put() {
        must_init_db().await;
        
        let cache = PersistentCache::new(
            CacheType::Custom("test_put".to_string()),
            10,
            60,
            |_key: String| async move { Some("loaded".to_string()) },
        );

        cache.put("key1".to_string(), "manual".to_string()).await;
        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("manual".to_string()));

        cache.put("key1".to_string(), "updated".to_string()).await;
        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("updated".to_string()));
    }

    #[tokio::test]
    async fn test_persistence_across_instances() {
        must_init_db().await;
        
        let cache_type = CacheType::Custom("test_persist".to_string());
        
        {
            let cache = PersistentCache::new(
                cache_type.clone(),
                10,
                60,
                |_key: String| async move { Some("loaded".to_string()) },
            );
            
            cache.put("key1".to_string(), "persisted".to_string()).await;
        }
        
        {
            let load_count = Arc::new(AtomicUsize::new(0));
            let count_clone = load_count.clone();
            
            let cache = PersistentCache::new(
                cache_type,
                10,
                60,
                move |_key: String| {
                    let count = count_clone.clone();
                    async move {
                        count.fetch_add(1, Ordering::SeqCst);
                        Some("loaded".to_string())
                    }
                },
            );
            
            let val = cache.get(&"key1".to_string()).await;
            assert_eq!(val, Some("persisted".to_string()));
            assert_eq!(load_count.load(Ordering::SeqCst), 0);
        }
    }

    #[tokio::test]
    async fn test_custom_db_functions() {
        must_init_db().await;
        
        use std::collections::HashMap;
        use tokio::sync::Mutex;
        
        let custom_storage: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let storage_clone1 = custom_storage.clone();
        let storage_clone2 = custom_storage.clone();
        
        let load_count = Arc::new(AtomicUsize::new(0));
        let save_count = Arc::new(AtomicUsize::new(0));
        let load_count_clone = load_count.clone();
        let save_count_clone = save_count.clone();
        
        let cache = PersistentCache::new_with_custom_db(
            CacheType::Custom("test_custom_db".to_string()),
            10,
            60,
            |key: String| async move {
                if key == "exists" {
                    Some(format!("loaded_{}", key))
                } else {
                    None
                }
            },
            move |key: String| {
                let storage = storage_clone1.clone();
                let count = load_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    storage.lock().await.get(&key).cloned()
                }
            },
            move |key: String, value: String, _ttl: i64| {
                let storage = storage_clone2.clone();
                let count = save_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    storage.lock().await.insert(key, value);
                }
            },
        );
        
        // First call loads from loader and saves via custom function
        let val = cache.get(&"exists".to_string()).await;
        assert_eq!(val, Some("loaded_exists".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);
        assert_eq!(save_count.load(Ordering::SeqCst), 1);
        
        // Put should use custom save function
        cache.put("custom_key".to_string(), "custom_value".to_string()).await;
        assert_eq!(save_count.load(Ordering::SeqCst), 2);
        
        // Verify it was saved in custom storage
        assert_eq!(
            custom_storage.lock().await.get("custom_key"),
            Some(&"custom_value".to_string())
        );
    }

    #[tokio::test]
    async fn test_custom_functions_with_both_provided() {
        must_init_db().await;
        
        let cache_type = CacheType::Custom("test_both_custom".to_string());
        
        // First create cache with custom functions and save something
        {
            use std::collections::HashMap;
            use tokio::sync::Mutex;
            
            let storage: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
            let storage_clone1 = storage.clone();
            let storage_clone2 = storage.clone();
            
            let cache = PersistentCache::new_with_custom_db(
                cache_type.clone(),
                10,
                60,
                |_key: String| async move { None },
                move |key: String| {
                    let storage = storage_clone1.clone();
                    async move {
                        storage.lock().await.get(&key).cloned()
                    }
                },
                move |key: String, value: String, _ttl: i64| {
                    let storage = storage_clone2.clone();
                    async move {
                        storage.lock().await.insert(key, value);
                    }
                },
            );
            
            cache.put("test_key".to_string(), "test_value".to_string()).await;
        }
        
        // Now create a regular cache with no custom functions
        // It should NOT find the value because custom functions don't use kv_cache
        {
            let cache = PersistentCache::new(
                cache_type,
                10,
                60,
                |_key: String| async move { Some("from_loader".to_string()) },
            );
            
            let val = cache.get(&"test_key".to_string()).await;
            assert_eq!(val, Some("from_loader".to_string()));
        }
    }
}