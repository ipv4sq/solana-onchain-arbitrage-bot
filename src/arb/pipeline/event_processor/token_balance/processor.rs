use crate::arb::convention::chain::meta::UiTokenAmount;
use crate::arb::convention::chain::Transaction;
use crate::arb::global::trace::types::WithTrace;
use crate::arb::util::alias::{AResult, MintAddress};
use crate::arb::util::structs::ttl_loading_cache::TtlLoadingCache;
use crate::arb::util::traits::option::OptionExt;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{f, lazy_arc, unit_ok};
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;

#[allow(non_upper_case_globals)]
pub static TokenBalanceProcessor: Lazy<Arc<PubSubProcessor<WithTrace<Transaction>>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 8,
        channel_buffer_size: 1_000_000,
        name: "TokenBalanceProcessor".to_string(),
    };

    PubSubProcessor::new(config, process_token_balance_change)
});

#[derive(Clone)]
pub struct TokenAmount {
    pub amount: u64,
    pub decimals: u8,
}
#[allow(non_upper_case_globals)]
pub static TokenBalanceShortLivingCache: Lazy<TtlLoadingCache<(Pubkey, MintAddress), TokenAmount>> =
    Lazy::new(|| TtlLoadingCache::new(2_000_000, Duration::from_secs(180), |_| async move { None }));

pub async fn process_token_balance_change(update: WithTrace<Transaction>) -> AResult<()> {
    let WithTrace(tx, trace) = update;
    trace.step_with_custom("Tracking Token balance change");
    let balances = tx
        .meta
        .map(|t| t.post_token_balances)
        .or_err(f!("Tx: {} Meta is empty, skipping", tx.signature))?;

    for t in balances.iter().filter(|x| x.owner.is_some()) {
        let owner = t.owner.as_ref().unwrap().to_pubkey();
        let mint = t.mint.to_pubkey();
        let amount = TokenAmount {
            amount: t.ui_token_amount.amount.parse::<u64>().unwrap_or(0),
            decimals: t.ui_token_amount.decimals,
        };
        TokenBalanceShortLivingCache
            .put((owner, mint), amount)
            .await;
    }

    unit_ok!()
}
