#![allow(non_upper_case_globals)]

use crate::arb::convention::chain::AccountState;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::PoolRecord;
use crate::arb::util::alias::MintAddress;
use crate::arb::util::structs::cache_type::CacheType;
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::time::Duration;

pub static VaultAccountCache: LazyCache<Pubkey, AccountState> = LazyCache::new();

pub static MintWithPools: LazyCache<MintAddress, Vec<PoolRecord>> = LazyCache::new();

pub static PoolConfigCache: Lazy<PersistentCache<Pubkey, AnyPoolConfig>> = Lazy::new(|| {
    PersistentCache::new(
        CacheType::PoolConfig,
        3000,
        Duration::MAX, // 1 hour TTL
        |pool: &Pubkey| {
            let pool = *pool;
            async move { AnyPoolConfig::from(&pool).await.ok() }
        },
    )
});
