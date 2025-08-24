use crate::arb::convention::chain::AccountState;
use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use anyhow::Result;
use tracing::info;

pub async fn process_balance_change(
    current: &AccountState,
    _previous: &AccountState,
    change: i64,
) -> Result<()> {
    if change > 0 {
        info!(
            "Vault {} received {} lamports at slot {}",
            current.pubkey, change, current.slot
        );
    } else {
        info!(
            "Vault {} sent {} lamports at slot {}",
            current.pubkey, -change, current.slot
        );
    }

    Ok(())
}

pub async fn process_vault_update(update: VaultUpdate) -> Result<()> {
    let lamport_change = update.lamport_change();
    let slot_delta = update.slot_delta();

    if lamport_change != 0 {
        info!(
            "Processing vault {} update: {} lamports change over {} slots",
            update.vault(),
            lamport_change,
            slot_delta
        );
    }

    if update.data_changed() {
        info!("Vault {} data changed", update.vault());
    }

    if update.owner_changed() {
        info!("Vault {} owner changed", update.vault());
    }

    Ok(())
}
