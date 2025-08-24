use crate::arb::pipeline::swap_changes::account_monitor::pool_vault::{
    get_mint_and_pool_of_vault, MINT_WITH_POOLS,
};
use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;

pub async fn on_swap_occurred(update: VaultUpdate) -> Option<()> {
    let vault = update.current.pubkey;
    let (mint, pool) = get_mint_and_pool_of_vault(&vault)?;
    let pool_record = MINT_WITH_POOLS.get(&mint)?;
    None
}
