use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::global::enums::block_reason::BlocklistReason;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::state::any_pool_holder::AnyPoolHolder;
use crate::arb::global::trace::types::WithTrace;
use crate::arb::util::alias::AResult;
use crate::arb::util::structs::cache_type::CacheType;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use crate::arb::util::structs::rate_limiter::RateLimitError;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;

#[allow(non_upper_case_globals)]
pub static NewPoolProcessor: Lazy<Arc<PubSubProcessor<WithTrace<Pubkey>>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 16,
        channel_buffer_size: 100_000,
        name: "NewPoolProcesseor".to_string(),
    };
    PubSubProcessor::new(config, on_new_pool_received)
});

pub async fn on_new_pool_received(with_trace: WithTrace<Pubkey>) -> anyhow::Result<()> {
    let WithTrace(pool_address, trace) = with_trace;
    trace.step(StepType::IsAccountPoolData);

    record_if_real_pool(&pool_address).await;

    unit_ok!()
}

async fn record_if_real_pool(addr: &Pubkey) {
    if NonPoolBlocklist.get(addr).await.is_some() {
        return;
    }
    let result = AnyPoolHolder::fresh_get(addr).await;
    match result {
        Ok(c) => {
            let _ = PoolRecordRepository::ensure_exists(addr).await;
        }
        Err(e) => {
            // if it's not rate limit error, we block it.
            if !e.is::<RateLimitError>() {
                NonPoolBlocklist
                    .put(*addr, BlocklistEntry::new(BlocklistReason::SaveFailed))
                    .await;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocklistEntry {
    pub reason: BlocklistReason,
    pub blocklisted_at: DateTime<Utc>,
    pub data_size: Option<usize>,
}

impl BlocklistEntry {
    fn new(reason: BlocklistReason) -> Self {
        Self {
            reason,
            blocklisted_at: Utc::now(),
            data_size: None,
        }
    }

    fn with_data_size(mut self, size: usize) -> Self {
        self.data_size = Some(size);
        self
    }
}

#[allow(non_upper_case_globals)]
pub static NonPoolBlocklist: Lazy<PersistentCache<Pubkey, BlocklistEntry>> = Lazy::new(|| {
    PersistentCache::new(
        CacheType::Custom("non_pool_blocklist".to_string()),
        10000,
        Duration::from_secs(86400 * 7),
        |_addr: &Pubkey| async move { None },
    )
});
