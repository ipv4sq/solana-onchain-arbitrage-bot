use crate::arb::chain::Transaction;
use crate::arb::subscriber::pubsub::{PubSubConfig, PubSubProcessor};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::ops::Deref;
use std::sync::Arc;
use tracing::info;

pub static MEV_TX_CONSUMER: Lazy<Arc<MevConsumerPool>> = Lazy::new(|| Arc::new(MevConsumerPool::new()));

pub async fn publish_mev_transaction(tx: Transaction) -> Result<()> {
    MEV_TX_CONSUMER.publish(tx).await
}

pub fn try_publish_mev_transaction(tx: Transaction) -> Result<()> {
    MEV_TX_CONSUMER.try_publish(tx)
}

impl MevConsumerPool {
    fn new() -> Self {
        let config = PubSubConfig {
            worker_pool_size: 8,
            channel_buffer_size: 1000,
            name: "SolanaMevBotTransactionDetector".to_string(),
        };

        use crate::arb::program::solana_mev_bot::subscriber::entry as process_mev_tx;
        let processor = PubSubProcessor::new(config, |tx| {
            Box::pin(async move {
                process_mev_tx(&tx).await?;
                Ok(())
            })
        });

        info!("MEV transaction processor auto-initialized");

        Self(processor)
    }
}

pub struct MevConsumerPool(PubSubProcessor<Transaction>);
impl Deref for MevConsumerPool {
    type Target = PubSubProcessor<Transaction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
