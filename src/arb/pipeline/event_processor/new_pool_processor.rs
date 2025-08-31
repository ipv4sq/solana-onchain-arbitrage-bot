use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::global::client::rpc::rpc_client;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::enums::block_reason::BlocklistReason;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::trace::types::WithTrace;
use crate::arb::util::structs::cache_type::CacheType;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

#[allow(non_upper_case_globals)]
pub static NewPoolProcessor: Lazy<Arc<PubSubProcessor<WithTrace<Pubkey>>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 32,
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

async fn record_if_real_pool(addr: &Pubkey) -> bool {
    if let Some(entry) = NonPoolBlocklist.get(addr).await {
        debug!("Address {} is blocklisted: {:?}", addr, entry.reason);
        return false;
    }

    let data_len = if let Ok(account) = rpc_client().get_account(addr).await {
        let data_len = account.data.len();
        if data_len < 200 || data_len > 2000 {
            let entry = BlocklistEntry::new(BlocklistReason::InvalidDataSize { size: data_len })
                .with_data_size(data_len);
            warn!("Account {} data size {} is outside pool range (200-2000 bytes), adding to blocklist", addr, data_len);
            NonPoolBlocklist.put(*addr, entry).await;
            return false;
        }
        data_len
    } else {
        let entry = BlocklistEntry::new(BlocklistReason::AccountNotFound);
        warn!(
            "Account {} does not exist on chain, adding to blocklist",
            addr
        );
        NonPoolBlocklist.put(*addr, entry).await;
        return false;
    };

    let pool = match PoolRecordRepository::ensure_exists(addr).await {
        Some(p) => p,
        None => {
            let entry =
                BlocklistEntry::new(BlocklistReason::NotInDatabase).with_data_size(data_len);
            warn!("Cannot find pool {} in database, adding to blocklist", addr);
            NonPoolBlocklist.put(*addr, entry).await;
            return false;
        }
    };

    if pool.base_mint.0 != Mints::WSOL && pool.quote_mint.0 != Mints::WSOL {
        let entry = BlocklistEntry::new(BlocklistReason::NoWsolInvolved).with_data_size(data_len);
        warn!("Pool {} does not involve WSOL, adding to blocklist", addr);
        NonPoolBlocklist.put(*addr, entry).await;
        return false;
    }

    true
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
