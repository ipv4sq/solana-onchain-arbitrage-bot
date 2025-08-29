use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

pub struct LazyCache<K, V> {
    inner: Lazy<RwLock<HashMap<K, Arc<V>>>>,
}

impl<K, V> LazyCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub const fn new() -> Self {
        Self {
            inner: Lazy::new(|| RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.read().get(key).map(|v| v.as_ref().clone())
    }

    pub fn get_or_insert<F>(&self, key: K, f: F) -> V
    where
        F: FnOnce() -> V,
    {
        {
            let read_guard = self.inner.read();
            if let Some(value) = read_guard.get(&key) {
                return value.as_ref().clone();
            }
        }

        let mut write_guard = self.inner.write();
        write_guard
            .entry(key)
            .or_insert_with(|| Arc::new(f()))
            .as_ref()
            .clone()
    }

    pub fn get_or_insert_with_result<F, E>(&self, key: K, f: F) -> Result<V, E>
    where
        F: FnOnce() -> Result<V, E>,
    {
        {
            let read_guard = self.inner.read();
            if let Some(value) = read_guard.get(&key) {
                return Ok(value.as_ref().clone());
            }
        }

        let value = f()?;
        let mut write_guard = self.inner.write();
        Ok(write_guard
            .entry(key)
            .or_insert_with(|| Arc::new(value))
            .as_ref()
            .clone())
    }

    pub fn put(&self, key: K, value: V) -> Option<V> {
        self.inner
            .write()
            .insert(key, Arc::new(value))
            .map(|v| v.as_ref().clone())
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.write().remove(key).map(|v| v.as_ref().clone())
    }

    pub fn clear(&self) {
        self.inner.write().clear();
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.read().contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    pub fn keys(&self) -> Vec<K> {
        self.inner.read().keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<V> {
        self.inner
            .read()
            .values()
            .map(|v| v.as_ref().clone())
            .collect()
    }

    pub fn iter(&self) -> Vec<(K, V)> {
        self.inner
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_ref().clone()))
            .collect()
    }

    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&K, &V) -> bool,
    {
        let mut f = f;
        self.inner.write().retain(|k, v| f(k, v.as_ref()));
    }
}

unsafe impl<K: Send, V: Send> Send for LazyCache<K, V> {}
unsafe impl<K: Send, V: Send> Sync for LazyCache<K, V> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        static CACHE: LazyCache<String, i32> = LazyCache::new();

        assert!(CACHE.is_empty());
        assert_eq!(CACHE.len(), 0);

        CACHE.put("key1".to_string(), 100);
        assert_eq!(CACHE.get(&"key1".to_string()), Some(100));
        assert_eq!(CACHE.len(), 1);

        let value = CACHE.get_or_insert("key2".to_string(), || 200);
        assert_eq!(value, 200);
        assert_eq!(CACHE.len(), 2);

        let value = CACHE.get_or_insert("key2".to_string(), || 300);
        assert_eq!(value, 200);
        assert_eq!(CACHE.len(), 2);

        assert!(CACHE.contains_key(&"key1".to_string()));
        assert!(!CACHE.contains_key(&"key3".to_string()));

        let removed = CACHE.remove(&"key1".to_string());
        assert_eq!(removed, Some(100));
        assert_eq!(CACHE.len(), 1);

        CACHE.clear();
        assert!(CACHE.is_empty());
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        static CACHE: LazyCache<i32, String> = LazyCache::new();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let value = CACHE.get_or_insert(i % 5, || format!("value_{}", i % 5));
                    assert!(value.starts_with("value_"));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(CACHE.len(), 5);
    }

    #[test]
    fn test_get_or_insert_with_result() {
        static CACHE: LazyCache<String, String> = LazyCache::new();

        let result = CACHE
            .get_or_insert_with_result("key".to_string(), || Ok::<_, &str>("success".to_string()));
        assert_eq!(result, Ok("success".to_string()));

        let result = CACHE.get_or_insert_with_result("key".to_string(), || {
            Err::<String, _>("should not be called")
        });
        assert_eq!(result, Ok("success".to_string()));

        let result = CACHE.get_or_insert_with_result("error_key".to_string(), || {
            Err::<String, _>("error occurred")
        });
        assert_eq!(result, Err("error occurred"));
    }

    #[test]
    fn test_iteration_methods() {
        static CACHE: LazyCache<String, i32> = LazyCache::new();

        CACHE.put("a".to_string(), 1);
        CACHE.put("b".to_string(), 2);
        CACHE.put("c".to_string(), 3);

        let keys = CACHE.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert!(keys.contains(&"c".to_string()));

        let values = CACHE.values();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&1));
        assert!(values.contains(&2));
        assert!(values.contains(&3));

        let items = CACHE.iter();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_retain() {
        static CACHE: LazyCache<i32, String> = LazyCache::new();

        for i in 0..10 {
            CACHE.put(i, format!("value_{}", i));
        }

        CACHE.retain(|k, _| *k % 2 == 0);
        assert_eq!(CACHE.len(), 5);

        for i in 0..10 {
            if i % 2 == 0 {
                assert!(CACHE.contains_key(&i));
            } else {
                assert!(!CACHE.contains_key(&i));
            }
        }
    }
}
