#![allow(non_upper_case_globals)]

use crate::database::columns::PubkeyTypeString;
use crate::database::pool_record::converter::build_model;
use crate::database::pool_record::model;
use crate::database::pool_record::model::{Entity as PoolRecordEntity, Model as PoolRecord};
use crate::database::pool_record::repository::PoolRecordRepository;
use crate::global::client::db::get_db;
use crate::global::state::any_pool_holder::AnyPoolHolder;
use crate::util::alias::{MintAddress, PoolAddress};
use crate::util::cache::loading_cache::LoadingCache;
use crate::util::cache::persistent_cache::PersistentCache;
use crate::util::structs::cache_type::CacheType;
use once_cell::sync::Lazy;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;

pub static PoolsContainMintSecondary: Lazy<PersistentCache<MintAddress, HashSet<PoolRecord>>> =
    Lazy::new(|| {
        PersistentCache::new_with_custom_db(
            CacheType::Custom("mint_to_pools".to_string()),
            1_000_000,
            i64::MAX,
            |_mint: MintAddress| async move { None },
            Some(|mint_str: String| async move {
                if let Ok(mint) = mint_str.parse::<Pubkey>() {
                    PoolRecordRepository::find_by_any_mint(&mint)
                        .await
                        .ok()
                        .map(|pools| pools.into_iter().collect())
                } else {
                    None
                }
            }),
            Some(|_mint_str: String, _pools: HashSet<PoolRecord>, _ttl: i64| async move {}),
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
        Some(|addr_str: String| async move {
            if let Ok(addr) = addr_str.parse::<Pubkey>() {
                PoolRecordEntity::find()
                    .filter(model::Column::Address.eq(PubkeyTypeString::from(addr)))
                    .one(get_db().await)
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            }
        }),
        Some(
            |_addr_str: String, record: PoolRecord, _ttl: i64| async move {
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
                PoolRecordEntity::find()
                    .filter(model::Column::Address.eq(PubkeyTypeString::from(addr)))
                    .one(get_db().await)
                    .await
                    .ok()
                    .flatten()
                    .is_some(),
            )
        }
    })
});
