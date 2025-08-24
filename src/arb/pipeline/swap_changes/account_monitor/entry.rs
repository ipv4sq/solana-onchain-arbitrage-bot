use crate::arb::convention::chain::AccountState;
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
