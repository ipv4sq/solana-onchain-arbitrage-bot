use crate::arb::global::trace::types::WithTrace;
use crate::arb::pipeline::swap_changes::account_monitor::entry;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::lazy_arc;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;

#[allow(non_upper_case_globals)]
pub static NewPoolProcessor: Lazy<Arc<PubSubProcessor<WithTrace<Pubkey>>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 32,
        channel_buffer_size: 100_000,
        name: "NewPoolProcesseor".to_string(),
    };
    PubSubProcessor::new(config, entry::on_new_pool_received)
});
