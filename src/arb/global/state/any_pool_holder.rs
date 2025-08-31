use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::util::structs::loading_cache::LoadingCache;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;

#[allow(non_upper_case_globals)]
pub static PoolConfigCache: Lazy<LoadingCache<Pubkey, AnyPoolConfig>> = Lazy::new(|| {
    LoadingCache::new(200_000_000, |pool: &Pubkey| {
        let pool = *pool;
        async move { AnyPoolConfig::from(&pool).await.ok() }
    })
});
