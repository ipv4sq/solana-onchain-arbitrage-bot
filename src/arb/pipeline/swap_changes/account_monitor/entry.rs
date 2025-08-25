use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use crate::arb::pipeline::trade_strategy::entry::on_swap_occurred;
use anyhow::Result;

pub async fn process_vault_update(update: VaultUpdate) -> Result<()> {
    if update.is_initial() {
        return Ok(());
    }

    on_swap_occurred(update).await;

    Ok(())
}
