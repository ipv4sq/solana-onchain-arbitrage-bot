use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

// Configuration for the pub/sub system
pub struct PubSubConfig {
    pub worker_pool_size: usize,
    pub channel_buffer_size: usize,
    pub name: String,
}

impl Default for PubSubConfig {
    fn default() -> Self {
        Self {
            worker_pool_size: 8,
            channel_buffer_size: 100,
            name: "PubSub".to_string(),
        }
    }
}

// Generic pub/sub processor that can handle any message type
pub struct PubSubProcessor<T: Send + 'static> {
    tx_sender: mpsc::Sender<T>,
    worker_handles: Vec<JoinHandle<()>>,
    name: String,
}

impl<T: Send + 'static> PubSubProcessor<T> {
    pub fn new<F>(config: PubSubConfig, processor: F) -> Self
    where
        F: Fn(T) -> futures::future::BoxFuture<'static, Result<()>> + Send + Sync + 'static + Clone,
    {
        let (tx_sender, tx_receiver) = mpsc::channel::<T>(config.channel_buffer_size);
        let rx = Arc::new(Mutex::new(tx_receiver));
        let mut worker_handles = Vec::with_capacity(config.worker_pool_size);
        
        for worker_id in 0..config.worker_pool_size {
            let rx_clone = Arc::clone(&rx);
            let processor_clone = processor.clone();
            let name_clone = config.name.clone();
            
            let handle = tokio::spawn(async move {
                info!("[{}] Worker {} started", name_clone, worker_id);
                
                loop {
                    let message = {
                        let mut receiver = rx_clone.lock().await;
                        receiver.recv().await
                    };
                    
                    match message {
                        Some(msg) => {
                            if let Err(e) = processor_clone(msg).await {
                                error!("[{}] Worker {} failed to process message: {}", name_clone, worker_id, e);
                            }
                        }
                        None => {
                            info!("[{}] Worker {} shutting down - channel closed", name_clone, worker_id);
                            break;
                        }
                    }
                }
            });
            
            worker_handles.push(handle);
        }
        
        info!("[{}] Processor initialized with {} workers", config.name, config.worker_pool_size);
        
        Self {
            tx_sender,
            worker_handles,
            name: config.name,
        }
    }
    
    pub async fn publish(&self, message: T) -> Result<()> {
        self.tx_sender
            .send(message)
            .await
            .map_err(|e| anyhow::anyhow!("[{}] Failed to send message: {}", self.name, e))?;
        Ok(())
    }
    
    pub fn try_publish(&self, message: T) -> Result<()> {
        self.tx_sender
            .try_send(message)
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => {
                    anyhow::anyhow!("[{}] Channel buffer is full, message dropped", self.name)
                }
                mpsc::error::TrySendError::Closed(_) => {
                    anyhow::anyhow!("[{}] Channel is closed", self.name)
                }
            })
    }
    
    pub async fn shutdown(self) {
        info!("[{}] Shutting down processor...", self.name);
        drop(self.tx_sender);
        
        for (idx, handle) in self.worker_handles.into_iter().enumerate() {
            if let Err(e) = handle.await {
                error!("[{}] Worker {} failed to join: {}", self.name, idx, e);
            }
        }
        
        info!("[{}] Processor shutdown complete", self.name);
    }
}

// Singleton wrapper for global instances
pub struct SingletonPubSub<T: Send + 'static> {
    inner: Arc<RwLock<Option<PubSubProcessor<T>>>>,
    name: String,
}

impl<T: Send + 'static> SingletonPubSub<T> {
    pub fn new(name: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            name,
        }
    }
    
    pub async fn initialize<F>(&self, config: PubSubConfig, processor: F) -> Result<()>
    where
        F: Fn(T) -> futures::future::BoxFuture<'static, Result<()>> + Send + Sync + 'static + Clone,
    {
        let mut inner = self.inner.write().await;
        
        if inner.is_some() {
            warn!("[{}] Processor already initialized", self.name);
            return Ok(());
        }
        
        *inner = Some(PubSubProcessor::new(config, processor));
        Ok(())
    }
    
    pub async fn publish(&self, message: T) -> Result<()> {
        let inner = self.inner.read().await;
        
        let processor = inner
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("[{}] Processor not initialized", self.name))?;
        
        processor.publish(message).await
    }
    
    pub async fn is_initialized(&self) -> bool {
        let inner = self.inner.read().await;
        inner.is_some()
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        if let Some(processor) = inner.take() {
            processor.shutdown().await;
        } else {
            warn!("[{}] Processor was not initialized", self.name);
        }
        
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pubsub_processor() {
        let config = PubSubConfig {
            worker_pool_size: 4,
            channel_buffer_size: 50,
            name: "TestProcessor".to_string(),
        };
        
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        let processor = PubSubProcessor::new(config, move |msg: i32| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let mut count = counter.lock().await;
                *count += msg;
                info!("Processed message: {}, total: {}", msg, *count);
                Ok(())
            })
        });
        
        for i in 1..=10 {
            processor.publish(i).await.unwrap();
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let final_count = *counter.lock().await;
        assert_eq!(final_count, 55); // Sum of 1..=10
        
        processor.shutdown().await;
    }
    
    #[tokio::test]
    async fn test_singleton_pubsub() {
        let singleton = SingletonPubSub::<String>::new("TestSingleton".to_string());
        
        let config = PubSubConfig {
            worker_pool_size: 2,
            channel_buffer_size: 10,
            name: "TestSingleton".to_string(),
        };
        
        singleton.initialize(config, |msg: String| {
            Box::pin(async move {
                info!("Processing: {}", msg);
                Ok(())
            })
        }).await.unwrap();
        
        assert!(singleton.is_initialized().await);
        
        singleton.publish("Hello".to_string()).await.unwrap();
        singleton.publish("World".to_string()).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        singleton.shutdown().await.unwrap();
    }
}