use crate::arb::constant::mint::{MintPair, USDC_KEY, WSOL_KEY};
use crate::arb::global::db::{get_database, Database};
use crate::arb::global::mem_pool::{mem_pool, MemPool};
use crate::arb::tx::tx_parser::{convert_to_smb_ix, filter_swap_inner_ix, parse_swap_inner_ix};
use crate::arb::tx::types::LitePool;
use anyhow::Result;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiInnerInstructions, UiPartiallyDecodedInstruction,
};
use tracing::{debug, info};

pub async fn on_mev_bot_transaction(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
    ix: &UiPartiallyDecodedInstruction,
    inner: &UiInnerInstructions,
) -> Result<()> {
    let _smb_ix = convert_to_smb_ix(ix)?;
    let swap_instructions = filter_swap_inner_ix(inner);

    info!(
        "Found {} swap instructions to parse",
        swap_instructions.len()
    );

    let mapped = swap_instructions
        .values()
        .into_iter()
        .filter_map(|x| match parse_swap_inner_ix(x, tx) {
            Ok(swap) => {
                debug!(
                    "Successfully parsed swap: {:?} on pool {}",
                    swap.dex_type, swap.pool_address
                );
                Some(swap)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to parse swap instruction. Program: {}, Error: {}",
                    x.program_id,
                    e
                );
                None
            }
        })
        .collect::<Vec<_>>();

    info!("Successfully parsed {} swaps", mapped.len());

    for swap in mapped.iter() {
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
