use crate::arb::convention::chain::AccountState;
use crate::arb::pipeline::swap_changes::account_monitor::consumer::{
    publish_vault_update, VaultUpdate,
};
use crate::arb::pipeline::swap_changes::account_monitor::pool_vault::list_all_vaults;
use crate::arb::pipeline::swap_changes::account_monitor::{consumer, entry};
use crate::arb::sdk::yellowstone::{AccountFilter, GrpcAccountUpdate, SolanaGrpcClient};
use crate::arb::util::types::cache::LazyCache;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{error, info, warn};

static VAULT_ACCOUNT_CACHE: LazyCache<Pubkey, AccountState> = LazyCache::new();

pub struct VaultAccountMonitor {
    client: SolanaGrpcClient,
    vaults: HashSet<Pubkey>,
}

impl VaultAccountMonitor {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
            vaults: list_all_vaults().await?.into_iter().collect(),
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
        let account_state = AccountState::from_grpc_update(&update);

        let previous = VAULT_ACCOUNT_CACHE.insert(update.account, account_state.clone());

        if let Some(prev_state) = previous {
            let lamport_change = account_state.calculate_lamport_change(&prev_state);
            if lamport_change != 0 {
                info!(
                    "Vault {} balance changed by {} lamports (slot {} -> {})",
                    update.account, lamport_change, prev_state.slot, account_state.slot
                );

                entry::process_balance_change(&account_state, &prev_state, lamport_change).await?;

                let vault_update = VaultUpdate {
                    previous: prev_state,
                    current: account_state.clone(),
                };

                if let Err(e) = publish_vault_update(vault_update).await {
                    error!("Failed to publish vault update: {}", e);
                }
            }
        }

        Ok(())
    }
}

pub async fn start_vault_monitor() -> Result<()> {
    let monitor = VaultAccountMonitor::new().await?;
    monitor.start().await
}
