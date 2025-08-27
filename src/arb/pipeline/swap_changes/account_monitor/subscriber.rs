use crate::arb::convention::chain::AccountState;
use crate::arb::pipeline::swap_changes::account_monitor::entry;
use crate::arb::pipeline::swap_changes::account_monitor::pool_tracker::list_all_pools;
use crate::arb::pipeline::swap_changes::account_monitor::pool_update::PoolUpdate;
use crate::arb::pipeline::swap_changes::cache::PoolAccountCache;
use crate::arb::sdk::yellowstone::{AccountFilter, GrpcAccountUpdate, SolanaGrpcClient};
use crate::arb::util::structs::buffered_debouncer::BufferedDebouncer;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{empty_ok, lazy_arc};
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

#[allow(unused)]
static POOL_UPDATE_CONSUMER: Lazy<Arc<PubSubProcessor<PoolUpdate>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 4,
        channel_buffer_size: 500,
        name: "PoolUpdateProcessor".to_string(),
    };

    PubSubProcessor::new(config, |update: PoolUpdate| {
        Box::pin(async move {
            entry::process_pool_update(update).await?;
            Ok(())
        })
    })
});

#[allow(unused)]
static POOL_UPDATE_DEBOUNCER: Lazy<Arc<BufferedDebouncer<Pubkey, GrpcAccountUpdate>>> = lazy_arc!({
    BufferedDebouncer::new(
        Duration::from_millis(30),
        |update: GrpcAccountUpdate| async move {
            let updated = AccountState::from_grpc_update(&update);
            let previous = PoolAccountCache.put(update.account, updated.clone());
            let pool_update = PoolUpdate {
                previous,
                current: updated,
            };
            if let Err(e) = POOL_UPDATE_CONSUMER.publish(pool_update).await {
                error!("Failed to publish pool update: {}", e);
            }
        },
    )
});

#[allow(unused)]
pub struct PoolAccountMonitor {
    client: SolanaGrpcClient,
    pools: HashSet<Pubkey>,
}

impl PoolAccountMonitor {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
            pools: list_all_pools().await?,
        })
    }

    pub async fn start(self) -> Result<()> {
        info!(
            "Starting pool account subscription for {} pools",
            self.pools.len(),
        );

        let pool_vec: Vec<Pubkey> = self.pools.into_iter().collect();
        let filter = AccountFilter::new("pool_monitor").with_accounts(&pool_vec);

        self.client
            .subscribe_accounts(
                filter,
                move |account_update| {
                    async move { Self::handle_account_update(account_update).await }
                },
                true,
            )
            .await
    }

    async fn handle_account_update(update: GrpcAccountUpdate) -> Result<()> {
        debug!("Pool account update received: {}", update.account);
        
        POOL_UPDATE_DEBOUNCER.update(update.account, update);
        
        empty_ok!()
    }
}

pub async fn start_pool_monitor() -> Result<()> {
    let monitor = PoolAccountMonitor::new().await?;
    monitor.start().await
}
