use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::util::alias::{AResult, PoolAddress};
use crate::arb::util::structs::cache_type::CacheType::PoolConfig;
use crate::arb::util::structs::loading_cache::LoadingCache;
use crate::arb::util::traits::option::OptionExt;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;

#[allow(non_upper_case_globals)]
static PoolConfigCache: Lazy<LoadingCache<Pubkey, AnyPoolConfig>> = Lazy::new(|| {
    LoadingCache::new(200_000_000, |pool: &Pubkey| {
        let pool = *pool;
        async move { AnyPoolConfig::from(&pool).await.ok() }
    })
});

pub struct AnyPoolHolder;

impl AnyPoolHolder {
    pub async fn get(addr: &Pubkey) -> Option<AnyPoolConfig> {
        PoolConfigCache.get(addr).await
    }

    pub async fn refresh(addr: &Pubkey) -> Option<AnyPoolConfig> {
        PoolConfigCache.invalidate(addr).await;
        PoolConfigCache.get(addr).await
    }

    pub async fn upsert(config: AnyPoolConfig) {
        PoolConfigCache.put(config.pool(), config).await
    }

    pub async fn update_config(
        pool_address: &PoolAddress,
        owner: &Pubkey,
        data: &[u8],
    ) -> AResult<AnyPoolConfig> {
        let updated_config = AnyPoolConfig::from_owner_and_data(pool_address, owner, data)?;
        PoolConfigCache
            .put(updated_config.pool(), updated_config)
            .await;
        Ok(PoolConfigCache.get(pool_address).await.or_err("")?)
    }
}
