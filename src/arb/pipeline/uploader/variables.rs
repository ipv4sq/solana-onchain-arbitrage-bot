#![allow(non_upper_case_globals)]
use crate::arb::global::trace::types::Trace;
use crate::arb::pipeline::uploader::entry::fire_mev_bot;
use crate::arb::util::alias::{MintAddress, PoolAddress};
use crate::arb::util::structs::rate_limiter::RateLimiter;
use crate::arb::util::structs::tx_dedup::TxDeduplicator;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::lazy_arc;
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
        |event: MevBotFire| {
            Box::pin(
                async move { fire_mev_bot(&event.minor_mint, &event.pools, event.trace).await },
            )
        },
    )
});
pub static MevBotRateLimiter: Lazy<Arc<RateLimiter>> = lazy_arc!({
    RateLimiter::new(
        5,
        Duration::from_secs(1),
        8,
        "MevBotRateLimiter".to_string(),
    )
});
pub static ENABLE_SEND_TX: Lazy<bool> = Lazy::new(|| {
    let env = std::env::var("ENABLE_SEND_TX").unwrap_or("false".to_string());
    return env == "true";
});
pub static MevBotDeduplicator: Lazy<Arc<TxDeduplicator>> =
    lazy_arc!(TxDeduplicator::new(Duration::from_secs(60)));
