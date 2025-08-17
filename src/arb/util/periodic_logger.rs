use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::interval;
use tracing::info;

pub struct PeriodicLogger<K: MetricKey = String> {
    name: String,
    interval: Duration,
    metrics: Arc<LoggerMetrics<K>>,
}

pub trait MetricKey: Send + Sync + Clone + Eq + std::hash::Hash + std::fmt::Display + 'static {}

impl MetricKey for String {}
impl MetricKey for &'static str {}

pub struct LoggerMetrics<K: MetricKey> {
    counters: RwLock<HashMap<K, Arc<AtomicU64>>>,
}

impl<K: MetricKey> Default for LoggerMetrics<K> {
    fn default() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
        }
    }
}

impl<K: MetricKey> PeriodicLogger<K> {
    pub fn new(name: impl Into<String>, interval: Duration) -> Self {
        Self {
            name: name.into(),
            interval,
            metrics: Arc::new(LoggerMetrics::default()),
        }
    }

    pub fn metrics_handle(&self) -> MetricsHandle<K> {
        MetricsHandle {
            metrics: Arc::clone(&self.metrics),
        }
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(self.interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                ticker.tick().await;
                self.log_metrics();
            }
        })
    }

    fn log_metrics(&self) {
        let counters = self.metrics.counters.read().unwrap();
        
        if counters.is_empty() {
            return;
        }

        let mut entries = Vec::new();
        let mut total = 0u64;
        
        for (key, counter) in counters.iter() {
            let value = counter.swap(0, Ordering::Relaxed);
            if value > 0 {
                entries.push(format!("{}: {}", key, value));
                total += value;
            }
        }

        if !entries.is_empty() {
            info!(
                "[{}] Total: {} | {}",
                self.name,
                total,
                entries.join(", ")
            );
        }
    }
}

#[derive(Clone)]
pub struct MetricsHandle<K: MetricKey> {
    metrics: Arc<LoggerMetrics<K>>,
}

impl<K: MetricKey> MetricsHandle<K> {
    pub fn add(&self, key: K, count: u64) {
        let counters = self.metrics.counters.read().unwrap();
        
        if let Some(counter) = counters.get(&key) {
            counter.fetch_add(count, Ordering::Relaxed);
        } else {
            drop(counters);
            let mut counters = self.metrics.counters.write().unwrap();
            let counter = counters.entry(key).or_insert_with(|| Arc::new(AtomicU64::new(0)));
            counter.fetch_add(count, Ordering::Relaxed);
        }
    }

    pub fn inc(&self, key: K) {
        self.add(key, 1);
    }

    pub fn get(&self, key: &K) -> u64 {
        let counters = self.metrics.counters.read().unwrap();
        counters.get(key)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub fn reset(&self, key: &K) {
        let counters = self.metrics.counters.read().unwrap();
        if let Some(counter) = counters.get(key) {
            counter.store(0, Ordering::Relaxed);
        }
    }

    pub fn reset_all(&self) {
        let counters = self.metrics.counters.read().unwrap();
        for counter in counters.values() {
            counter.store(0, Ordering::Relaxed);
        }
    }
}

pub struct PeriodicLoggerBuilder<K: MetricKey = String> {
    name: String,
    interval: Duration,
    _phantom: std::marker::PhantomData<K>,
}

impl<K: MetricKey> PeriodicLoggerBuilder<K> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            interval: Duration::from_secs(10),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> PeriodicLogger<K> {
        PeriodicLogger::new(self.name, self.interval)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum TransactionMetric {
    Received,
    Success,
    Failed,
    ErrorInvalidAccount,
    ErrorPublishFailed,
    ErrorOther,
}

impl std::fmt::Display for TransactionMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Received => write!(f, "received"),
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
            Self::ErrorInvalidAccount => write!(f, "error_invalid_account"),
            Self::ErrorPublishFailed => write!(f, "error_publish_failed"),
            Self::ErrorOther => write!(f, "error_other"),
        }
    }
}

impl MetricKey for TransactionMetric {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_metrics() {
        let logger = PeriodicLogger::<String>::new("test", Duration::from_secs(5));
        let handle = logger.metrics_handle();
        
        handle.inc("received".to_string());
        handle.add("processed".to_string(), 5);
        handle.inc("failed".to_string());
        
        assert_eq!(handle.get(&"received".to_string()), 1);
        assert_eq!(handle.get(&"processed".to_string()), 5);
        assert_eq!(handle.get(&"failed".to_string()), 1);
        assert_eq!(handle.get(&"unknown".to_string()), 0);
    }

    #[test]
    fn test_enum_metrics() {
        let logger = PeriodicLogger::<TransactionMetric>::new("test", Duration::from_secs(5));
        let handle = logger.metrics_handle();
        
        handle.inc(TransactionMetric::Received);
        handle.inc(TransactionMetric::Success);
        handle.add(TransactionMetric::Failed, 3);
        
        assert_eq!(handle.get(&TransactionMetric::Received), 1);
        assert_eq!(handle.get(&TransactionMetric::Success), 1);
        assert_eq!(handle.get(&TransactionMetric::Failed), 3);
    }

    #[test]
    fn test_static_str_metrics() {
        let logger = PeriodicLogger::<&'static str>::new("test", Duration::from_secs(5));
        let handle = logger.metrics_handle();
        
        handle.inc("transactions");
        handle.inc("errors");
        handle.add("bytes_processed", 1024);
        
        assert_eq!(handle.get(&"transactions"), 1);
        assert_eq!(handle.get(&"errors"), 1);
        assert_eq!(handle.get(&"bytes_processed"), 1024);
    }
}