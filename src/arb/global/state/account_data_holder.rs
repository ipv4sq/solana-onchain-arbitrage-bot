use crate::arb::global::client::rpc::rpc_client;
use crate::arb::global::constant::duration::Interval;
use crate::arb::global::state::token_balance_holder::QueryRateLimiter;
use crate::arb::util::structs::ttl_loading_cache::TtlLoadingCache;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use tracing::warn;

#[allow(non_upper_case_globals)]
static AccountDataCache: Lazy<TtlLoadingCache<Pubkey, Vec<u8>>> =
    Lazy::new(|| TtlLoadingCache::new(500_000, Interval::HOUR, |_| async move { None }));

pub struct AccountDataHolder {}

impl AccountDataHolder {
    pub async fn get_account_data(addr: &Pubkey) -> Option<Vec<u8>> {
        if let Some(data) = AccountDataCache.get_sync(addr) {
            return Some(data);
        }

        if !QueryRateLimiter.try_acquire() {
            warn!("Rpc client query limited");
        }

        if let Some(data) = rpc_client().get_account_data(addr).await.ok() {
            AccountDataCache.put(*addr, data.clone()).await;
            return Some(data);
        }

        None
    }

    pub async fn update(addr: Pubkey, data: Vec<u8>) {
        AccountDataCache.put(addr, data).await;
    }
}
