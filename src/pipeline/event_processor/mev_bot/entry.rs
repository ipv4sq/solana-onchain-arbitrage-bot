use crate::convention::chain::Transaction;
use crate::database::pool_record::repository::PoolRecordRepository;
use crate::pipeline::event_processor::mev_bot::logging::log_token_balances_of;
use crate::program::mev_bot::ix;
use crate::util::traits::option::OptionExt;
use anyhow::Result;
use tracing::info;

pub async fn entry(tx: &Transaction) -> Result<()> {
    let Some((_ix, inner)) = ix::extract_mev_instruction(tx) else {
        return Ok(());
    };

    log_token_balances_of(tx);

    let swaps = tx
        .extract_known_swap_ix(&inner.instructions)
        .or_err("missing swap instruction")?;

    for swap in swaps {
        info!(
            "Recording pool {} for {:?}",
            swap.pool_address, swap.dex_type
        );

        PoolRecordRepository::ensure_exists(&swap.pool_address).await;
    }
    Ok(())
}
