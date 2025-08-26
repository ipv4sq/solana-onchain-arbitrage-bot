use crate::arb::convention::chain::types::LitePool;
use crate::arb::convention::chain::Transaction;
use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
pub use crate::arb::global::constant::mint::Mints;
use crate::arb::global::state::mem_pool::mem_pool;
use crate::arb::pipeline::pool_indexer::mev_bot::logging::log_token_balances_of;
use crate::arb::pipeline::pool_indexer::token_recorder::ensure_mint_record_exist;
use crate::arb::program::mev_bot::ix;
use crate::empty_ok;
use anyhow::Result;
use tracing::info;

pub async fn entry(tx: &Transaction) -> Result<()> {
    let Some((_ix, inner)) = ix::extract_mev_instruction(tx) else {
        return Ok(());
    };

    log_token_balances_of(tx);

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
    tokio::try_join!(
        ensure_mint_record_exist(&lite_pool.mints.0),
        ensure_mint_record_exist(&lite_pool.mints.1)
    )?;
    todo!();
    empty_ok!()
}
