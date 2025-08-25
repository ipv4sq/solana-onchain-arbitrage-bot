use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct CacheEntry<V> {
    value: Arc<V>,
    inserted_at: Instant,
    ttl: Duration,
    last_accessed: Instant,
}

impl<V> CacheEntry<V> {
    fn is_expired(&self) -> bool {
        Instant::now() > self.inserted_at + self.ttl
    }
}

type LoaderFn<K, V> =
    Arc<dyn Fn(&K) -> Pin<Box<dyn Future<Output = Option<V>> + Send>> + Send + Sync>;

pub struct TtlLoadingCache<K, V> {
    inner: Arc<RwLock<CacheInner<K, V>>>,
    loader: LoaderFn<K, V>,
    max_entries: usize,
    default_ttl: Duration,
}

struct CacheInner<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    access_order: VecDeque<K>,
}

impl<K, V> TtlLoadingCache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new<F, Fut>(max_entries: usize, default_ttl: Duration, loader: F) -> Self
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
            default_ttl,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        {
            let mut cache = self.inner.write();
            if let Some(entry) = cache.entries.get_mut(key) {
                if !entry.is_expired() {
                    entry.last_accessed = Instant::now();
                    let value = entry.value.as_ref().clone();
                    Self::move_to_front(&mut cache.access_order, key);
                    return Some(value);
                } else {
                    cache.entries.remove(key);
                    cache.access_order.retain(|k| k != key);
                }
            }
        }

        let value = (self.loader)(key).await?;
        self.store_value(key.clone(), value.clone(), self.default_ttl).await;
        Some(value)
    }

    pub fn get_sync(&self, key: &K) -> Option<V> {
        {
            let mut cache = self.inner.write();
            if let Some(entry) = cache.entries.get_mut(key) {
                if !entry.is_expired() {
                    entry.last_accessed = Instant::now();
                    let value = entry.value.as_ref().clone();
                    Self::move_to_front(&mut cache.access_order, key);
                    return Some(value);
                } else {
                    cache.entries.remove(key);
                    cache.access_order.retain(|k| k != key);
                }
            }
        }

        let handle = tokio::runtime::Handle::try_current()
            .expect("get_sync must be called within a tokio runtime");

        tokio::task::block_in_place(|| {
            handle.block_on(async {
                let value = (self.loader)(key).await?;
                self.store_value(key.clone(), value.clone(), self.default_ttl).await;
                Some(value)
            })
        })
    }

    pub fn get_if_present(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write();
        if let Some(entry) = cache.entries.get_mut(key) {
            if !entry.is_expired() {
                entry.last_accessed = Instant::now();
                let value = entry.value.as_ref().clone();
                Self::move_to_front(&mut cache.access_order, key);
                Some(value)
            } else {
                cache.entries.remove(key);
                cache.access_order.retain(|k| k != key);
                None
            }
        } else {
            None
        }
    }

    pub async fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, self.default_ttl).await;
    }

    pub async fn put_with_ttl(&self, key: K, value: V, ttl: Duration) {
        self.store_value(key, value, ttl).await;
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

    pub async fn cleanup_expired(&self) {
        let mut cache = self.inner.write();
        let expired_keys: Vec<K> = cache
            .entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            cache.entries.remove(&key);
            cache.access_order.retain(|k| k != &key);
        }
    }

    pub async fn size(&self) -> usize {
        let cache = self.inner.read();
        cache
            .entries
            .iter()
            .filter(|(_, entry)| !entry.is_expired())
            .count()
    }

    pub async fn size_including_expired(&self) -> usize {
        self.inner.read().entries.len()
    }

    async fn store_value(&self, key: K, value: V, ttl: Duration) {
        let mut cache = self.inner.write();

        if cache.entries.contains_key(&key) {
            let entry = cache.entries.get_mut(&key).unwrap();
            entry.value = Arc::new(value);
            entry.inserted_at = Instant::now();
            entry.ttl = ttl;
            entry.last_accessed = Instant::now();
            Self::move_to_front(&mut cache.access_order, &key);
            return;
        }

        if cache.entries.len() >= self.max_entries {
            self.evict_entry(&mut cache);
        }

        let now = Instant::now();
        cache.entries.insert(
            key.clone(),
            CacheEntry {
                value: Arc::new(value),
                inserted_at: now,
                ttl,
                last_accessed: now,
            },
        );
        cache.access_order.push_front(key);
    }

    fn evict_entry(& self, cache: &mut CacheInner<K, V>) {
        let expired_keys: Vec<K> = cache
            .entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        if !expired_keys.is_empty() {
            for key in expired_keys {
                cache.entries.remove(&key);
                cache.access_order.retain(|k| k != &key);
            }
            return;
        }

        if let Some(lru_key) = cache.access_order.pop_back() {
            cache.entries.remove(&lru_key);
        }
    }

    fn move_to_front(access_order: &mut VecDeque<K>, key: &K) {
        if let Some(pos) = access_order.iter().position(|k| k == key) {
            access_order.remove(pos);
        }
        access_order.push_front(key.clone());
    }

    pub fn start_background_cleanup(self: Arc<Self>, interval: Duration) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                interval_timer.tick().await;
                self.cleanup_expired().await;
            }
        })
    }
}

