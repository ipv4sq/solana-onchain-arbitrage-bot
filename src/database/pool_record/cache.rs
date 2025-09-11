#![allow(non_upper_case_globals)]

use crate::database::pool_record::converter::build_model;
use crate::database::pool_record::model::Model as PoolRecord;
use crate::database::pool_record::repository::PoolRecordRepository;
use crate::global::state::any_pool_holder::AnyPoolHolder;
use crate::util::alias::{MintAddress, PoolAddress};
use crate::util::cache::loading_cache::LoadingCache;
use crate::util::cache::persistent_cache::PersistentCache;
use crate::util::structs::cache_type::CacheType;
use once_cell::sync::Lazy;
use std::collections::HashSet;

pub static PoolsContainMintSecondary: Lazy<PersistentCache<MintAddress, HashSet<PoolRecord>>> =
    Lazy::new(|| {
        PersistentCache::new_with_custom_db(
            CacheType::Custom("mint_to_pools".to_string()),
            1_000_000,
            i64::MAX,
            |_mint: MintAddress| async move { None },
            Some(|mint: MintAddress| async move {
                PoolRecordRepository::find_by_any_mint(&mint)
                    .await
                    .ok()
                    .map(|pools| pools.into_iter().collect())
            }),
            Some(|_mint: MintAddress, _pools: HashSet<PoolRecord>, _ttl: i64| async move {}),
        )
    });

pub static PoolCachePrimary: Lazy<PersistentCache<PoolAddress, PoolRecord>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        CacheType::Custom("pool_cache".to_string()),
        1_000_000,
        i64::MAX,
        |addr: PoolAddress| {
            let addr = addr;
            async move {
                let config = AnyPoolHolder::get(&addr).await?;
                build_model(config).await.ok()
            }
        },
        Some(|addr: PoolAddress| async move {
            PoolRecordRepository::find_by_address(&addr)
                .await
                .ok()
                .flatten()
        }),
        Some(
            |_addr: PoolAddress, record: PoolRecord, _ttl: i64| async move {
                let _ = PoolRecordRepository::upsert_pool(record).await;
            },
        ),
    )
});

pub static PoolInDataBaseSecondary: Lazy<LoadingCache<PoolAddress, bool>> = Lazy::new(|| {
    LoadingCache::new(1_000_000, |addr: &PoolAddress| {
        let addr = *addr;
        async move {
            Some(
                PoolRecordRepository::find_by_address(&addr)
                    .await
                    .ok()
                    .flatten()
                    .is_some(),
            )
        }
    })
});
