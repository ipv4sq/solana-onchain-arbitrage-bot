use crate::arb::database::repositories::KvCacheRepository;
use crate::arb::util::structs::cache_type::CacheType;
use crate::arb::util::structs::ttl_loading_cache::TtlLoadingCache;
use chrono::Utc;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
#[path = "persistent_cache_test.rs"]
mod tests;

pub struct PersistentCache<K, V> {
    cache_type: CacheType,
    mem_cache: TtlLoadingCache<K, V>,
    default_ttl: Duration,
}

impl<K, V> PersistentCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + Serialize + DeserializeOwned + 'static,
    V: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn new<F, Fut>(
        cache_type: CacheType,
        max_entries: usize,
        default_ttl: Duration,
        loader: F,
    ) -> Self
    where
        F: Fn(&K) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        let cache_type_clone = cache_type.clone();
        let ttl = default_ttl;
        let loader = Arc::new(loader);
        
        let db_aware_loader = move |key: &K| {
            let cache_type = cache_type_clone.clone();
            let key_clone = key.clone();
            let loader_fn = loader.clone();
            let ttl = ttl;
            
            Box::pin(async move {
                let key_json = serde_json::to_string(&key_clone).ok()?;
                
                if let Ok(Some(cache_entry)) = KvCacheRepository::get(cache_type.clone(), &key_json).await {
                    if let Ok(value) = serde_json::from_value::<V>(cache_entry.value) {
                        return Some(value);
                    }
                }
                
                let value = loader_fn(&key_clone).await?;
                
                let value_json = serde_json::to_value(&value).ok()?;
                let valid_until = Utc::now() + chrono::Duration::from_std(ttl).ok()?;
                
                let _ = KvCacheRepository::put(
                    cache_type,
                    key_json,
                    value_json,
                    valid_until,
                )
                .await;
                
                Some(value)
            }) as Pin<Box<dyn Future<Output = Option<V>> + Send>>
        };
        
        let mem_cache = TtlLoadingCache::new(
            max_entries,
            default_ttl,
            db_aware_loader,
        );
        
        Self {
            cache_type,
            mem_cache,
            default_ttl,
        }
    }
    
    pub async fn get(&self, key: &K) -> Option<V> {
        self.mem_cache.get(key).await
    }
    
    pub async fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, self.default_ttl).await;
    }
    
    pub async fn put_with_ttl(&self, key: K, value: V, ttl: Duration) {
        self.mem_cache.put_with_ttl(key.clone(), value.clone(), ttl).await;
        
        if let (Ok(key_json), Ok(value_json)) = (
            serde_json::to_string(&key),
            serde_json::to_value(&value),
        ) {
            let valid_until = Utc::now() + chrono::Duration::from_std(ttl).unwrap_or_else(|_| chrono::Duration::seconds(0));
            
            let _ = KvCacheRepository::put(
                self.cache_type.clone(),
                key_json,
                value_json,
                valid_until,
            )
            .await;
        }
    }
    
    pub async fn evict(&self, key: &K) {
        self.mem_cache.invalidate(key).await;
        
        if let Ok(key_json) = serde_json::to_string(key) {
            let _ = KvCacheRepository::evict(self.cache_type.clone(), &key_json).await;
        }
    }
}