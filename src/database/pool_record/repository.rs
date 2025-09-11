use crate::database::columns::PubkeyTypeString;
use crate::database::pool_record::cache::{
    PoolCachePrimary, PoolInDataBaseSecondary, PoolsContainMintSecondary,
};
use crate::database::pool_record::model::{
    self, Entity as PoolRecordEntity, Model as PoolRecord, Model,
};
use crate::global::client::db::get_db;
use crate::util::alias::{MintAddress, PoolAddress};
use anyhow::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use tracing::error;

pub struct PoolRecordRepository;

// cache related
impl PoolRecordRepository {
    pub async fn get_pools_contains_mint(mint: &MintAddress) -> Option<HashSet<PoolRecord>> {
        PoolsContainMintSecondary.get(mint).await
    }

    pub async fn ensure_exists(pool: &PoolAddress) -> Option<Model> {
        PoolCachePrimary.get(pool).await
    }

    pub async fn is_pool_recorded(pool: &PoolAddress) -> bool {
        PoolInDataBaseSecondary.get(pool).await.unwrap_or(false)
    }
}

impl PoolRecordRepository {
    pub(crate) async fn upsert_pool(pool: PoolRecord) -> Result<PoolRecord> {
        let db = get_db().await;
        let active_model = model::ActiveModel {
            address: Set(pool.address.clone()),
            name: Set(pool.name.clone()),
            dex_type: Set(pool.dex_type.clone()),
            base_mint: Set(pool.base_mint.clone()),
            quote_mint: Set(pool.quote_mint.clone()),
            description: Set(pool.description.clone()),
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
        PoolInDataBaseSecondary.put(pool.address.0, true).await;
        // update mint to pool cache
        for mint in [pool.base_mint, pool.quote_mint] {
            if let Some(mut pools) = PoolsContainMintSecondary.get(mint.as_ref()).await {
                pools.insert(pool.clone());
                PoolsContainMintSecondary.put(mint.0, pools).await;
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
        Ok(
            PoolRecordEntity::find_by_id(PubkeyTypeString::from(*address))
                .one(db)
                .await?,
        )
    }
}
