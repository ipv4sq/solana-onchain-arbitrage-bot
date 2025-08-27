use crate::arb::convention::chain::AccountState;
use crate::arb::global::trace::types::StepType::AccountUpdateDebounced;
use crate::arb::global::trace::types::{StepType, Trace, WithTrace};
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
static POOL_UPDATE_CONSUMER: Lazy<Arc<PubSubProcessor<WithTrace<PoolUpdate>>>> = lazy_arc!({
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
static POOL_UPDATE_DEBOUNCER: Lazy<Arc<BufferedDebouncer<Pubkey, WithTrace<GrpcAccountUpdate>>>> =
    lazy_arc!({
        BufferedDebouncer::new(
            Duration::from_millis(30),
            |update: WithTrace<GrpcAccountUpdate>| async move {
                update.step_with_address(
                    AccountUpdateDebounced,
                    "account_address",
                    update.param.account,
                );
                let updated = AccountState::from_grpc_update(&update.param);
                let previous = PoolAccountCache.put(update.param.account, updated.clone());
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
                    let mut trace = Trace::new();
                    trace.step_with_address(
                        StepType::AccountUpdateReceived,
                        "account_address",
                        account_update.account,
                    );
                    async move { Self::handle_account_update(account_update, trace).await }
                },
                true,
            )
            .await
    }

    async fn handle_account_update(update: GrpcAccountUpdate, mut trace: Trace) -> Result<()> {
        debug!("Pool account update received: {}", update.account);
        trace.step_with_address(
            StepType::AccountUpdateDebouncing,
            "account_address",
            update.account,
        );
        POOL_UPDATE_DEBOUNCER.update(update.account, WithTrace::new(trace, update));

        empty_ok!()
    }
}

pub async fn start_pool_monitor() -> Result<()> {
    let monitor = PoolAccountMonitor::new().await?;
    monitor.start().await
}
