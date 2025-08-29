use dashmap::DashMap;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Instant};

type Callback<V> = Arc<dyn Fn(V) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

struct BufferedEntry<V> {
    value: V,
    timer_handle: JoinHandle<()>,
}

pub struct BufferedDebouncer<K, V> {
    buffers: Arc<DashMap<K, BufferedEntry<V>>>,
    delay: Duration,
    callback: Callback<V>,
}

impl<K, V> BufferedDebouncer<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new<F, Fut>(delay: Duration, callback: F) -> Self
    where
        F: Fn(V) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let callback = Arc::new(
            move |value: V| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                Box::pin(callback(value))
            },
        );

        Self {
            buffers: Arc::new(DashMap::new()),
            delay,
            callback,
        }
    }

    pub fn update(&self, key: K, value: V) {
        let delay = self.delay;
        let buffers = self.buffers.clone();
        let callback = self.callback.clone();
        let key_clone = key.clone();

        if let Some(mut entry) = self.buffers.get_mut(&key) {
            entry.timer_handle.abort();
            entry.value = value;

            let timer_handle = tokio::spawn(async move {
                sleep(delay).await;
                if let Some((_, entry)) = buffers.remove(&key_clone) {
                    (callback)(entry.value).await;
                }
            });

            entry.timer_handle = timer_handle;
        } else {
            let timer_handle = tokio::spawn(async move {
                sleep(delay).await;
                if let Some((_, entry)) = buffers.remove(&key_clone) {
                    (callback)(entry.value).await;
                }
            });

            self.buffers.insert(
                key,
                BufferedEntry {
                    value,
                    timer_handle,
                },
            );
        }
    }

    pub fn cancel(&self, key: &K) -> Option<V> {
        self.buffers.remove(key).map(|(_, entry)| {
            entry.timer_handle.abort();
            entry.value
        })
    }

    pub fn cancel_all(&self) {
        self.buffers.clear();
    }

    pub fn size(&self) -> usize {
        self.buffers.len()
    }

    pub fn has_pending(&self, key: &K) -> bool {
        self.buffers.contains_key(key)
    }
}

