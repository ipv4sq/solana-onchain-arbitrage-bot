use moka::future::{Cache, CacheBuilder};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

type LoaderFn<K, V> =
    Arc<dyn Fn(&K) -> Pin<Box<dyn Future<Output = Option<V>> + Send>> + Send + Sync>;

pub struct LoadingCache<K, V> {
    cache: Cache<K, Arc<V>>,
    loader: LoaderFn<K, V>,
}

impl<K, V> LoadingCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new<F, Fut>(max_capacity: u64, loader: F) -> Self
    where
        F: Fn(&K) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        let cache = CacheBuilder::new(max_capacity).build();

        let loader = Arc::new(
            move |key: &K| -> Pin<Box<dyn Future<Output = Option<V>> + Send>> {
                Box::pin(loader(key))
            },
        );

        Self { cache, loader }
    }

    pub fn with_ttl<F, Fut>(max_capacity: u64, ttl: Duration, loader: F) -> Self
    where
        F: Fn(&K) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        let cache = CacheBuilder::new(max_capacity)
            .time_to_live(ttl)
            .build();

        let loader = Arc::new(
            move |key: &K| -> Pin<Box<dyn Future<Output = Option<V>> + Send>> {
                Box::pin(loader(key))
            },
        );

        Self { cache, loader }
    }

    pub fn with_ttl_and_tti<F, Fut>(
        max_capacity: u64,
        ttl: Duration,
        tti: Duration,
        loader: F,
    ) -> Self
    where
        F: Fn(&K) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        let cache = CacheBuilder::new(max_capacity)
            .time_to_live(ttl)
            .time_to_idle(tti)
            .build();

        let loader = Arc::new(
            move |key: &K| -> Pin<Box<dyn Future<Output = Option<V>> + Send>> {
                Box::pin(loader(key))
            },
        );

        Self { cache, loader }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let loader = self.loader.clone();
        let key_clone = key.clone();
        
        self.cache
            .try_get_with(key.clone(), async move {
                match (loader)(&key_clone).await {
                    Some(value) => Ok(Arc::new(value)),
                    None => Err(()),
                }
            })
            .await
            .ok()
            .map(|arc| (*arc).clone())
    }

    pub async fn get_if_present(&self, key: &K) -> Option<V> {
        self.cache.get(key).await.map(|arc| (*arc).clone())
    }

    pub async fn put(&self, key: K, value: V) {
        self.cache.insert(key, Arc::new(value)).await;
    }

    pub async fn invalidate(&self, key: &K) {
        self.cache.invalidate(key).await;
    }

    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    pub fn weighted_size(&self) -> u64 {
        self.cache.weighted_size()
    }

    pub async fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks().await;
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    pub async fn get_multiple<I>(&self, keys: I) -> Vec<(K, V)>
    where
        I: IntoIterator<Item = K>,
    {
        let mut results = Vec::new();
        for key in keys {
            if let Some(value) = self.get(&key).await {
                results.push((key, value));
            }
        }
        results
    }

    pub async fn put_multiple<I>(&self, entries: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in entries {
            self.put(key, value).await;
        }
    }

    pub async fn invalidate_multiple<I>(&self, keys: I)
    where
        I: IntoIterator<Item = K>,
    {
        for key in keys {
            self.invalidate(&key).await;
        }
    }
}

unsafe impl<K: Send, V: Send> Send for LoadingCache<K, V> {}
unsafe impl<K: Send, V: Send> Sync for LoadingCache<K, V> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_operations() {
        let load_count = Arc::new(AtomicUsize::new(0));
        let count_clone = load_count.clone();

