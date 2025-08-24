use crate::arb::convention::chain::AccountState;
use crate::arb::util::worker::pubsub::SingletonPubSub;
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct VaultUpdate {
    pub previous: AccountState,
    pub current: AccountState,
}

impl VaultUpdate {
    pub fn vault(&self) -> &Pubkey {
        &self.current.pubkey
    }
    
    pub fn lamport_change(&self) -> i64 {
        self.current.calculate_lamport_change(&self.previous)
    }
    
    pub fn data_changed(&self) -> bool {
        self.current.data_changed(&self.previous)
    }
    
    pub fn owner_changed(&self) -> bool {
        self.current.owner_changed(&self.previous)
    }
    
    pub fn slot_delta(&self) -> u64 {
        self.current.slot - self.previous.slot
    }
}

pub static VAULT_UPDATE_PROCESSOR: Lazy<Arc<SingletonPubSub<VaultUpdate>>> =
    Lazy::new(|| Arc::new(SingletonPubSub::new("VaultUpdateProcessor".to_string())));

pub async fn publish_vault_update(update: VaultUpdate) -> Result<()> {
    VAULT_UPDATE_PROCESSOR.publish(update).await
}
