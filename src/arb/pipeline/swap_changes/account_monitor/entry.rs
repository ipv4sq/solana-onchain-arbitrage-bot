use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use anyhow::Result;

pub async fn process_vault_update(update: VaultUpdate) -> Result<()> {
    if update.is_initial() {
        return Ok(());
    }

    let lamport_change = update.lamport_change();
    Ok(())
}
