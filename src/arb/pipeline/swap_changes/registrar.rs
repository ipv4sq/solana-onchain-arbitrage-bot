use crate::arb::pipeline::swap_changes::account_monitor::subscriber::start_vault_monitor;
use crate::empty_ok;
use anyhow::Result;
use tracing::error;

pub async fn bootstrap_swap_changes_monitors() -> Result<()> {
    tokio::spawn(async move {
        if let Err(e) = start_vault_monitor().await {
            error!("Vault monitor failed: {}", e);
        }
    });
    empty_ok!()
}
