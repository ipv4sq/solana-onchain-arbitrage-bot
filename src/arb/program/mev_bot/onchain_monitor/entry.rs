use crate::arb::convention::chain::types::LitePool;
use crate::arb::convention::chain::Transaction;
use crate::arb::repository::get_repository_manager;
use crate::arb::global::state::mem_pool::mem_pool;
use crate::arb::program::mev_bot::ix;
use crate::constants::addresses::TokenMint;
use anyhow::Result;
use tracing::info;

pub async fn entry(tx: &Transaction) -> Result<()> {
    let Some((_ix, inner)) = ix::extract_mev_instruction(tx) else {
        return Ok(());
    };

    // Check if transaction has metadata
    if tx.meta.is_none() {
        info!("Transaction {} has no metadata - cannot extract token balance changes", tx.signature);
    } else if let Some(ref meta) = tx.meta {
        info!("Transaction {} has {} pre and {} post token balances", 
            tx.signature,
            meta.pre_token_balances.len(),
            meta.post_token_balances.len()
        );
    }
    
    // Log token balance changes for WSOL and USDC
    let balance_changes = tx.token_balance_changes();
    
    // Debug: log all mints that have balance changes with full details
    if !balance_changes.is_empty() {
        info!("Transaction {} has balance changes for {} mints", 
            tx.signature, 
            balance_changes.len()
        );
        
        // Log all balance changes to see what we're actually getting
        for (mint, owner_changes) in &balance_changes {
            info!("  Mint: {} (length: {})", mint, mint.len());
            for (owner, change) in owner_changes {
                if change.change != 0 {
                    let divisor = 10_f64.powi(change.decimals as i32);
                    info!(
                        "    Owner: {}, Change: {} (from {} to {}, decimals: {})",
                        owner,
                        change.change as f64 / divisor,
                        change.pre_balance as f64 / divisor,
                        change.post_balance as f64 / divisor,
                        change.decimals
                    );
                }
            }
        }
    } else {
        info!("Transaction {} has no token balance changes", tx.signature);
    }
    
    // Also check if any mint contains SOL or starts with expected patterns
    for mint in balance_changes.keys() {
        if mint.contains("111111111111111111111111111111111") || mint.starts_with("So1") {
            info!("Found potential SOL mint: {}", mint);
            if let Some(changes) = balance_changes.get(mint) {
                log_token_changes_for_mint(&changes, mint, "SOL/WSOL");
            }
        }
    }
    
    log_token_changes(&balance_changes, TokenMint::SOL, "WSOL");
    log_token_changes(&balance_changes, TokenMint::USDC, "USDC");

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

fn log_token_changes(
    balance_changes: &std::collections::HashMap<String, std::collections::HashMap<String, crate::arb::convention::chain::util::transaction::TokenBalanceChange>>,
    token_mint: &str,
    token_name: &str,
) {
    if let Some(token_changes) = balance_changes.get(token_mint) {
        log_token_changes_for_mint(token_changes, token_mint, token_name);
    }
}

fn log_token_changes_for_mint(
    token_changes: &std::collections::HashMap<String, crate::arb::convention::chain::util::transaction::TokenBalanceChange>,
    token_mint: &str,
    token_name: &str,
) {
    for (owner, change) in token_changes {
        if change.change != 0 {
            let divisor = 10_f64.powi(change.decimals as i32);
            info!(
                "{} balance change - Mint: {}, Owner: {}, Change: {} (from {} to {})",
                token_name,
                token_mint,
                owner,
                change.change as f64 / divisor,
                change.pre_balance as f64 / divisor,
                change.post_balance as f64 / divisor
            );
        }
    }
}

pub(crate) async fn record_pool_and_mints(lite_pool: &LitePool) -> Result<()> {
    let manager = get_repository_manager().await?;
    let dex_type_str = format!("{:?}", lite_pool.dex_type);
    let desired_mint = lite_pool.mints.desired_mint()?;
    let the_other_mint = lite_pool.mints.minor_mint()?;

    // Only record if we have a desired mint
    info!(
        "Recording pool {} with desired mint {} and other mint {} for {}",
        lite_pool.pool_address, desired_mint, the_other_mint, dex_type_str
    );
    manager.pools().record_pool_and_mints(
        &lite_pool.pool_address,
        &desired_mint,
        &the_other_mint,
        lite_pool.dex_type,
    )
    .await?;
    Ok(())
}
