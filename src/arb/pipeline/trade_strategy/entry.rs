use crate::arb::pipeline::swap_changes::account_monitor::pool_vault::get_mint_and_pool_of_vault;
use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use crate::arb::pipeline::swap_changes::cache::MintWithPools;
use crate::arb::pipeline::uploader::entry::{FireMevBotConsumer, MevBotFire};
use solana_program::pubkey::Pubkey;

pub async fn on_swap_occurred(update: VaultUpdate) -> Option<()> {
    let vault = update.current.pubkey;
    let (mint, pool) = get_mint_and_pool_of_vault(&vault)?;
    let pool_record: Vec<Pubkey> = MintWithPools
        .get(&mint)
        .iter()
        .flatten()
        .map(|x| x.address.0)
        .collect();

    let _ = FireMevBotConsumer
        .publish(MevBotFire {
            minor_mint: mint,
            pools: pool_record,
        })
        .await;
    None
}

pub async fn compute() {}
