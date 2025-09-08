use crate::convention::chain::util::simulation::SimulationResult;
use crate::global::constant::mint::Mints;
use crate::global::trace::types::Trace;
use solana_program::pubkey::Pubkey;
use tracing::{info, warn};

pub fn print_log_to_console(result: SimulationResult, wallet_address: &Pubkey, trace: Trace) {
    info!("Finished simulation: {}", trace.dump_pretty());
    if let Some(err) = result.err {
        tracing::error!("TX aborted: {}", err);
        return;
    }

    let Some(meta) = result.meta else {
        info!("TX simulation completed (no metadata)");
        return;
    };

    let wsol_change = meta
        .post_token_balances
        .iter()
        .find(|tb| {
            tb.mint == Mints::WSOL.to_string()
                && tb.owner.as_ref() == Some(&wallet_address.to_string())
        })
        .and_then(|post| {
            meta.pre_token_balances
                .iter()
                .find(|tb| {
                    tb.mint == Mints::WSOL.to_string()
                        && tb.owner.as_ref() == Some(&wallet_address.to_string())
                })
                .map(|pre| {
                    let post_amount: i64 = post.ui_token_amount.amount.parse().unwrap_or(0);
                    let pre_amount: i64 = pre.ui_token_amount.amount.parse().unwrap_or(0);
                    post_amount - pre_amount
                })
        })
        .unwrap_or(0);

    if wsol_change > 0 {
        info!("Profitable TX: +{} WSOL lamports", wsol_change);
    } else if wsol_change < 0 {
        warn!("Unprofitable TX: {} WSOL lamports", wsol_change);
    } else {
        info!("Break-even TX: 0 WSOL change");
    }
}
