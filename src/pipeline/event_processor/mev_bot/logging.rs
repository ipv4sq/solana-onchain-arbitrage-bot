use crate::convention::chain::util::token_balance::TokenBalanceChange;
use crate::convention::chain::Transaction;
use crate::global::constant::mint::Mints;
use tracing::info;

pub fn log_token_changes(
    balance_changes: &std::collections::HashMap<
        String,
        std::collections::HashMap<String, TokenBalanceChange>,
    >,
    token_mint: &str,
    token_name: &str,
) {
    if let Some(token_changes) = balance_changes.get(token_mint) {
        log_token_changes_for_mint(token_changes, token_mint, token_name);
    }
}

pub fn log_token_changes_for_mint(
    token_changes: &std::collections::HashMap<String, TokenBalanceChange>,
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

pub fn log_token_balances_of(tx: &Transaction) {
    // Check if transaction has metadata
    if tx.meta.is_none() {
        info!(
            "Transaction {} has no metadata - cannot extract token balance changes",
            tx.signature
        );
    } else if let Some(ref meta) = tx.meta {
        info!(
            "Transaction {} has {} pre and {} post token balances",
            tx.signature,
            meta.pre_token_balances.len(),
            meta.post_token_balances.len()
        );
    }

    // Log token balance changes for WSOL and USDC
    let balance_changes = tx.token_balance_changes();

    // Debug: log all mints that have balance changes with full details
    if !balance_changes.is_empty() {
        info!(
            "Transaction {} has balance changes for {} mints",
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
        if *mint == Mints::WSOL.to_string() || *mint == Mints::USDC.to_string() {
            info!("Found potential SOL mint: {}", mint);
            if let Some(changes) = balance_changes.get(mint) {
                log_token_changes_for_mint(&changes, mint, "SOL/WSOL");
            }
        }
    }

    log_token_changes(&balance_changes, &Mints::WSOL.to_string(), "WSOL");
    log_token_changes(&balance_changes, &Mints::USDC.to_string(), "USDC");
}