        let cache = LoadingCache::new(3, move |key: &String| {
            let count = count_clone.clone();
            let key = key.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Some(format!("value_{}", key))
            }
        });

        assert_eq!(cache.entry_count(), 0);

        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 1);
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        let val = cache.get_if_present(&"key2".to_string()).await;
        assert_eq!(val, None);
        assert_eq!(load_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_capacity_eviction() {
        let cache = LoadingCache::new(3, |key: &u32| {
            let key = *key;
            async move { Some(format!("value_{}", key)) }
        });

        cache.get(&1).await;
        cache.get(&2).await;
        cache.get(&3).await;
        
        cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 3);

        cache.get(&4).await;
        cache.run_pending_tasks().await;
        sleep(Duration::from_millis(100)).await;
        assert_eq!(cache.entry_count(), 3);

        assert!(cache.contains_key(&4));
    }

    #[tokio::test]
    async fn test_ttl() {
        let cache = LoadingCache::with_ttl(10, Duration::from_millis(100), |key: &u32| {
            let key = *key;
            async move { Some(format!("value_{}", key)) }
        });

        cache.get(&1).await;
        assert_eq!(cache.get_if_present(&1).await, Some("value_1".to_string()));

        sleep(Duration::from_millis(150)).await;
        cache.run_pending_tasks().await;

        assert_eq!(cache.get_if_present(&1).await, None);
    }

    #[tokio::test]
    async fn test_explicit_put() {
        let cache = LoadingCache::new(2, |_key: &String| async move {
            Some("loaded_value".to_string())
        });

        cache
            .put("key1".to_string(), "manual_value".to_string())
            .await;
        assert_eq!(
            cache.get(&"key1".to_string()).await,
            Some("manual_value".to_string())
        );

        cache
            .put("key1".to_string(), "updated_value".to_string())
            .await;
        assert_eq!(
            cache.get(&"key1".to_string()).await,
            Some("updated_value".to_string())
        );
    }

    #[tokio::test]
    async fn test_invalidation() {
        let cache = LoadingCache::new(5, |key: &u32| {
            let key = *key;
            async move { Some(format!("value_{}", key)) }
        });

        cache.get(&1).await;
        cache.get(&2).await;
        cache.get(&3).await;
        cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 3);

        cache.invalidate(&2).await;
        cache.run_pending_tasks().await;
        assert_eq!(cache.get_if_present(&2).await, None);

        cache.invalidate_all();
        cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 0);
    }

    #[tokio::test]
    async fn test_loader_returning_none() {
        let cache = LoadingCache::new(3, |key: &u32| {
            let key = *key;
            async move {
                if key % 2 == 0 {
                    Some(format!("value_{}", key))
                } else {
                    None
                }
            }
        });

        assert_eq!(cache.get(&1).await, None);
        assert_eq!(cache.get(&2).await, Some("value_2".to_string()));
        assert_eq!(cache.get(&3).await, None);
        assert_eq!(cache.get(&4).await, Some("value_4".to_string()));

        cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 2);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use tokio::task;

        let cache = Arc::new(LoadingCache::new(10, |key: &u32| {
            let key = *key;
            async move {
                sleep(Duration::from_millis(10)).await;
                Some(format!("value_{}", key))
            }
        }));

        let mut handles = vec![];

        for i in 0..20 {
            let cache_clone = cache.clone();
            handles.push(task::spawn(async move {
                let key = i % 5;
                let val = cache_clone.get(&key).await;
                assert_eq!(val, Some(format!("value_{}", key)));
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        cache.run_pending_tasks().await;
        assert!(cache.entry_count() <= 10);
    }

    #[tokio::test]
    async fn test_multiple_operations() {
        let cache = LoadingCache::new(10, |key: &u32| {
            let key = *key;
            async move { Some(format!("value_{}", key)) }
        });

        let keys = vec![1, 2, 3];
        let results = cache.get_multiple(keys.clone()).await;
        assert_eq!(results.len(), 3);

        let entries = vec![
            (10, "ten".to_string()),
            (11, "eleven".to_string()),
            (12, "twelve".to_string()),
        ];
        cache.put_multiple(entries).await;

        assert_eq!(cache.get_if_present(&10).await, Some("ten".to_string()));
        assert_eq!(cache.get_if_present(&11).await, Some("eleven".to_string()));
        assert_eq!(cache.get_if_present(&12).await, Some("twelve".to_string()));

        cache.invalidate_multiple(vec![10, 11]).await;
        cache.run_pending_tasks().await;
        
        assert_eq!(cache.get_if_present(&10).await, None);
        assert_eq!(cache.get_if_present(&11).await, None);
        assert_eq!(cache.get_if_present(&12).await, Some("twelve".to_string()));
    }
}