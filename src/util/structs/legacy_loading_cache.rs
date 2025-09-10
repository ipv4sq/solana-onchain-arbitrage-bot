use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

struct CacheEntry<V> {
    value: Arc<V>,
    last_accessed: Instant,
}

type LoaderFn<K, V> =
    Arc<dyn Fn(&K) -> Pin<Box<dyn Future<Output = Option<V>> + Send>> + Send + Sync>;

pub struct LoadingCache<K, V> {
    inner: Arc<RwLock<CacheInner<K, V>>>,
    loader: LoaderFn<K, V>,
    max_entries: usize,
}

struct CacheInner<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    access_order: VecDeque<K>,
}

impl<K, V> LoadingCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new<F, Fut>(max_entries: usize, loader: F) -> Self
    where
        F: Fn(&K) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<V>> + Send + 'static,
    {
        assert!(max_entries > 0, "max_entries must be greater than 0");

        let loader = Arc::new(
            move |key: &K| -> Pin<Box<dyn Future<Output = Option<V>> + Send>> {
                Box::pin(loader(key))
            },
        );

        Self {
            inner: Arc::new(RwLock::new(CacheInner {
                entries: HashMap::with_capacity(max_entries),
                access_order: VecDeque::with_capacity(max_entries),
            })),
            loader,
            max_entries,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        {
            let mut cache = self.inner.write();
            if cache.entries.contains_key(key) {
                let entry = cache.entries.get_mut(key).unwrap();
                entry.last_accessed = Instant::now();
                let value = entry.value.as_ref().clone();
                Self::move_to_front(&mut cache.access_order, key);
                return Some(value);
            }
        }

        let value = (self.loader)(key).await?;
        self.store_value(key.clone(), value.clone()).await;
        Some(value)
    }

    pub fn get_sync(&self, key: &K) -> Option<V> {
        {
            let mut cache = self.inner.write();
            if cache.entries.contains_key(key) {
                let entry = cache.entries.get_mut(key).unwrap();
                entry.last_accessed = Instant::now();
                let value = entry.value.as_ref().clone();
                Self::move_to_front(&mut cache.access_order, key);
                return Some(value);
            }
        }

        let handle = tokio::runtime::Handle::try_current()
            .expect("get_sync must be called within a tokio runtime");

        tokio::task::block_in_place(|| {
            handle.block_on(async {
                let value = (self.loader)(key).await?;
                self.store_value(key.clone(), value.clone()).await;
                Some(value)
            })
        })
    }

    pub fn get_if_present(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write();
        if cache.entries.contains_key(key) {
            let entry = cache.entries.get_mut(key).unwrap();
            entry.last_accessed = Instant::now();
            let value = entry.value.as_ref().clone();
            Self::move_to_front(&mut cache.access_order, key);
            Some(value)
        } else {
            None
        }
    }

    pub async fn put(&self, key: K, value: V) {
        self.store_value(key, value).await;
    }

    pub async fn invalidate(&self, key: &K) {
        let mut cache = self.inner.write();
        if cache.entries.remove(key).is_some() {
            cache.access_order.retain(|k| k != key);
        }
    }

    pub async fn invalidate_all(&self) {
        let mut cache = self.inner.write();
        cache.entries.clear();
        cache.access_order.clear();
    }

    pub async fn size(&self) -> usize {
        self.inner.read().entries.len()
    }

    async fn store_value(&self, key: K, value: V) {
        let mut cache = self.inner.write();

        if cache.entries.contains_key(&key) {
            let entry = cache.entries.get_mut(&key).unwrap();
            entry.value = Arc::new(value);
            entry.last_accessed = Instant::now();
            Self::move_to_front(&mut cache.access_order, &key);
            return;
        }

        if cache.entries.len() >= self.max_entries {
            if let Some(lru_key) = cache.access_order.pop_back() {
                cache.entries.remove(&lru_key);
            }
        }

        cache.entries.insert(
            key.clone(),
            CacheEntry {
                value: Arc::new(value),
                last_accessed: Instant::now(),
            },
        );
        cache.access_order.push_front(key);
    }

    fn move_to_front(access_order: &mut VecDeque<K>, key: &K) {
        if let Some(pos) = access_order.iter().position(|k| k == key) {
            access_order.remove(pos);
        }
        access_order.push_front(key.clone());
    }
}

unsafe impl<K: Send, V: Send> Send for LoadingCache<K, V> {}
unsafe impl<K: Send, V: Send> Sync for LoadingCache<K, V> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
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

        assert_eq!(cache.size().await, 0);

        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(cache.size().await, 1);
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        let val = cache.get_if_present(&"key2".to_string());
        assert_eq!(val, None);
        assert_eq!(load_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = LoadingCache::new(3, |key: &u32| {
            let key = *key;
            async move { Some(format!("value_{}", key)) }
        });

        cache.get(&1).await;
        cache.get(&2).await;
        cache.get(&3).await;
        assert_eq!(cache.size().await, 3);

        cache.get(&4).await;
        assert_eq!(cache.size().await, 3);

        assert_eq!(cache.get_if_present(&1), None);
        assert_eq!(cache.get_if_present(&2), Some("value_2".to_string()));
        assert_eq!(cache.get_if_present(&3), Some("value_3".to_string()));
        assert_eq!(cache.get_if_present(&4), Some("value_4".to_string()));
    }

    #[tokio::test]
    async fn test_lru_access_order() {
        let cache = LoadingCache::new(3, |key: &u32| {
            let key = *key;
            async move { Some(format!("value_{}", key)) }
        });

        cache.get(&1).await;
        cache.get(&2).await;
        cache.get(&3).await;

        cache.get(&1).await;

        cache.get(&4).await;

        assert_eq!(cache.get_if_present(&1), Some("value_1".to_string()));
        assert_eq!(cache.get_if_present(&2), None);
        assert_eq!(cache.get_if_present(&3), Some("value_3".to_string()));
        assert_eq!(cache.get_if_present(&4), Some("value_4".to_string()));
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
        assert_eq!(cache.size().await, 3);

        cache.invalidate(&2).await;
        assert_eq!(cache.size().await, 2);
        assert_eq!(cache.get_if_present(&2), None);

        cache.invalidate_all().await;
        assert_eq!(cache.size().await, 0);
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

        assert_eq!(cache.size().await, 2);
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

        assert!(cache.size().await <= 10);
    }

    #[tokio::test]
    async fn test_single_entry_cache() {
        let cache = LoadingCache::new(1, |key: &u32| {
            let key = *key;
            async move { Some(format!("value_{}", key)) }
        });

        cache.get(&1).await;
        assert_eq!(cache.size().await, 1);

        cache.get(&2).await;
        assert_eq!(cache.size().await, 1);
        assert_eq!(cache.get_if_present(&1), None);
        assert_eq!(cache.get_if_present(&2), Some("value_2".to_string()));
    }

    #[tokio::test]
    async fn test_loader_with_captured_state() {
        let multiplier = 10;
        let cache = LoadingCache::new(3, move |key: &u32| {
            let key = *key;
            async move { Some(key * multiplier) }
        });

        assert_eq!(cache.get(&5).await, Some(50));
        assert_eq!(cache.get(&7).await, Some(70));
    }
}
