#![allow(non_upper_case_globals)]

use crate::arb::convention::chain::AccountState;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::pool_do::Model as PoolDo;
use crate::arb::util::alias::MintAddress;
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::arb::util::structs::loading_cache::LoadingCache;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;

#[allow(unused)]
pub static VaultAccountCache: LazyCache<Pubkey, AccountState> = LazyCache::new();
pub static MintWithPools: LazyCache<MintAddress, Vec<PoolDo>> = LazyCache::new();
pub static POOL_CONFIG_CACHE: Lazy<LoadingCache<Pubkey, AnyPoolConfig>> = Lazy::new(|| {
    LoadingCache::new(3000, |pool: &Pubkey| {
        let pool = *pool;
        async move { AnyPoolConfig::from(&pool).await.ok() }
    })
});
