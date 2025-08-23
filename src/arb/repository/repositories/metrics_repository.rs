use sea_orm::*;
use sea_orm::ActiveValue::Set;
use chrono::Utc;
use rust_decimal::Decimal;
use crate::arb::repository::core::error::RepositoryResult;
use crate::arb::repository::core::traits::WithConnection;
use super::super::entity::{pool_metrics, PoolMetrics};

pub struct MetricsRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> MetricsRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn update_pool_metrics(
        &self,
        pool_id: String,
        dex_type: String,
        tvl_usd: Decimal,
        volume_24h_usd: Decimal,
        fee_24h_usd: Decimal,
        swap_count_24h: i64,
    ) -> RepositoryResult<pool_metrics::Model> {
        // Check if metrics exist
        let existing = PoolMetrics::find()
            .filter(pool_metrics::Column::PoolId.eq(pool_id.clone()))
            .one(self.db)
            .await?;

        let apy_24h = if tvl_usd > Decimal::ZERO {
            (fee_24h_usd / tvl_usd) * Decimal::from(365) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        if let Some(model) = existing {
            // Update existing
            let mut active: pool_metrics::ActiveModel = model.into();
            active.tvl_usd = Set(tvl_usd);
            active.volume_24h_usd = Set(volume_24h_usd);
            active.fee_24h_usd = Set(fee_24h_usd);
            active.apy_24h = Set(apy_24h);
            active.swap_count_24h = Set(swap_count_24h);
            active.last_swap_at = Set(Some(Utc::now()));
            active.updated_at = Set(Utc::now());
            Ok(active.update(self.db).await?)
        } else {
            // Insert new
            let new_metrics = pool_metrics::ActiveModel {
                pool_id: Set(pool_id),
                dex_type: Set(dex_type),
                tvl_usd: Set(tvl_usd),
                volume_24h_usd: Set(volume_24h_usd),
                volume_7d_usd: Set(Decimal::ZERO),
                fee_24h_usd: Set(fee_24h_usd),
                apy_24h: Set(apy_24h),
                price_impact_2_percent: Set(Decimal::ZERO),
                swap_count_24h: Set(swap_count_24h),
                unique_traders_24h: Set(0),
                last_swap_at: Set(Some(Utc::now())),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };
            Ok(new_metrics.insert(self.db).await?)
        }
    }

    pub async fn find_top_by_tvl(
        &self,
        limit: u64,
    ) -> RepositoryResult<Vec<pool_metrics::Model>> {
        Ok(PoolMetrics::find()
            .order_by_desc(pool_metrics::Column::TvlUsd)
            .limit(limit)
            .all(self.db)
            .await?)
    }

    pub async fn find_top_by_volume(
        &self,
        limit: u64,
    ) -> RepositoryResult<Vec<pool_metrics::Model>> {
        Ok(PoolMetrics::find()
            .order_by_desc(pool_metrics::Column::Volume24hUsd)
            .limit(limit)
            .all(self.db)
            .await?)
    }

    pub async fn find_top_by_apy(
        &self,
        limit: u64,
    ) -> RepositoryResult<Vec<pool_metrics::Model>> {
        Ok(PoolMetrics::find()
            .filter(pool_metrics::Column::Apy24h.gt(Decimal::ZERO))
            .order_by_desc(pool_metrics::Column::Apy24h)
            .limit(limit)
            .all(self.db)
            .await?)
    }

    pub async fn find_by_dex_type(
        &self,
        dex_type: &str,
    ) -> RepositoryResult<Vec<pool_metrics::Model>> {
        Ok(PoolMetrics::find()
            .filter(pool_metrics::Column::DexType.eq(dex_type))
            .order_by_desc(pool_metrics::Column::TvlUsd)
            .all(self.db)
            .await?)
    }

    pub async fn get_dex_summary(&self) -> RepositoryResult<Vec<DexSummary>> {
        #[derive(FromQueryResult)]
        struct Summary {
            dex_type: String,
            total_tvl: Option<Decimal>,
            total_volume: Option<Decimal>,
            pool_count: Option<i64>,
        }

        let summaries = PoolMetrics::find()
            .select_only()
            .column(pool_metrics::Column::DexType)
            .column_as(pool_metrics::Column::TvlUsd.sum(), "total_tvl")
            .column_as(pool_metrics::Column::Volume24hUsd.sum(), "total_volume")
            .column_as(pool_metrics::Column::Id.count(), "pool_count")
            .group_by(pool_metrics::Column::DexType)
            .into_model::<Summary>()
            .all(self.db)
            .await?;

        Ok(summaries.into_iter().map(|s| DexSummary {
            dex_type: s.dex_type,
            total_tvl: s.total_tvl.unwrap_or(Decimal::ZERO),
            total_volume_24h: s.total_volume.unwrap_or(Decimal::ZERO),
            pool_count: s.pool_count.unwrap_or(0),
        }).collect())
    }

    pub async fn find_low_liquidity_pools(
        &self,
        min_tvl: Decimal,
    ) -> RepositoryResult<Vec<pool_metrics::Model>> {
        Ok(PoolMetrics::find()
            .filter(pool_metrics::Column::TvlUsd.lt(min_tvl))
            .order_by_asc(pool_metrics::Column::TvlUsd)
            .all(self.db)
            .await?)
    }

    pub async fn cleanup_stale(&self, days: i64) -> RepositoryResult<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        
        let result = PoolMetrics::delete_many()
            .filter(pool_metrics::Column::UpdatedAt.lt(cutoff))
            .exec(self.db)
            .await?;

        Ok(result.rows_affected)
    }
}

impl<'a> WithConnection for MetricsRepository<'a> {
    fn connection(&self) -> &DatabaseConnection {
        self.db
    }
}

#[derive(Debug, Clone)]
pub struct DexSummary {
    pub dex_type: String,
    pub total_tvl: Decimal,
    pub total_volume_24h: Decimal,
    pub pool_count: i64,
}