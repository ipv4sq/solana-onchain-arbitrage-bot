use crate::arb::convention::chain::Transaction;
use crate::arb::pipeline::pool_indexer::mev_bot::entry;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static MEV_TX_CONSUMER: Lazy<Arc<PubSubProcessor<Transaction>>> =
    Lazy::new(|| Arc::new(init()));

fn init() -> PubSubProcessor<Transaction> {
    let config = PubSubConfig {
        worker_pool_size: 8,
        channel_buffer_size: 1000,
        name: "SolanaMevBotTransactionDetector".to_string(),
    };

    PubSubProcessor::new(config, |tx: Transaction| {
        Box::pin(async move {
            entry::entry(&tx).await?;
            Ok(())
        })
    })
}
