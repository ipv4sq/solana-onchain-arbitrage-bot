use crate::arb::pipeline::swap_changes::account_monitor::entry;
use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static VAULT_UPDATE_CONSUMER: Lazy<Arc<PubSubProcessor<VaultUpdate>>> =
    Lazy::new(|| Arc::new(init()));

fn init() -> PubSubProcessor<VaultUpdate> {
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
}
