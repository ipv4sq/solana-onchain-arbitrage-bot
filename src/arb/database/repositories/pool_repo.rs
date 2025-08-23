use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::pool_do::{self, Entity as PoolRecord, Model};
use crate::arb::global::db::get_db;
use anyhow::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use solana_program::pubkey::Pubkey;

pub struct PoolRecordRepository;

impl PoolRecordRepository {
    pub async fn upsert_pool(pool: Model) -> Result<Model> {
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

        let result = PoolRecord::insert(active_model)
            .on_conflict(
                OnConflict::column(pool_do::Column::Address)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;

        match result {
            Ok(_) => Ok(pool),
            Err(_) => Self::find_by_address(&pool.address.0)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Failed to fetch existing pool")),
        }
    }

    pub async fn find_by_mints(mint1: &Pubkey, mint2: &Pubkey) -> Result<Vec<Model>> {
        let db = get_db();
        Ok(PoolRecord::find()
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

    pub async fn find_by_base_mint(base_mint: &Pubkey) -> Result<Vec<Model>> {
        let db = get_db();
        Ok(PoolRecord::find()
            .filter(pool_do::Column::BaseMint.eq(PubkeyType::from(*base_mint)))
            .all(db)
            .await?)
    }

    pub async fn find_by_quote_mint(quote_mint: &Pubkey) -> Result<Vec<Model>> {
        let db = get_db();
        Ok(PoolRecord::find()
            .filter(pool_do::Column::QuoteMint.eq(PubkeyType::from(*quote_mint)))
            .all(db)
            .await?)
    }

    pub async fn find_by_address(address: &Pubkey) -> Result<Option<Model>> {
        let db = get_db();
        Ok(PoolRecord::find_by_id(PubkeyType::from(*address))
            .one(db)
            .await?)
    }
}
