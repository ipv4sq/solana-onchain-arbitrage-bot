#![allow(non_upper_case_globals)]

use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::pool_do::{
    self, Entity as PoolRecordEntity, Model as PoolRecord,
};
use crate::arb::global::db::get_db;
use crate::arb::pipeline::pool_indexer::pool_recorder::build_model;
use crate::arb::util::alias::{MintAddress, VaultAddress};
use crate::arb::util::structs::persistent_cache::PersistentCache;
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

pub struct PoolRecordRepository;

static VAULT_TO_POOL: Lazy<PersistentCache<VaultAddress, PoolRecord>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        100_000,
        Duration::MAX,
        |addr: &VaultAddress| {
            let addr = *addr;
            async move {
                let config = AnyPoolConfig::from(&addr).await.ok()?;
                let dex_type = config.dex_type();
                match config {
                    AnyPoolConfig::MeteoraDlmm(c) => {
                        build_model(&addr, &c.data, dex_type).await.ok()
                    }
                    AnyPoolConfig::MeteoraDammV2(c) => {
                        build_model(&addr, &c.data, dex_type).await.ok()
                    }
                    AnyPoolConfig::Unsupported => None,
                }
            }
        },
        |_addr: VaultAddress, record: PoolRecord, _d: Duration| async move {
            let _ = PoolRecordRepository::upsert_pool(record).await;
        },
        |addr: &VaultAddress| {
            let addr = *addr;
            async move {
                PoolRecordEntity::find()
                    .filter(
                        pool_do::Column::BaseVault
                            .eq(PubkeyType::from(addr))
                            .or(pool_do::Column::QuoteVault.eq(PubkeyType::from(addr))),
                    )
                    .one(get_db())
                    .await
                    .ok()
                    .flatten()
            }
        },
    )
});

static MINT_TO_POOLS: Lazy<PersistentCache<MintAddress, HashSet<PoolRecord>>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        100_000,
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

// cache related
impl PoolRecordRepository {
    pub async fn get_record_by_any_vault(vault: &VaultAddress) -> Option<PoolRecord> {
        VAULT_TO_POOL.get(vault).await
    }

    pub async fn ensure_exists(vault: &VaultAddress) {
        VAULT_TO_POOL.ensure_exists(vault).await
    }
}

impl PoolRecordRepository {
    async fn upsert_pool(pool: PoolRecord) -> Result<PoolRecord> {
        let db = get_db();
        let active_model = pool_do::ActiveModel {
            address: Set(pool.address.clone()),
            name: Set(pool.name.clone()),
            dex_type: Set(pool.dex_type.clone()),
            base_mint: Set(pool.base_mint.clone()),
            quote_mint: Set(pool.quote_mint.clone()),
            base_vault: Set(pool.base_vault.clone()),
            quote_vault: Set(pool.quote_vault.clone()),
            description: Set(pool.description.clone()),
            data_snapshot: Set(pool.data_snapshot.clone()),
            created_at: NotSet,
            updated_at: NotSet,
        };

        let result = PoolRecordEntity::insert(active_model)
            .on_conflict(
                OnConflict::column(pool_do::Column::Address)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;
        MINT_TO_POOLS.evict(&pool.address).await;
        MINT_TO_POOLS.ensure_exists(&pool.address).await;
        match result {
            Ok(_) => Ok(pool),
            Err(_) => Self::find_by_address(&pool.address.0)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Failed to fetch existing pool")),
        }
    }

    pub async fn find_by_mints(mint1: &Pubkey, mint2: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db();
        Ok(PoolRecordEntity::find()
            .filter(
                pool_do::Column::BaseMint
                    .eq(PubkeyType::from(*mint1))
                    .and(pool_do::Column::QuoteMint.eq(PubkeyType::from(*mint2)))
                    .or(pool_do::Column::BaseMint
                        .eq(PubkeyType::from(*mint2))
                        .and(pool_do::Column::QuoteMint.eq(PubkeyType::from(*mint1)))),
            )
            .all(db)
            .await?)
    }

    pub async fn find_by_base_mint(base_mint: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db();
        Ok(PoolRecordEntity::find()
            .filter(pool_do::Column::BaseMint.eq(PubkeyType::from(*base_mint)))
            .all(db)
            .await?)
    }

    pub async fn find_by_quote_mint(quote_mint: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db();
        Ok(PoolRecordEntity::find()
            .filter(pool_do::Column::QuoteMint.eq(PubkeyType::from(*quote_mint)))
            .all(db)
            .await?)
    }

    pub async fn find_by_any_mint(mint: &Pubkey) -> Result<Vec<PoolRecord>> {
        let db = get_db();
        Ok(PoolRecordEntity::find()
            .filter(
                pool_do::Column::BaseMint
                    .eq(PubkeyType::from(*mint))
                    .or(pool_do::Column::QuoteMint.eq(PubkeyType::from(*mint))),
            )
            .all(db)
            .await?)
    }

    pub async fn find_by_address(address: &Pubkey) -> Result<Option<PoolRecord>> {
        let db = get_db();
        Ok(PoolRecordEntity::find_by_id(PubkeyType::from(*address))
            .one(db)
            .await?)
    }
}
