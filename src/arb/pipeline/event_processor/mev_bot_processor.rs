use crate::arb::convention::chain::Transaction;
use crate::arb::pipeline::event_processor::mev_bot::entry;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::lazy_arc;
use once_cell::sync::Lazy;
use std::sync::Arc;

#[allow(non_upper_case_globals)]
pub static MevBotTxProcessor: Lazy<Arc<PubSubProcessor<Transaction>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 8,
        channel_buffer_size: 1000,
        name: "SolanaMevBotTransactionDetector".to_string(),
    };

    PubSubProcessor::new(config, |tx: Transaction| async move {
        entry::entry(&tx).await?;
        Ok(())
    })
});
