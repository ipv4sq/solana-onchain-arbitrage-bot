use crate::arb::convention::chain::AccountState;
use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::trace::types::StepType::AccountUpdateDebounced;
use crate::arb::global::trace::types::{StepType, Trace};
use crate::arb::pipeline::swap_changes::account_monitor::entry;
use crate::arb::pipeline::swap_changes::account_monitor::pool_update::PoolUpdate;
use crate::arb::sdk::yellowstone::{AccountFilter, GrpcAccountUpdate, SolanaGrpcClient};
use crate::arb::util::structs::buffered_debouncer::BufferedDebouncer;
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

#[allow(non_upper_case_globals)]
pub static PoolAccountCache: LazyCache<Pubkey, AccountState> = LazyCache::new();

pub static POOL_UPDATE_CONSUMER: Lazy<Arc<PubSubProcessor<(PoolUpdate, Trace)>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 8,
        channel_buffer_size: 5000,
        name: "PoolUpdateProcessor".to_string(),
    };

    PubSubProcessor::new(config, |(update, trace): (PoolUpdate, Trace)| {
        Box::pin(async move {
            entry::process_pool_update(update, trace).await?;
            Ok(())
        })
    })
});

pub static NEW_POOL_CONSUMER: Lazy<Arc<PubSubProcessor<(Pubkey, Trace)>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 32,
        channel_buffer_size: 100_000,
        name: "NewPoolProcesseor".to_string(),
    };

    PubSubProcessor::new(config, |(pool_address, trace): (Pubkey, Trace)| {
        Box::pin(async move {
            entry::on_new_pool_received(pool_address, trace).await?;
            Ok(())
        })
    })
});

#[allow(unused)]
static POOL_UPDATE_DEBOUNCER: Lazy<Arc<BufferedDebouncer<Pubkey, (GrpcAccountUpdate, Trace)>>> =
    lazy_arc!({
        BufferedDebouncer::new(
            Duration::from_millis(1),
            |(update, trace): (GrpcAccountUpdate, Trace)| async move {
                let updated = AccountState::from_grpc_update(&update);
                let previous = PoolAccountCache.put(update.account, updated.clone());
                let pool_update = PoolUpdate {
                    previous,
                    current: updated,
                };
                trace.step_with_address(AccountUpdateDebounced, "account_address", update.account);
                if PoolRecordRepository::is_pool_recorded(pool_update.pool()).await {
                    if let Err(e) = POOL_UPDATE_CONSUMER.publish((pool_update, trace)).await {
                        error!("Failed to publish pool update: {}", e);
                    }
                } else {
                    if let Err(e) = NEW_POOL_CONSUMER
                        .publish((*pool_update.pool(), trace))
                        .await
                    {
                        error!("Failed to publish new pool update: {}", e);
                    }
                }
            },
        )
    });

#[allow(unused)]
pub struct PoolAccountMonitor {
    client: SolanaGrpcClient,
}

impl PoolAccountMonitor {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
        })
    }

    pub async fn start(self) -> Result<()> {
        info!("Starting pool account subscription for Meteora DLMM and DAMM V2 programs");

        let filter = AccountFilter::new("meteora_pools").with_owners(&[
            PoolProgram::METEORA_DLMM,
            PoolProgram::METEORA_DAMM_V2,
            PoolProgram::PUMP_AMM,
        ]);

        self.client
            .subscribe_accounts(
                filter,
                move |account_update| {
                    let trace = Trace::new();
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

    async fn handle_account_update(update: GrpcAccountUpdate, trace: Trace) -> Result<()> {
        debug!("Pool account update received: {}", update.account);
        trace.step_with_address(
            StepType::AccountUpdateDebouncing,
            "account_address",
            update.account,
        );
        POOL_UPDATE_DEBOUNCER.update(update.account, (update, trace));

        unit_ok!()
    }
}

pub async fn start_pool_monitor() -> Result<()> {
    let monitor = PoolAccountMonitor::new().await?;
    monitor.start().await
}
