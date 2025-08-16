use crate::arb::constant::mint::{MintPair, USDC_KEY, WSOL_KEY};
use crate::arb::db::Database;
use crate::arb::subscriber::in_mem_pool::{mem_pool, MemPool};
use crate::arb::tx::constants::DexType;
use crate::arb::tx::tx_parser::{convert_to_smb_ix, filter_swap_inner_ix, parse_swap_inner_ix};
use crate::arb::tx::types::LitePool;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiInnerInstructions, UiPartiallyDecodedInstruction,
};
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{debug, error, info};

static DATABASE: OnceCell<Arc<Database>> = OnceCell::const_new();

async fn get_database() -> Result<Arc<Database>> {
    DATABASE
        .get_or_init(|| async {
            Arc::new(
                Database::new()
                    .await
                    .expect("Failed to initialize database"),
            )
        })
        .await
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to get database instance"))
}

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
        record_pool_and_mints(&swap.pool_address, swap.dex_type, &swap.mints).await?;
        mem_pool().add_if_not_exists(lite_pool)?;
    }

    Ok(())
}

pub(super) async fn record_pool_and_mints(
    pool: &Pubkey,
    dex_type: DexType,
    mints: &MintPair,
) -> Result<()> {
    let db = get_database().await?;
    let dex_type_str = format!("{:?}", dex_type);

    // Determine which mint is the desired one (WSOL or USDC for arbitrage)
    let (desired_mint, the_other_mint) = if mints.0 == *WSOL_KEY || mints.0 == *USDC_KEY {
        (Some(&mints.0), Some(&mints.1))
    } else if mints.1 == *WSOL_KEY || mints.1 == *USDC_KEY {
        (Some(&mints.1), Some(&mints.0))
    } else {
        // If neither is WSOL or USDC, skip recording
        debug!(
            "Skipping pool {} - no WSOL/USDC mint found in pair {:?}",
            pool, mints
        );
        return Ok(());
    };

    // Only record if we have a desired mint
    if let (Some(desired), Some(other)) = (desired_mint, the_other_mint) {
        info!(
            "Recording pool {} with desired mint {} and other mint {} for {}",
            pool, desired, other, dex_type_str
        );
        db.record_pool_and_mints(pool, desired, other, &dex_type_str)
            .await
    } else {
        Ok(())
    }
}
