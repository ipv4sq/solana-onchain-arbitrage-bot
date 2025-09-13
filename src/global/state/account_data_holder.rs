use crate::global::constant::duration::Interval;
use crate::sdk::rpc::methods::account::buffered_get_account;
use crate::util::cache::loading_cache::LoadingCache;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use tracing::warn;

#[allow(non_upper_case_globals)]
static AccountDataCache: Lazy<LoadingCache<Pubkey, Vec<u8>>> =
    Lazy::new(|| LoadingCache::with_ttl(500_000, Interval::HOUR, |_| async move { None }));

pub struct AccountDataHolder {}

impl AccountDataHolder {
    pub async fn get_account_data(addr: &Pubkey) -> Option<Vec<u8>> {
        if let Some(data) = AccountDataCache.get_if_present(addr).await {
            return Some(data);
        }

        if let Some(account) = buffered_get_account(addr).await.ok() {
            let data = account.data;
            AccountDataCache.put(*addr, data.clone()).await;
            return Some(data);
        }

        None
    }

    pub async fn update(addr: Pubkey, data: Vec<u8>) {
        AccountDataCache.put(addr, data).await;
    }
}
