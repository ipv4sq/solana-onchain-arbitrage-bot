use crate::arb::global::trace::types::WithTrace;
use crate::arb::pipeline::swap_changes::account_monitor::entry;
use crate::arb::pipeline::swap_changes::account_monitor::trigger::Trigger;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::lazy_arc;
use once_cell::sync::Lazy;
use std::sync::Arc;

#[allow(non_upper_case_globals)]
pub static PoolUpdateProcessor: Lazy<Arc<PubSubProcessor<WithTrace<Trigger>>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 24,
        channel_buffer_size: 5000,
        name: "PoolUpdateProcessor".to_string(),
    };

    PubSubProcessor::new(config, entry::process_pool_update)
});
