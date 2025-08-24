use crate::arb::util::worker::pubsub::SingletonPubSub;
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;

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

pub async fn publish_vault_update(update: VaultUpdate) -> Result<()> {
    VAULT_UPDATE_PROCESSOR.publish(update).await
}
