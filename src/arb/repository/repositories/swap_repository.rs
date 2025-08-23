use sea_orm::*;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use crate::arb::repository::{
    entity::{swap_history, prelude::*},
    error::RepositoryResult,
    traits::WithConnection,
};

pub struct SwapRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> SwapRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn record_swap(
        &self,
        tx_hash: String,
        pool_id: String,
        dex_type: String,
        input_mint: String,
        output_mint: String,
        amount_in: i64,
        amount_out: i64,
        slot: i64,
        success: bool,
        error_message: Option<String>,
    ) -> RepositoryResult<swap_history::Model> {
        let price = if amount_in > 0 {
            amount_out as f64 / amount_in as f64
        } else {
            0.0
        };

        let swap = swap_history::ActiveModel {
            transaction_hash: Set(tx_hash),
            pool_id: Set(pool_id),
            dex_type: Set(dex_type),
            input_mint: Set(input_mint),
            output_mint: Set(output_mint),
            amount_in: Set(amount_in),
            amount_out: Set(amount_out),
            price: Set(price),
            slot: Set(slot),
            timestamp: Set(Utc::now()),
            success: Set(success),
            error_message: Set(error_message),
            ..Default::default()
        };

        Ok(swap.insert(self.db).await?)
    }

    pub async fn find_by_pool(
        &self,
        pool_id: &str,
        limit: u64,
    ) -> RepositoryResult<Vec<swap_history::Model>> {
        Ok(SwapHistory::find()
            .filter(swap_history::Column::PoolId.eq(pool_id))
            .order_by_desc(swap_history::Column::Timestamp)
            .limit(limit)
            .all(self.db)
            .await?)
    }

    pub async fn find_by_tx_hash(
        &self,
        tx_hash: &str,
    ) -> RepositoryResult<Vec<swap_history::Model>> {
        Ok(SwapHistory::find()
            .filter(swap_history::Column::TransactionHash.eq(tx_hash))
            .all(self.db)
            .await?)
    }

    pub async fn find_recent(
        &self,
        hours: i64,
    ) -> RepositoryResult<Vec<swap_history::Model>> {
        let since = Utc::now() - chrono::Duration::hours(hours);
        
        Ok(SwapHistory::find()
            .filter(swap_history::Column::Timestamp.gte(since))
            .order_by_desc(swap_history::Column::Timestamp)
            .all(self.db)
            .await?)
    }

    pub async fn find_by_mints(
        &self,
        mint1: &str,
        mint2: &str,
        limit: u64,
    ) -> RepositoryResult<Vec<swap_history::Model>> {
        Ok(SwapHistory::find()
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(swap_history::Column::InputMint.eq(mint1))
                            .add(swap_history::Column::OutputMint.eq(mint2))
                    )
                    .add(
                        Condition::all()
                            .add(swap_history::Column::InputMint.eq(mint2))
                            .add(swap_history::Column::OutputMint.eq(mint1))
                    )
            )
            .order_by_desc(swap_history::Column::Timestamp)
            .limit(limit)
            .all(self.db)
            .await?)
    }

    pub async fn get_volume_stats(
        &self,
        pool_id: &str,
        hours: i64,
    ) -> RepositoryResult<VolumeStats> {
        let since = Utc::now() - chrono::Duration::hours(hours);
        
        #[derive(FromQueryResult)]
        struct Stats {
            total_volume: Option<i64>,
            swap_count: Option<i64>,
            avg_swap_size: Option<f64>,
        }

        let stats = SwapHistory::find()
            .select_only()
            .column_as(swap_history::Column::AmountIn.sum(), "total_volume")
            .column_as(swap_history::Column::Id.count(), "swap_count")
            .column_as(swap_history::Column::AmountIn.sum().div(swap_history::Column::Id.count()), "avg_swap_size")
            .filter(swap_history::Column::PoolId.eq(pool_id))
            .filter(swap_history::Column::Timestamp.gte(since))
            .filter(swap_history::Column::Success.eq(true))
            .into_model::<Stats>()
            .one(self.db)
            .await?;

        Ok(VolumeStats {
            total_volume: stats.and_then(|s| s.total_volume).unwrap_or(0),
            swap_count: stats.and_then(|s| s.swap_count).unwrap_or(0),
            avg_swap_size: stats.and_then(|s| s.avg_swap_size).unwrap_or(0.0),
        })
    }

    pub async fn cleanup_old(&self, days: i64) -> RepositoryResult<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        
        let result = SwapHistory::delete_many()
            .filter(swap_history::Column::Timestamp.lt(cutoff))
            .exec(self.db)
            .await?;

        Ok(result.rows_affected)
    }
}

impl<'a> WithConnection for SwapRepository<'a> {
    fn connection(&self) -> &DatabaseConnection {
        self.db
    }
}

#[derive(Debug, Clone)]
pub struct VolumeStats {
    pub total_volume: i64,
    pub swap_count: i64,
    pub avg_swap_size: f64,
}