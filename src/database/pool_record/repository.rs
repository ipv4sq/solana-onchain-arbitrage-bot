#![allow(non_upper_case_globals)]

use crate::database::columns::PubkeyTypeString;
use crate::database::pool_record::converter::build_model;
use crate::database::pool_record::model::{
    self, Entity as PoolRecordEntity, Model as PoolRecord, Model,
};
use crate::global::client::db::get_db;
use crate::global::state::any_pool_holder::AnyPoolHolder;
use crate::util::alias::{MintAddress, PoolAddress};
use crate::util::structs::legacy_loading_cache::LoadingCache;
use crate::util::structs::persistent_cache::PersistentCache;
use anyhow::Result;
use once_cell::sync::Lazy;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use std::time::Duration;
use tracing::error;

pub struct PoolRecordRepository;

static MINT_TO_POOLS: Lazy<PersistentCache<MintAddress, HashSet<PoolRecord>>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        1_000_000,
        Duration::MAX,
        |_mint: &MintAddress| async move { None },
        |_mint: MintAddress, _pools: HashSet<PoolRecord>, _duration: Duration| async move {},
        |mint: &MintAddress| {
            let mint = *mint;
            async move {
                PoolRecordRepository::find_by_any_mint(&mint)
                    .await
                    .ok()
                    .map(|pools| pools.into_iter().collect())
            }
        },
    )
});

static POOL_CACHE: Lazy<PersistentCache<PoolAddress, PoolRecord>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        1_000_000,
        Duration::MAX,
        |addr| {
            let addr = *addr;
            async move {
                let config = AnyPoolHolder::get(&addr).await?;
                build_model(config).await.ok()
            }
        },
        |_mint, record, _duration| async move {
            let _ = PoolRecordRepository::upsert_pool(record).await;
        },
        |addr| {
            let addr = *addr;
            async move {
                PoolRecordEntity::find()
                    .filter(model::Column::Address.eq(PubkeyTypeString::from(addr)))
                    .one(get_db().await)
                    .await
                    .ok()
                    .flatten()
            }
        },
    )
});

static POOL_RECORDED: Lazy<LoadingCache<PoolAddress, bool>> = Lazy::new(|| {
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

// cache related
impl PoolRecordRepository {
    pub async fn get_pools_contains_mint(mint: &MintAddress) -> Option<HashSet<PoolRecord>> {
        MINT_TO_POOLS.get(mint).await
    }

    pub async fn get_pool_by_address(pool: &PoolAddress) -> Option<PoolRecord> {
        POOL_CACHE.get(pool).await
    }

    pub async fn ensure_exists(pool: &PoolAddress) -> Option<Model> {
        POOL_CACHE.ensure_exists(pool).await
    }

    pub async fn is_pool_recorded(pool: &PoolAddress) -> bool {
        POOL_RECORDED.get(pool).await.unwrap_or(false)
    }
}

impl PoolRecordRepository {
    async fn upsert_pool(pool: PoolRecord) -> Result<PoolRecord> {
        let db = get_db().await;
        let active_model = model::ActiveModel {
            address: Set(pool.address.clone()),
            name: Set(pool.name.clone()),
            dex_type: Set(pool.dex_type.clone()),
            base_mint: Set(pool.base_mint.clone()),
            quote_mint: Set(pool.quote_mint.clone()),
            description: Set(pool.description.clone()),
            data_snapshot: Set(pool.data_snapshot.clone()),
            created_at: NotSet,
            updated_at: NotSet,
        };

        let result = PoolRecordEntity::insert(active_model)
            .on_conflict(
                OnConflict::column(model::Column::Address)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;

        // update corresponding cache
        POOL_RECORDED.put(pool.address.0, true).await;
        // update mint to pool cache
        for mint in [pool.base_mint, pool.quote_mint] {
            if let Some(mut pools) = MINT_TO_POOLS.get(mint.as_ref()).await {
                pools.insert(pool.clone());
                MINT_TO_POOLS.put(mint.0, pools).await;
            }
        }

        match result {
            Ok(_) => Ok(pool),
            Err(e) => {
                error!("Failed to update pool record: {}", e);
                Self::find_by_address(&pool.address.0)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Failed to fetch existing pool"))
            }
        }
    }

    pub async fn find_by_mints(mint1: &Pubkey, mint2: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db().await;
        Ok(PoolRecordEntity::find()
            .filter(
                model::Column::BaseMint
                    .eq(PubkeyTypeString::from(*mint1))
                    .and(model::Column::QuoteMint.eq(PubkeyTypeString::from(*mint2)))
                    .or(model::Column::BaseMint
                        .eq(PubkeyTypeString::from(*mint2))
                        .and(model::Column::QuoteMint.eq(PubkeyTypeString::from(*mint1)))),
            )
            .all(db)
            .await?)
    }

    pub async fn find_by_base_mint(base_mint: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db().await;
        Ok(PoolRecordEntity::find()
            .filter(model::Column::BaseMint.eq(PubkeyTypeString::from(*base_mint)))
            .all(db)
            .await?)
    }

    pub async fn find_by_quote_mint(quote_mint: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db().await;
        Ok(PoolRecordEntity::find()
            .filter(model::Column::QuoteMint.eq(PubkeyTypeString::from(*quote_mint)))
            .all(db)
            .await?)
    }

    pub async fn find_by_any_mint(mint: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db().await;
        Ok(PoolRecordEntity::find()
            .filter(
                model::Column::BaseMint
                    .eq(PubkeyTypeString::from(*mint))
                    .or(model::Column::QuoteMint.eq(PubkeyTypeString::from(*mint))),
            )
            .all(db)
            .await?)
    }

    pub async fn find_by_address(address: &Pubkey) -> Result<Option<PoolRecord>> {
        let db = get_db().await;
        Ok(PoolRecordEntity::find_by_id(PubkeyTypeString::from(*address))
            .one(db)
            .await?)
    }
}