unsafe impl<K: Send, V: Send> Send for BufferedDebouncer<K, V> {}
unsafe impl<K: Send, V: Send> Sync for BufferedDebouncer<K, V> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_buffering() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        let debouncer = Arc::new(BufferedDebouncer::new(
            Duration::from_millis(30),
            move |value: i32| {
                let received = received_clone.clone();
                async move {
                    received.lock().await.push(value);
                }
            },
        ));

        debouncer.update(1, 100);
        assert_eq!(received.lock().await.len(), 0);

        sleep(Duration::from_millis(10)).await;
        debouncer.update(1, 200);
        assert_eq!(received.lock().await.len(), 0);

        sleep(Duration::from_millis(10)).await;
        debouncer.update(1, 300);
        assert_eq!(received.lock().await.len(), 0);

        sleep(Duration::from_millis(35)).await;

        let values = received.lock().await;
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], 300);
    }

    #[tokio::test]
    async fn test_timeline_scenario() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let debouncer = BufferedDebouncer::new(Duration::from_millis(30), move |value: String| {
            let events = events_clone.clone();
            async move {
                let mut events = events.lock().await;
                println!("Fired: {}", value);
                events.push(value);
            }
        });

        println!("T+0ms: Emit 'first'");
        debouncer.update("key1", "first".to_string());

        sleep(Duration::from_millis(10)).await;
        println!("T+10ms: Update to 'second'");
        debouncer.update("key1", "second".to_string());

        sleep(Duration::from_millis(10)).await;
        println!("T+20ms: Update to 'third'");
        debouncer.update("key1", "third".to_string());

        println!("T+20ms: Waiting for automatic fire at T+50ms...");
        sleep(Duration::from_millis(35)).await;

        let fired_events = events.lock().await;
        assert_eq!(fired_events.len(), 1, "Should fire exactly once");
        assert_eq!(fired_events[0], "third", "Should fire the latest value");
    }

    #[tokio::test]
    async fn test_multiple_keys_independent() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let debouncer = Arc::new(BufferedDebouncer::new(
            Duration::from_millis(30),
            move |_: i32| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            },
        ));

        debouncer.update("key1", 1);
        sleep(Duration::from_millis(10)).await;

        debouncer.update("key2", 2);
        sleep(Duration::from_millis(10)).await;

        debouncer.update("key3", 3);

        sleep(Duration::from_millis(40)).await;

        assert_eq!(
            counter.load(Ordering::SeqCst),
            3,
            "All three keys should fire"
        );
    }

    #[tokio::test]
    async fn test_cancel() {
        let fired = Arc::new(AtomicU32::new(0));
        let fired_clone = fired.clone();

        let debouncer = BufferedDebouncer::new(Duration::from_millis(30), move |_: i32| {
            let fired = fired_clone.clone();
            async move {
                fired.fetch_add(1, Ordering::SeqCst);
            }
        });

        debouncer.update(1, 100);
        sleep(Duration::from_millis(10)).await;

        let cancelled_value = debouncer.cancel(&1);
        assert_eq!(cancelled_value, Some(100));

        sleep(Duration::from_millis(30)).await;
        assert_eq!(
            fired.load(Ordering::SeqCst),
            0,
            "Should not fire after cancel"
        );
    }

    #[tokio::test]
    async fn test_restart_after_fire() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let debouncer = Arc::new(BufferedDebouncer::new(
            Duration::from_millis(20),
            move |value: i32| {
                let events = events_clone.clone();
                async move {
                    events.lock().await.push(value);
                }
            },
        ));

        debouncer.update(1, 100);
        sleep(Duration::from_millis(25)).await;

        {
            let events = events.lock().await;
            assert_eq!(events.len(), 1);
            assert_eq!(events[0], 100);
        }

        debouncer.update(1, 200);
        sleep(Duration::from_millis(25)).await;

        {
            let events = events.lock().await;
            assert_eq!(events.len(), 2);
            assert_eq!(events[1], 200);
        }
    }

    #[tokio::test]
    async fn test_rapid_updates_only_fires_last() {
        let last_value = Arc::new(Mutex::new(0));
        let last_value_clone = last_value.clone();

        let debouncer = BufferedDebouncer::new(Duration::from_millis(30), move |value: i32| {
            let last_value = last_value_clone.clone();
            async move {
                *last_value.lock().await = value;
            }
        });

        for i in 0..10 {
            debouncer.update(1, i);
            sleep(Duration::from_millis(2)).await;
        }

        sleep(Duration::from_millis(35)).await;

        assert_eq!(
            *last_value.lock().await,
            9,
            "Should only fire the last update"
        );
    }

    #[tokio::test]
    async fn test_with_pubkey_scenario() {
        use solana_program::pubkey::Pubkey;
        use std::str::FromStr;

        let processed = Arc::new(Mutex::new(Vec::new()));
        let processed_clone = processed.clone();

        let debouncer = BufferedDebouncer::new(
            Duration::from_millis(30),
            move |(key, slot): (Pubkey, u64)| {
                let processed = processed_clone.clone();
                async move {
                    processed.lock().await.push((key, slot));
                }
            },
        );

        let pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();

        debouncer.update(pubkey, (pubkey, 100));
        sleep(Duration::from_millis(5)).await;

        debouncer.update(pubkey, (pubkey, 101));
        sleep(Duration::from_millis(5)).await;

        debouncer.update(pubkey, (pubkey, 102));

        sleep(Duration::from_millis(35)).await;

        let processed = processed.lock().await;
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].1, 102, "Should process the latest slot");
    }

    #[tokio::test]
    async fn test_visual_buffer_timeline() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let debouncer = BufferedDebouncer::new(Duration::from_millis(30), move |msg: String| {
            let events = events_clone.clone();
            async move {
                let instant = Instant::now();
                println!("    -> FIRED: '{}' at {:?}", msg, instant);
                events.lock().await.push(msg);
            }
        });

        println!("\n=== Buffered Debouncer Timeline (30ms delay) ===");

        println!("T+0ms: Emit 'update1'");
        debouncer.update("key", "update1".to_string());

        sleep(Duration::from_millis(10)).await;
        println!("T+10ms: Update to 'update2'");
        debouncer.update("key", "update2".to_string());

        sleep(Duration::from_millis(10)).await;
        println!("T+20ms: Update to 'update3'");
        debouncer.update("key", "update3".to_string());

        println!("T+20ms: Timer restarted, will fire at T+50ms");

        sleep(Duration::from_millis(35)).await;
        println!("T+55ms: Check results");

        let fired = events.lock().await;
        assert_eq!(fired.len(), 1, "Should fire exactly once");
        assert_eq!(fired[0], "update3", "Should fire latest value");

        println!("=== Timeline Complete ===\n");
    }
}
