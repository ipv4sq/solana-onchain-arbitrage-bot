use crate::arb::convention::chain::AccountState;
use crate::arb::util::types::cache::LazyCache;
use crate::arb::util::worker::pubsub::{PubSubConfig, SingletonPubSub};
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct VaultUpdate {
    pub vault: Pubkey,
    pub slot: u64,
    pub lamports: u64,
    pub lamport_change: i64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub timestamp: std::time::Instant,
}

pub static VAULT_UPDATE_PROCESSOR: Lazy<Arc<SingletonPubSub<VaultUpdate>>> =
    Lazy::new(|| Arc::new(SingletonPubSub::new("VaultUpdateProcessor".to_string())));

pub async fn initialize_vault_processor() -> Result<()> {
    let config = PubSubConfig {
        worker_pool_size: 4,
        channel_buffer_size: 1000,
        name: "VaultUpdateProcessor".to_string(),
    };

    VAULT_UPDATE_PROCESSOR
        .initialize(config, |update| {
            Box::pin(async move {
                info!(
                    "Processing vault update: {} (slot: {}, change: {} lamports)",
                    update.vault, update.slot, update.lamport_change
                );

                process_vault_update(update)
            })
        })
        .await
}

static RECENT_VAULT_UPDATES: LazyCache<Pubkey, Vec<VaultUpdate>> = LazyCache::new();

fn process_vault_update(update: VaultUpdate) -> Result<()> {
    let mut updates = RECENT_VAULT_UPDATES.get(&update.vault).unwrap_or_default();

    updates.push(update.clone());

    const MAX_HISTORY: usize = 100;
    if updates.len() > MAX_HISTORY {
        updates.drain(0..updates.len() - MAX_HISTORY);
    }

    RECENT_VAULT_UPDATES.insert(update.vault, updates);

    if update.lamport_change.abs() > 1_000_000_000 {
        info!(
            "Large vault change detected: {} changed by {} SOL at slot {}",
            update.vault,
            update.lamport_change as f64 / 1_000_000_000.0,
            update.slot
        );
    }

    Ok(())
}

pub fn get_recent_updates(vault: &Pubkey) -> Vec<VaultUpdate> {
    RECENT_VAULT_UPDATES.get(vault).unwrap_or_default()
}

pub fn get_all_recent_updates() -> Vec<(Pubkey, Vec<VaultUpdate>)> {
    RECENT_VAULT_UPDATES.iter()
}

pub async fn publish_vault_update(update: VaultUpdate) -> Result<()> {
    VAULT_UPDATE_PROCESSOR.publish(update).await
}
