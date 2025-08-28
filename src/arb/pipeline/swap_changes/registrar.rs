use crate::arb::pipeline::swap_changes::account_monitor::subscriber::start_pool_monitor;
use crate::unit_ok;
use anyhow::Result;
use tracing::{error, info};

pub async fn bootstrap_swap_changes_monitors() -> Result<()> {
    info!("🚀 Starting pool account monitor");
    tokio::spawn(async move {
        if let Err(e) = start_pool_monitor().await {
            error!("Pool monitor failed: {}", e);
        }
    });
    unit_ok!()
}
