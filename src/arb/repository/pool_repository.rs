use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sea_orm::*;
use sea_orm::ActiveValue::Set;
use crate::arb::repository::entity::{pool_mints, prelude::*};

pub struct PoolRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PoolRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(
        &self,
        pool_id: String,
        desired_mint: String,
        the_other_mint: String,
        dex_type: String,
    ) -> Result<pool_mints::Model> {
        // Check if exists
        let existing = PoolMints::find()
            .filter(pool_mints::Column::PoolId.eq(pool_id.clone()))
            .one(self.db)
            .await?;

        if let Some(model) = existing {
            // Update
            let mut active: pool_mints::ActiveModel = model.into();
            active.desired_mint = Set(desired_mint);
            active.the_other_mint = Set(the_other_mint);
            active.dex_type = Set(dex_type);
            active.updated_at = Set(Some(Utc::now()));
            Ok(active.update(self.db).await?)
        } else {
            // Insert
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

    pub async fn find_by_id(&self, id: i32) -> Result<Option<pool_mints::Model>> {
        Ok(PoolMints::find_by_id(id).one(self.db).await?)
    }

    pub async fn find_by_pool_id(&self, pool_id: &str) -> Result<Option<pool_mints::Model>> {
        Ok(PoolMints::find()
            .filter(pool_mints::Column::PoolId.eq(pool_id))
            .one(self.db)
            .await?)
    }

    pub async fn find_by_mints(
        &self,
        mint1: &str,
        mint2: &str,
    ) -> Result<Vec<pool_mints::Model>> {
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

    pub async fn find_by_dex_types(&self, dex_types: Vec<String>) -> Result<Vec<pool_mints::Model>> {
        Ok(PoolMints::find()
            .filter(pool_mints::Column::DexType.is_in(dex_types))
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(self.db)
            .await?)
    }

    pub async fn find_recent(
        &self,
        hours: i64,
    ) -> Result<Vec<pool_mints::Model>> {
        let since = Utc::now() - chrono::Duration::hours(hours);
        
        Ok(PoolMints::find()
            .filter(pool_mints::Column::CreatedAt.gte(since))
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(self.db)
            .await?)
    }

    pub async fn count_by_dex_type(&self) -> Result<Vec<(String, i64)>> {
        // Using SeaORM's query builder for aggregation
        #[derive(FromQueryResult)]
        struct DexTypeCount {
            dex_type: String,
            count: i64,
        }

        let results = pool_mints::Entity::find()
            .select_only()
            .column(pool_mints::Column::DexType)
            .column_as(pool_mints::Column::Id.count(), "count")
            .group_by(pool_mints::Column::DexType)
            .into_model::<DexTypeCount>()
            .all(self.db)
            .await?;

        Ok(results.into_iter().map(|r| (r.dex_type, r.count)).collect())
    }

    pub async fn delete_old(&self, days: i64) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        
        let result = PoolMints::delete_many()
            .filter(pool_mints::Column::UpdatedAt.lt(cutoff))
            .exec(self.db)
            .await?;

        Ok(result.rows_affected)
    }

    pub async fn paginate(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<pool_mints::Model>, u64)> {
        let paginator = PoolMints::find()
            .order_by_desc(pool_mints::Column::CreatedAt)
            .paginate(self.db, per_page);

        let total_pages = paginator.num_pages().await?;
        let items = paginator.fetch_page(page - 1).await?;

        Ok((items, total_pages))
    }

    pub async fn search(
        &self,
        search_term: &str,
    ) -> Result<Vec<pool_mints::Model>> {
        Ok(PoolMints::find()
            .filter(
                Condition::any()
                    .add(pool_mints::Column::PoolId.contains(search_term))
                    .add(pool_mints::Column::DesiredMint.contains(search_term))
                    .add(pool_mints::Column::TheOtherMint.contains(search_term))
                    .add(pool_mints::Column::DexType.contains(search_term))
            )
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(self.db)
            .await?)
    }

    pub async fn batch_insert(&self, pools: Vec<(String, String, String, String)>) -> Result<()> {
        let models: Vec<pool_mints::ActiveModel> = pools
            .into_iter()
            .map(|(pool_id, desired_mint, the_other_mint, dex_type)| {
                pool_mints::ActiveModel {
                    pool_id: Set(pool_id),
                    desired_mint: Set(desired_mint),
                    the_other_mint: Set(the_other_mint),
                    dex_type: Set(dex_type),
                    created_at: Set(Some(Utc::now())),
                    updated_at: Set(Some(Utc::now())),
                    ..Default::default()
                }
            })
            .collect();

        PoolMints::insert_many(models)
            .exec(self.db)
            .await?;

        Ok(())
    }
}