unsafe impl<K: Send, V: Send> Send for TtlLoadingCache<K, V> {}
unsafe impl<K: Send, V: Send> Sync for TtlLoadingCache<K, V> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_operations_with_ttl() {
        let load_count = Arc::new(AtomicUsize::new(0));
        let count_clone = load_count.clone();

        let cache = TtlLoadingCache::new(
            3,
            Duration::from_secs(1),
            move |key: &String| {
                let count = count_clone.clone();
                let key = key.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Some(format!("value_{}", key))
                }
            },
        );

        assert_eq!(cache.size().await, 0);

        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(cache.size().await, 1);
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        sleep(Duration::from_millis(1100)).await;

        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value_key1".to_string()));
        assert_eq!(load_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_different_ttls() {
        let cache = TtlLoadingCache::new(
            10,
            Duration::from_secs(10),
            |key: &u32| {
                let key = *key;
                async move { Some(format!("value_{}", key)) }
            },
        );

        cache.put_with_ttl(1, "short".to_string(), Duration::from_millis(100)).await;
        cache.put_with_ttl(2, "long".to_string(), Duration::from_secs(10)).await;

        assert_eq!(cache.get_if_present(&1), Some("short".to_string()));
        assert_eq!(cache.get_if_present(&2), Some("long".to_string()));

        sleep(Duration::from_millis(150)).await;

        assert_eq!(cache.get_if_present(&1), None);
        assert_eq!(cache.get_if_present(&2), Some("long".to_string()));
    }

    #[tokio::test]
    async fn test_eviction_prefers_expired() {
        let cache = TtlLoadingCache::new(
            3,
            Duration::from_secs(10),
            |key: &u32| {
                let key = *key;
                async move { Some(format!("value_{}", key)) }
            },
        );

        cache.put_with_ttl(1, "v1".to_string(), Duration::from_millis(100)).await;
        cache.put_with_ttl(2, "v2".to_string(), Duration::from_secs(10)).await;
        cache.put_with_ttl(3, "v3".to_string(), Duration::from_secs(10)).await;

        sleep(Duration::from_millis(150)).await;

        cache.put_with_ttl(4, "v4".to_string(), Duration::from_secs(10)).await;
        
        assert_eq!(cache.size().await, 3);
        assert_eq!(cache.get_if_present(&1), None);
        assert_eq!(cache.get_if_present(&2), Some("v2".to_string()));
        assert_eq!(cache.get_if_present(&3), Some("v3".to_string()));
        assert_eq!(cache.get_if_present(&4), Some("v4".to_string()));
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let cache = TtlLoadingCache::new(
            10,
            Duration::from_secs(10),
            |key: &u32| {
                let key = *key;
                async move { Some(format!("value_{}", key)) }
            },
        );

        cache.put_with_ttl(1, "v1".to_string(), Duration::from_millis(100)).await;
        cache.put_with_ttl(2, "v2".to_string(), Duration::from_millis(100)).await;
        cache.put_with_ttl(3, "v3".to_string(), Duration::from_secs(10)).await;

        assert_eq!(cache.size().await, 3);
        assert_eq!(cache.size_including_expired().await, 3);

        sleep(Duration::from_millis(150)).await;

        assert_eq!(cache.size().await, 1);
        assert_eq!(cache.size_including_expired().await, 3);

        cache.cleanup_expired().await;

        assert_eq!(cache.size().await, 1);
        assert_eq!(cache.size_including_expired().await, 1);
        assert_eq!(cache.get_if_present(&3), Some("v3".to_string()));
    }

    #[tokio::test]
    async fn test_lru_with_ttl() {
        let cache = TtlLoadingCache::new(
            3,
            Duration::from_secs(10),
            |key: &u32| {
                let key = *key;
                async move { Some(format!("value_{}", key)) }
            },
        );

        cache.get(&1).await;
        cache.get(&2).await;
        cache.get(&3).await;
        assert_eq!(cache.size().await, 3);

        cache.get(&1).await;

        cache.get(&4).await;
        assert_eq!(cache.size().await, 3);

        assert_eq!(cache.get_if_present(&1), Some("value_1".to_string()));
        assert_eq!(cache.get_if_present(&2), None);
        assert_eq!(cache.get_if_present(&3), Some("value_3".to_string()));
        assert_eq!(cache.get_if_present(&4), Some("value_4".to_string()));
    }

    #[tokio::test]
    async fn test_with_default_ttl() {
        let cache = TtlLoadingCache::new(
            3,
            Duration::from_secs(1),
            |_key: &u32| async move { Some("loaded".to_string()) }
        );

        let val = cache.get(&1).await;
        assert_eq!(val, Some("loaded".to_string()));
        assert_eq!(cache.size().await, 1);

        cache.put_with_ttl(2, "manual".to_string(), Duration::from_secs(1)).await;
        assert_eq!(cache.get_if_present(&2), Some("manual".to_string()));
        assert_eq!(cache.size().await, 2);
    }

    #[tokio::test]
    async fn test_background_cleanup() {
        let cache = Arc::new(TtlLoadingCache::new(
            10,
            Duration::from_secs(10),
            |key: &u32| {
                let key = *key;
                async move { Some(format!("value_{}", key)) }
            },
        ));

        cache.put_with_ttl(1, "v1".to_string(), Duration::from_millis(100)).await;
        cache.put_with_ttl(2, "v2".to_string(), Duration::from_millis(100)).await;
        cache.put_with_ttl(3, "v3".to_string(), Duration::from_secs(10)).await;

        let _handle = cache.clone().start_background_cleanup(Duration::from_millis(150));

        assert_eq!(cache.size_including_expired().await, 3);

        sleep(Duration::from_millis(300)).await;

        assert_eq!(cache.size_including_expired().await, 1);
        assert_eq!(cache.get_if_present(&3), Some("v3".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_access_with_ttl() {
        use tokio::task;

        let cache = Arc::new(TtlLoadingCache::new(
            10,
            Duration::from_secs(1),
            |key: &u32| {
                let key = *key;
                async move {
                    sleep(Duration::from_millis(10)).await;
                    Some(format!("value_{}", key))
                }
            },
        ));

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
    async fn test_update_ttl() {
        let cache = TtlLoadingCache::new(
            5,
            Duration::from_secs(10),
            |_key: &u32| async move { None }
        );

        cache.put_with_ttl(1, "initial".to_string(), Duration::from_millis(100)).await;
        assert_eq!(cache.get_if_present(&1), Some("initial".to_string()));

        sleep(Duration::from_millis(50)).await;

        cache.put_with_ttl(1, "updated".to_string(), Duration::from_secs(10)).await;
        
        sleep(Duration::from_millis(100)).await;

        assert_eq!(cache.get_if_present(&1), Some("updated".to_string()));
    }
}