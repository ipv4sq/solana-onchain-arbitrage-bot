use crate::arb::pipeline::chain_subscriber::involved_account_subscriber::start_involved_account_monitor;
use crate::arb::pipeline::chain_subscriber::owner_account_subscriber::start_owner_account_monitor;
use crate::unit_ok;
use anyhow::Result;
use tracing::{error, info};

pub async fn bootstrap_subscriber() -> Result<()> {
    // tokio::spawn(async move {
    //     let _ = start_mev_bot_subscriber().await;
    // });

    info!("🚀 Starting pool account monitor");
    tokio::spawn(async move {
        if let Err(e) = start_owner_account_monitor().await {
            error!("Pool monitor failed: {}", e);
        }
    });
    info!("🚀 Starting involved account monitor");
    tokio::spawn(async move {
        if let Err(e) = start_involved_account_monitor().await {
            error!("Pool monitor failed: {}", e);
        }
    });
    unit_ok!()
}
