use crate::arb::chain::instruction::{InnerInstructions, Instruction};
use crate::arb::chain::types::LitePool;
use crate::arb::chain::Transaction;
use crate::arb::global::db::get_database;
use crate::arb::global::mem_pool::mem_pool;
use crate::arb::program::solana_mev_bot::ix;
use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use tracing::info;

pub async fn entry(tx: &Transaction) -> Result<()> {
    let Some((ix, inner)) = ix::extract_mev_instruction(tx) else {
        return Ok(());
    };

    let swaps = tx.extract_known_swap_inner_ix(inner);
    for swap in swaps {
        info!(
            "Recording pool {} with mints {:?} for {:?}",
            swap.pool_address, swap.mints, swap.dex_type
        );
        let lite_pool = LitePool {
            dex_type: swap.dex_type.clone(),
            pool_address: swap.pool_address.clone(),
            mints: swap.mints.clone(),
        };
        record_pool_and_mints(&lite_pool).await?;
        mem_pool().add_if_not_exists(lite_pool)?;
    }
    Ok(())
}

pub(crate) async fn record_pool_and_mints(lite_pool: &LitePool) -> Result<()> {
    let db = get_database().await?;
    let dex_type_str = format!("{:?}", lite_pool.dex_type);
    let desired_mint = lite_pool.mints.desired_mint()?;
    let the_other_mint = lite_pool.mints.the_other_mint()?;

    // Only record if we have a desired mint
    info!(
        "Recording pool {} with desired mint {} and other mint {} for {}",
        lite_pool.pool_address, desired_mint, the_other_mint, dex_type_str
    );
    db.record_pool_and_mints(
        &lite_pool.pool_address,
        &desired_mint,
        &the_other_mint,
        &dex_type_str,
    )
    .await
}
