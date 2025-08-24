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
        let vaults = list_all_vaults().await?;
        let vault_set = vaults.into_iter().collect::<HashSet<_>>();

        info!("Initialized vault monitor with {} vaults", vault_set.len());

        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
            vaults: vault_set,
        })
    }

    pub async fn start(self, auto_retry: bool) -> Result<()> {
        info!(
            "Starting vault account subscription for {} vaults (auto_retry: {})",
            self.vaults.len(),
            auto_retry
        );

        let vault_vec: Vec<Pubkey> = self.vaults.iter().cloned().collect();

        let filter = AccountFilter::new("vault_monitor").with_accounts(&vault_vec);

        let vaults_arc = Arc::new(self.vaults);

        self.client
            .subscribe_accounts(
                filter,
                move |account_update| {
                    let vaults = vaults_arc.clone();
                    async move { Self::handle_account_update(account_update, vaults).await }
                },
                auto_retry,
            )
            .await
    }

    async fn handle_account_update(
        update: GrpcAccountUpdate,
        vaults: Arc<HashSet<Pubkey>>,
    ) -> Result<()> {
        if !vaults.contains(&update.account) {
            warn!("Received update for non-vault account: {}", update.account);
            return Ok(());
        }

        info!(
            "Vault account update: {} at slot {} (lamports: {}, data_len: {})",
            update.account,
            update.slot,
            update.lamports,
            update.data.len()
        );

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
                    vault: update.account,
                    slot: update.slot,
                    lamports: update.lamports,
                    lamport_change,
                    data: update.data,
                    owner: update.owner,
                    timestamp: std::time::Instant::now(),
                };

                if let Err(e) = publish_vault_update(vault_update).await {
                    error!("Failed to publish vault update: {}", e);
                }
            }
        }

        Ok(())
    }

    pub fn get_cached_state(vault: &Pubkey) -> Option<AccountState> {
        VAULT_ACCOUNT_CACHE.get(vault)
    }

    pub fn get_all_cached_states() -> Vec<AccountState> {
        VAULT_ACCOUNT_CACHE.values()
    }
}

pub async fn start_vault_monitor() -> Result<()> {
    let monitor = VaultAccountMonitor::new().await?;
    monitor.start(true).await
}
