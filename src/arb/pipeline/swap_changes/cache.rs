#![allow(non_upper_case_globals)]

use crate::arb::convention::chain::AccountState;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::{MintRecord, PoolRecord};
use crate::arb::pipeline::pool_indexer::token_recorder::ensure_mint_record_exist;
use crate::arb::util::alias::MintAddress;
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::arb::util::structs::loading_cache::LoadingCache;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::cmp::min;

pub static VaultAccountCache: LazyCache<Pubkey, AccountState> = LazyCache::new();

pub static MintWithPools: LazyCache<MintAddress, Vec<PoolRecord>> = LazyCache::new();

pub static PoolConfigCache: Lazy<LoadingCache<Pubkey, AnyPoolConfig>> = Lazy::new(|| {
    LoadingCache::new(3000, |pool: &Pubkey| {
        let pool = *pool;
        async move { AnyPoolConfig::from(&pool).await.ok() }
    })
});

pub static MintCache: Lazy<LoadingCache<MintAddress, MintRecord>> = Lazy::new(|| {
    LoadingCache::new(10000, |mint: &MintAddress| {
        let mint = *mint;
        async move { ensure_mint_record_exist(&mint).await.ok() }
    })
});
