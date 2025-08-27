use crate::arb::global::trace::types::Trace;
use crate::arb::pipeline::swap_changes::account_monitor::pool_update::PoolUpdate;
use crate::arb::pipeline::trade_strategy::entry::on_pool_update;
use anyhow::Result;
use tracing::{debug, info};

pub async fn process_pool_update(update: PoolUpdate, trace: Trace) -> Result<()> {
    debug!("Processing pool update for: {}", update.pool());

    if update.is_initial() {
        debug!("Skipping initial pool update");
        return Ok(());
    }

    if update.data_changed() {
        info!("Pool data changed for: {}", update.pool());
        on_pool_update(update, trace).await;
    } else {
        debug!("Pool data unchanged, skipping update");
    }

    Ok(())
}
