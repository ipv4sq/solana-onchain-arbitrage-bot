use crate::arb::convention::chain::AccountState;
use crate::arb::pipeline::swap_changes::account_monitor::entry;
use crate::arb::pipeline::swap_changes::account_monitor::pool_vault::list_all_vaults;
use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use crate::arb::pipeline::swap_changes::cache::VaultAccountCache;
use crate::arb::sdk::yellowstone::{AccountFilter, GrpcAccountUpdate, SolanaGrpcClient};
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{empty_ok, lazy_arc};
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{error, info};

#[allow(unused)]
static VAULT_UPDATE_CONSUMER: Lazy<Arc<PubSubProcessor<VaultUpdate>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 4,
        channel_buffer_size: 500,
        name: "VaultUpdateProcessor".to_string(),
    };

    PubSubProcessor::new(config, |update: VaultUpdate| {
        Box::pin(async move {
            entry::process_vault_update(update).await?;
            Ok(())
        })
    })
});

#[allow(unused)]
pub struct VaultAccountMonitor {
    client: SolanaGrpcClient,
    vaults: HashSet<Pubkey>,
}

impl VaultAccountMonitor {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
            vaults: list_all_vaults().await?,
        })
    }

    pub async fn start(self) -> Result<()> {
        info!(
            "Starting vault account subscription for {} vaults",
            self.vaults.len(),
        );

        let vault_vec: Vec<Pubkey> = self.vaults.into_iter().collect();
        let filter = AccountFilter::new("vault_monitor").with_accounts(&vault_vec);

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
        let updated = AccountState::from_grpc_update(&update);
        let previous = VaultAccountCache.insert(update.account, updated.clone());
        let vault_update = VaultUpdate {
            previous,
            current: updated,
        };
        if let Err(e) = VAULT_UPDATE_CONSUMER.publish(vault_update).await {
            error!("Failed to publish vault update: {}", e);
        }
        empty_ok!()
    }
}

pub async fn start_vault_monitor() -> Result<()> {
    let monitor = VaultAccountMonitor::new().await?;
    monitor.start().await
}
