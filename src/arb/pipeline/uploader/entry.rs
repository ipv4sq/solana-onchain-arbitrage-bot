use crate::arb::convention::chain::util::simulation::SimulationResult;
use crate::arb::global::constant::mint::Mints;
use crate::arb::pipeline::swap_changes::cache::PoolConfigCache;
use crate::arb::pipeline::uploader::mev_bot::construct::build_and_send;
use crate::arb::pipeline::uploader::wallet::get_wallet;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{empty_ok, lazy_arc};
use futures::future::join_all;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::io::empty;
use std::sync::Arc;

pub struct MevBotFire {
    pub minor_mint: MintAddress,
    pub pools: Vec<PoolAddress>,
}

#[allow(non_upper_case_globals)]
pub static FireMevBotConsumer: Lazy<Arc<PubSubProcessor<MevBotFire>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 4,
        channel_buffer_size: 500,
        name: "VaultUpdateProcessor".to_string(),
    };

    PubSubProcessor::new(config, |event: MevBotFire| {
        Box::pin(async move { fire_mev_bot(&event.minor_mint, &event.pools).await })
    })
});

async fn fire_mev_bot(minor_mint: &Pubkey, pools: &Vec<Pubkey>) -> AResult<()> {
    let wallet = get_wallet();
    let configs: Vec<_> = join_all(
        pools
            .iter()
            .map(|pool_address| async move { PoolConfigCache.get(pool_address).await }),
    )
    .await
    .into_iter()
    .flatten()
    .collect();

    let wallet_pubkey = wallet.pubkey();
    build_and_send(&wallet, minor_mint, 700_000, 1_000, &configs, 0, true)
        .await
        .map(|result| log(result, &wallet_pubkey))?;
    empty_ok!()
}

pub fn log(result: SimulationResult, wallet_address: &Pubkey) {
    if let Some(err) = result.err {
        tracing::error!("TX aborted: {}", err);
        return;
    }

    let Some(meta) = result.meta else {
        tracing::info!("TX simulation completed (no metadata)");
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
        tracing::info!("Profitable TX: +{} WSOL lamports", wsol_change);
    } else if wsol_change < 0 {
        tracing::warn!("Unprofitable TX: {} WSOL lamports", wsol_change);
    } else {
        tracing::info!("Break-even TX: 0 WSOL change");
    }
}
