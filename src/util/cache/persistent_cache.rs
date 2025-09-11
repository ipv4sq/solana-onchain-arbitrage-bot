use crate::database::kv_cache::repository::KvCacheRepository;
use crate::util::cache::loading_cache::LoadingCache;
use crate::util::structs::cache_type::CacheType;
use chrono::{Duration, Utc};
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

pub struct PersistentCache<K, V> {
    cache_type: CacheType,
    loading_cache: LoadingCache<K, Arc<Option<V>>>,
    ttl_seconds: i64,
}

impl<K, V> PersistentCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static + ToString,
    V: Clone + Send + Sync + 'static + Serialize + DeserializeOwned,
{
    async fn load_from_db(cache_type: &CacheType, key: &str) -> Option<V> {
        if let Ok(Some(kv_cache)) = KvCacheRepository::get(cache_type.clone(), key).await {
            if let Ok(value) = serde_json::from_value::<V>(kv_cache.value) {
                return Some(value);
            }
        }
        None
    }

    async fn save_to_db(cache_type: &CacheType, key: &str, value: &V, ttl_seconds: i64) {
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
        let cache_type_clone = cache_type.clone();
        let ttl_seconds_clone = ttl_seconds;
        let loader_clone = loader.clone();

        let loading_cache = LoadingCache::new(max_capacity, move |key: &K| {
            let cache_type = cache_type_clone.clone();
            let key_str = key.to_string();
            let key_clone = key.clone();
            let loader = loader_clone.clone();
            let ttl = ttl_seconds_clone;
            
            async move {
                // Try to load from database first
                if let Some(value) = Self::load_from_db(&cache_type, &key_str).await {
                    return Some(Arc::new(Some(value)));
                }
                
                // If not in database, call the loader
                let loaded_value = loader(key_clone).await;
                
                // Save to database if we got a value
                if let Some(ref value) = loaded_value {
                    Self::save_to_db(&cache_type, &key_str, value, ttl).await;
                }
                
                Some(Arc::new(loaded_value))
            }
        });

        Self {
            cache_type,
            loading_cache,
            ttl_seconds,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        self.loading_cache
            .get(key)
            .await
            .and_then(|arc| (*arc).clone())
    }

    pub async fn put(&self, key: K, value: V) {
        let key_str = key.to_string();
        
        // Save to database
        Self::save_to_db(&self.cache_type, &key_str, &value, self.ttl_seconds).await;
        
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
}