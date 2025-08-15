use crate::arb::constant::mint::{MintPair, WSOL_KEY, USDC_KEY};
use crate::arb::db::Database;
use crate::arb::tx::constants::DexType;
use crate::arb::tx::tx_parser::{convert_to_smb_ix, filter_swap_inner_ix, parse_swap_inner_ix};
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiInnerInstructions, UiPartiallyDecodedInstruction,
};
use std::sync::Arc;
use tokio::sync::OnceCell;

static DATABASE: OnceCell<Arc<Database>> = OnceCell::const_new();

async fn get_database() -> Result<Arc<Database>> {
    DATABASE
        .get_or_init(|| async {
            Arc::new(Database::new().await.expect("Failed to initialize database"))
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

    let mapped = swap_instructions
        .values()
        .into_iter()
        .filter_map(|x| parse_swap_inner_ix(x, tx).ok())
        .collect::<Vec<_>>();

    let db = get_database().await?;
    
    for swap in mapped.iter() {
        if let Err(e) = record_pool_and_mints(
            db.clone(),
            &swap.pool_address,
            swap.dex_type,
            &swap.mints
        ).await {
            tracing::error!("Failed to record pool and mints: {}", e);
        }
    }

    Ok(())
}

pub(super) async fn record_pool_and_mints(
    db: Arc<Database>,
    pool: &Pubkey,
    dex_type: DexType,
    mints: &MintPair
) -> Result<()> {
    let dex_type_str = format!("{:?}", dex_type);
    
    // Determine which mint is the desired one (WSOL or USDC for arbitrage)
    let (desired_mint, the_other_mint) = if mints.0 == *WSOL_KEY || mints.0 == *USDC_KEY {
        (Some(&mints.0), Some(&mints.1))
    } else if mints.1 == *WSOL_KEY || mints.1 == *USDC_KEY {
        (Some(&mints.1), Some(&mints.0))
    } else {
        // If neither is WSOL or USDC, skip recording
        return Ok(());
    };
    
    // Only record if we have a desired mint
    if let (Some(desired), Some(other)) = (desired_mint, the_other_mint) {
        db.record_pool_and_mints(pool, desired, other, &dex_type_str).await
    } else {
        Ok(())
    }
}
