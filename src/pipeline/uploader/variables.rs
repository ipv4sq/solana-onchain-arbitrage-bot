#![allow(non_upper_case_globals)]
use crate::global::trace::types::Trace;
use crate::lazy_arc;
use crate::pipeline::uploader::entry::fire_mev_bot;
use crate::util::alias::{MintAddress, PoolAddress};
use crate::util::env::env_config::ENV_CONFIG;
use crate::util::structs::rate_limiter::RateLimiter;
use crate::util::structs::tx_dedup::TxDeduplicator;
use crate::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Duration;

pub struct MevBotFire {
    pub minor_mint: MintAddress,
    pub pools: Vec<PoolAddress>,
    pub trace: Trace,
}

pub static FireMevBotConsumer: Lazy<Arc<PubSubProcessor<MevBotFire>>> = lazy_arc!({
    PubSubProcessor::new(
        PubSubConfig {
            worker_pool_size: 12,
            channel_buffer_size: 1000,
            name: "VaultUpdateProcessor".to_string(),
        },
        |event: MevBotFire| async move {
            fire_mev_bot(&event.minor_mint, &event.pools, event.trace).await
        },
    )
});
pub static MevBotRateLimiter: Lazy<Arc<RateLimiter>> = lazy_arc!({
    RateLimiter::new(
        8,
        Duration::from_secs(1),
        10,
        "MevBotRateLimiter".to_string(),
    )
});
pub static ENABLE_SEND_TX: Lazy<bool> = Lazy::new(|| {
    return ENV_CONFIG.enable_send_tx;
});

pub static MevBotDeduplicator: Lazy<Arc<TxDeduplicator>> =
    lazy_arc!(TxDeduplicator::new(Duration::from_secs(60)));
