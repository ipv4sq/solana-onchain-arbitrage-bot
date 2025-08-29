use crate::arb::convention::chain::Transaction;
use crate::arb::database::pool_record::repository::PoolRecordRepository;
pub use crate::arb::global::constant::mint::Mints;
use crate::arb::pipeline::pool_indexer::mev_bot::logging::log_token_balances_of;
use crate::arb::program::mev_bot::ix;
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
            "Recording pool {} for {:?}",
            swap.pool_address, swap.dex_type
        );

        PoolRecordRepository::ensure_exists(&swap.pool_address).await;
    }
    Ok(())
}
