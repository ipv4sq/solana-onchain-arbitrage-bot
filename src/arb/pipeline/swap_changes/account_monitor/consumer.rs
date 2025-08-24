use crate::arb::convention::chain::AccountState;
use crate::arb::pipeline::swap_changes::account_monitor::entry;
use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::ops::Deref;
use std::sync::Arc;
use tracing::info;

pub static VAULT_UPDATE_CONSUMER: Lazy<Arc<VaultUpdateConsumerPool>> =
    Lazy::new(|| Arc::new(VaultUpdateConsumerPool::new()));

impl VaultUpdateConsumerPool {
    fn new() -> Self {
        let config = PubSubConfig {
            worker_pool_size: 4,
            channel_buffer_size: 500,
            name: "VaultUpdateProcessor".to_string(),
        };

        let processor = PubSubProcessor::new(config, |update: VaultUpdate| {
            Box::pin(async move {
                entry::process_vault_update(update).await?;
                Ok(())
            })
        });

        info!("Vault update processor auto-initialized");

        Self(processor)
    }
}

pub struct VaultUpdateConsumerPool(PubSubProcessor<VaultUpdate>);

impl Deref for VaultUpdateConsumerPool {
    type Target = PubSubProcessor<VaultUpdate>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
