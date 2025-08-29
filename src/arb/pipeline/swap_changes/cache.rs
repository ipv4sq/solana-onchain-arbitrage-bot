#![allow(non_upper_case_globals)]

use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::PoolRecord;
use crate::arb::util::alias::MintAddress;
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::arb::util::structs::loading_cache::LoadingCache;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;

pub static MintWithPools: LazyCache<MintAddress, Vec<PoolRecord>> = LazyCache::new();

pub static PoolConfigCache: Lazy<LoadingCache<Pubkey, AnyPoolConfig>> = Lazy::new(|| {
    LoadingCache::new(100_000_000, |pool: &Pubkey| {
        let pool = *pool;
        async move { AnyPoolConfig::from(&pool).await.ok() }
    })
});
