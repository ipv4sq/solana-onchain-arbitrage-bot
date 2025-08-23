use sea_orm::*;
use sea_orm::ActiveValue::Set;
use chrono::Utc;
use solana_sdk::pubkey::Pubkey;
use crate::arb::repository::core::error::RepositoryResult;
use crate::arb::repository::core::traits::WithConnection;
use crate::arb::repository::entity::PoolMints;
use super::super::entity::{pool_mints};
use crate::arb::global::enums::dex_type::DexType;

pub struct PoolRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PoolRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_pool_id(&self, pool_id: &str) -> RepositoryResult<Option<pool_mints::Model>> {
        Ok(PoolMints::find()
            .filter(pool_mints::Column::PoolId.eq(pool_id))
            .one(self.db)
            .await?)
    }

    pub async fn find_by_mints(
        &self,
        mint1: &str,
        mint2: &str,
    ) -> RepositoryResult<Vec<pool_mints::Model>> {
        Ok(PoolMints::find()
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(pool_mints::Column::DesiredMint.eq(mint1))
                            .add(pool_mints::Column::TheOtherMint.eq(mint2))
                    )
                    .add(
                        Condition::all()
                            .add(pool_mints::Column::DesiredMint.eq(mint2))
                            .add(pool_mints::Column::TheOtherMint.eq(mint1))
                    )
            )
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(self.db)
            .await?)
    }

    pub async fn find_by_dex_types(&self, dex_types: Vec<DexType>) -> RepositoryResult<Vec<pool_mints::Model>> {
        Ok(PoolMints::find()
            .filter(pool_mints::Column::DexType.is_in(dex_types))
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(self.db)
            .await?)
    }

    pub async fn record_pool_and_mints(
        &self,
        pool_id: &Pubkey,
        desired_mint: &Pubkey,
        the_other_mint: &Pubkey,
        dex_type: DexType,
    ) -> RepositoryResult<()> {
        self.upsert(
            pool_id.to_string(),
            desired_mint.to_string(),
            the_other_mint.to_string(),
            dex_type,
        )
        .await?;
        Ok(())
    }

    pub async fn find_pools_by_mints_pubkey(
        &self,
        desired_mint: &Pubkey,
        the_other_mint: &Pubkey,
    ) -> RepositoryResult<Vec<pool_mints::Model>> {
        self.find_by_mints(
            &desired_mint.to_string(),
            &the_other_mint.to_string(),
        )
        .await
    }

    pub async fn list_pool_mints(&self) -> RepositoryResult<Vec<pool_mints::Model>> {
        self.find_all().await
    }

    pub async fn list_pool_mints_by_dex(&self, dex_type: DexType) -> RepositoryResult<Vec<pool_mints::Model>> {
        self.find_by_dex_types(vec![dex_type]).await
    }

    pub async fn upsert(
        &self,
        pool_id: String,
        desired_mint: String,
        the_other_mint: String,
        dex_type: DexType,
    ) -> RepositoryResult<pool_mints::Model> {
        let existing = self.find_by_pool_id(&pool_id).await?;

        if let Some(model) = existing {
            let mut active: pool_mints::ActiveModel = model.into();
            active.desired_mint = Set(desired_mint);
            active.the_other_mint = Set(the_other_mint);
            active.dex_type = Set(dex_type);
            active.updated_at = Set(Some(Utc::now()));
            Ok(active.update(self.db).await?)
        } else {
            let new_pool = pool_mints::ActiveModel {
                pool_id: Set(pool_id),
                desired_mint: Set(desired_mint),
                the_other_mint: Set(the_other_mint),
                dex_type: Set(dex_type),
                created_at: Set(Some(Utc::now())),
                updated_at: Set(Some(Utc::now())),
                ..Default::default()
            };
            Ok(new_pool.insert(self.db).await?)
        }
    }
    
    // Repository methods
    
    pub async fn find_by_id(&self, id: i32) -> RepositoryResult<Option<pool_mints::Model>> {
        Ok(PoolMints::find_by_id(id).one(self.db).await?)
    }

    pub async fn find_all(&self) -> RepositoryResult<Vec<pool_mints::Model>> {
        Ok(PoolMints::find()
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(self.db)
            .await?)
    }

    pub async fn create(&self, model: pool_mints::ActiveModel) -> RepositoryResult<pool_mints::Model> {
        Ok(model.insert(self.db).await?)
    }

    pub async fn update(&self, model: pool_mints::ActiveModel) -> RepositoryResult<pool_mints::Model> {
        Ok(model.update(self.db).await?)
    }

    pub async fn delete(&self, id: i32) -> RepositoryResult<bool> {
        let result = PoolMints::delete_by_id(id).exec(self.db).await?;
        Ok(result.rows_affected > 0)
    }

    pub async fn count(&self) -> RepositoryResult<u64> {
        Ok(PoolMints::find().count(self.db).await?)
    }

    pub async fn paginate(
        &self,
        page: u64,
        per_page: u64,
    ) -> RepositoryResult<(Vec<pool_mints::Model>, u64)> {
        let paginator = PoolMints::find()
            .order_by_desc(pool_mints::Column::CreatedAt)
            .paginate(self.db, per_page);

        let total_pages = paginator.num_pages().await?;
        let items = paginator.fetch_page(page - 1).await?;

        Ok((items, total_pages))
    }

    pub async fn search(&self, query: &str) -> RepositoryResult<Vec<pool_mints::Model>> {
        Ok(PoolMints::find()
            .filter(
                Condition::any()
                    .add(pool_mints::Column::PoolId.contains(query))
                    .add(pool_mints::Column::DesiredMint.contains(query))
                    .add(pool_mints::Column::TheOtherMint.contains(query))
                    .add(pool_mints::Column::DexType.eq(query))
            )
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(self.db)
            .await?)
    }

    pub async fn batch_create(&self, models: Vec<pool_mints::ActiveModel>) -> RepositoryResult<()> {
        PoolMints::insert_many(models).exec(self.db).await?;
        Ok(())
    }

    pub async fn batch_update(&self, models: Vec<pool_mints::ActiveModel>) -> RepositoryResult<()> {
        for model in models {
            model.update(self.db).await?;
        }
        Ok(())
    }

    pub async fn batch_delete(&self, ids: Vec<i32>) -> RepositoryResult<u64> {
        let result = PoolMints::delete_many()
            .filter(pool_mints::Column::Id.is_in(ids))
            .exec(self.db)
            .await?;
        Ok(result.rows_affected)
    }
}

impl<'a> WithConnection for PoolRepository<'a> {
    fn connection(&self) -> &DatabaseConnection {
        self.db
    }
}