use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::global::client::rpc::rpc_client;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::global::state::token_balance_holder::QueryRateLimiter;
use crate::arb::util::alias::{AResult, PoolAddress};
use crate::arb::util::structs::loading_cache::LoadingCache;
use crate::arb::util::structs::rate_limiter::RateLimitError;
use crate::arb::util::traits::option::OptionExt;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use tracing::warn;

pub struct AnyPoolHolder;

impl AnyPoolHolder {
    pub async fn get(addr: &Pubkey) -> Option<AnyPoolConfig> {
        cache.get(addr).await
    }

    pub async fn fresh_get(addr: &Pubkey) -> AResult<AnyPoolConfig> {
        let config = AnyPoolConfig::from(addr).await?;
        cache.put(addr.clone(), config.clone()).await;
        Ok(config)
    }

    pub async fn upsert(config: AnyPoolConfig) {
        cache.put(config.pool(), config).await
    }

    pub async fn update_config(
        pool_address: &PoolAddress,
        owner: &Pubkey,
        data: &[u8],
    ) -> AResult<AnyPoolConfig> {
        let updated_config = AnyPoolConfig::from_owner_and_data(pool_address, owner, data)?;
        cache.put(updated_config.pool(), updated_config).await;
        Ok(cache.get(pool_address).await.or_err("")?)
    }
}

#[allow(non_upper_case_globals)]
static cache: Lazy<LoadingCache<Pubkey, AnyPoolConfig>> = Lazy::new(|| {
    LoadingCache::new(200_000_000, |pool: &Pubkey| {
        let pool = *pool;
        async move { AnyPoolConfig::from(&pool).await.ok() }
    })
});

impl AnyPoolConfig {
    fn from_owner_and_data(
        pool_address: &PoolAddress,
        owner: &Pubkey,
        data: &[u8],
    ) -> AResult<AnyPoolConfig> {
        let dex_type = DexType::determine_from(owner);
        Self::new(*pool_address, dex_type, data)
    }

    async fn from(pool_address: &Pubkey) -> AResult<AnyPoolConfig> {
        QueryRateLimiter.try_acquire_err()?;
        let account = rpc_client().get_account(pool_address).await?;
        let dex_type = DexType::determine_from(&account.owner);
        Self::new(*pool_address, dex_type, &account.data)
    }
}
