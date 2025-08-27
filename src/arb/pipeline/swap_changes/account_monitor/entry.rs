use crate::arb::global::trace::types::WithTrace;
use crate::arb::pipeline::swap_changes::account_monitor::pool_update::PoolUpdate;
use crate::arb::pipeline::trade_strategy::entry::on_pool_update;
use anyhow::Result;
use tracing::{debug, info};

pub async fn process_pool_update(update: WithTrace<PoolUpdate>) -> Result<()> {
    debug!("Processing pool update for: {}", update.param.pool());

    if update.param.is_initial() {
        debug!("Skipping initial pool update");
        return Ok(());
    }

    if update.param.data_changed() {
        info!("Pool data changed for: {}", update.param.pool());
        on_pool_update(update).await;
    } else {
        debug!("Pool data unchanged, skipping update");
    }

    Ok(())
}
