use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::util::structs::loading_cache::LoadingCache;
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

    pub async fn upsert(config: AnyPoolConfig) {
        PoolConfigCache.put(config.pool(), config).await
    }
}
