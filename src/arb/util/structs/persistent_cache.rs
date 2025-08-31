use crate::arb::database::kv_cache::repository::KvCacheRepository;
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

type SaveToDbFn<K, V> =
    Arc<dyn Fn(K, V, Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
type ReadFromDbFn<K, V> =
    Arc<dyn Fn(&K) -> Pin<Box<dyn Future<Output = Option<V>> + Send>> + Send + Sync>;

pub struct PersistentCache<K, V> {
    cache_type: Option<CacheType>,
    mem_cache: TtlLoadingCache<K, V>,
    default_ttl: Duration,
    save_to_db: Option<SaveToDbFn<K, V>>,
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
        F: Fn(&K) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        Self::from_boxed_fn(cache_type, max_entries, default_ttl, move |key| {
            Box::pin(loader(key))
        })
    }

    pub fn new_with_result<F, Fut>(
        cache_type: CacheType,
        max_entries: usize,
        default_ttl: Duration,
        loader: F,
    ) -> Self
    where
        F: Fn(&K) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    {
        Self::from_boxed_fn(cache_type, max_entries, default_ttl, move |key: &K| {
            let key = key.clone();
            let loader = loader.clone();
            Box::pin(async move { loader(&key).await.ok() })
        })
    }

    pub fn from_boxed_fn<F>(
        cache_type: CacheType,
        max_entries: usize,
        default_ttl: Duration,
        loader: F,
    ) -> Self
    where
        F: Fn(&K) -> Pin<Box<dyn Future<Output = Option<V>> + Send>> + Send + Sync + 'static + Clone,
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

                if let Ok(Some(cache_entry)) =
                    KvCacheRepository::get(cache_type.clone(), &key_json).await
                {
                    if let Ok(value) = serde_json::from_value::<V>(cache_entry.value) {
                        return Some(value);
                    }
                }

                let value = loader_fn(&key_clone).await?;

                let value_json = serde_json::to_value(&value).ok()?;
                let valid_until = Utc::now() + chrono::Duration::from_std(ttl).ok()?;

                let _ = KvCacheRepository::put(cache_type, key_json, value_json, valid_until).await;

                Some(value)
            }) as Pin<Box<dyn Future<Output = Option<V>> + Send>>
        };

        let mem_cache = TtlLoadingCache::new(max_entries, default_ttl, db_aware_loader);

        Self {
            cache_type: Some(cache_type),
            mem_cache,
            default_ttl,
            save_to_db: None,
        }
    }

    pub fn new_with_custom_db<F, Fut, SaveFut, ReadFut>(
        max_entries: usize,
        default_ttl: Duration,
        loader: F,
        save_to_db: impl Fn(K, V, Duration) -> SaveFut + Send + Sync + 'static,
        read_from_db: impl Fn(&K) -> ReadFut + Send + Sync + 'static,
    ) -> Self
    where
        F: Fn(&K) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<V>> + Send + 'static,
        SaveFut: Future<Output = ()> + Send + 'static,
        ReadFut: Future<Output = Option<V>> + Send + 'static,
    {
        let loader = Arc::new(loader);
        let read_from_db_arc: ReadFromDbFn<K, V> = Arc::new(
            move |key: &K| -> Pin<Box<dyn Future<Output = Option<V>> + Send>> {
                Box::pin(read_from_db(key))
            },
        );
        let save_to_db_arc: SaveToDbFn<K, V> = Arc::new(
            move |key: K, value: V, ttl: Duration| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                Box::pin(save_to_db(key, value, ttl))
            },
        );

        let read_from_db_clone = read_from_db_arc.clone();
        let save_to_db_clone = save_to_db_arc.clone();
        let ttl = default_ttl;

        let db_aware_loader = move |key: &K| {
            let key_clone = key.clone();
            let loader_fn = loader.clone();
            let read_from_db = read_from_db_clone.clone();
            let save_to_db = save_to_db_clone.clone();
            let ttl = ttl;

            Box::pin(async move {
                // Check read_from_db first
                if let Some(value) = read_from_db(&key_clone).await {
                    return Some(value);
                }

                // Call loader
                let value = loader_fn(&key_clone).await?;

                // Save to db
                save_to_db(key_clone.clone(), value.clone(), ttl).await;

                Some(value)
            }) as Pin<Box<dyn Future<Output = Option<V>> + Send>>
        };

        let mem_cache = TtlLoadingCache::new(max_entries, default_ttl, db_aware_loader);

        Self {
            cache_type: None,
            mem_cache,
            default_ttl,
            save_to_db: Some(save_to_db_arc),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        self.mem_cache.get(key).await
    }

    pub fn get_if_present(&self, key: &K) -> Option<V> {
        self.mem_cache.get_if_present(key)
    }

    pub async fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, self.default_ttl).await;
    }

    pub async fn put_with_ttl(&self, key: K, value: V, ttl: Duration) {
        self.mem_cache
            .put_with_ttl(key.clone(), value.clone(), ttl)
            .await;

        // Use custom save_to_db if provided
        if let Some(ref save_fn) = self.save_to_db {
            save_fn(key, value, ttl).await;
        }
        // Otherwise use default kv_cache if cache_type is set
        else if let Some(ref cache_type) = self.cache_type {
            if let (Ok(key_json), Ok(value_json)) =
                (serde_json::to_string(&key), serde_json::to_value(&value))
            {
                let valid_until = Utc::now()
                    + chrono::Duration::from_std(ttl)
                        .unwrap_or_else(|_| chrono::Duration::seconds(0));

                let _ =
                    KvCacheRepository::put(cache_type.clone(), key_json, value_json, valid_until)
                        .await;
            }
        }
    }

    pub async fn evict(&self, key: &K) {
        self.mem_cache.invalidate(key).await;

        // For evict, we only handle the default kv_cache case
        // Custom implementations should handle their own eviction logic
        if let Some(ref cache_type) = self.cache_type {
            if let Ok(key_json) = serde_json::to_string(key) {
                let _ = KvCacheRepository::evict(cache_type.clone(), &key_json).await;
            }
        }
    }

    pub async fn ensure_exists(&self, key: &K) -> Option<V> {
        self.get(key).await
    }
}
