use sea_orm::*;
use sea_orm::ActiveValue::Set;
use chrono::Utc;
use rust_decimal::Decimal;
use crate::arb::repository::core::error::RepositoryResult;
use crate::arb::repository::core::traits::WithConnection;
use super::super::entity::{arbitrage_results, ArbitrageResults};

pub struct ArbitrageRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> ArbitrageRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn record_arbitrage(
        &self,
        tx_hash: String,
        input_mint: String,
        output_mint: String,
        input_amount: i64,
        output_amount: i64,
        path: Vec<String>,  // Pool IDs in the arbitrage path
        gas_cost: i64,
        slot: i64,
        success: bool,
        error_message: Option<String>,
    ) -> RepositoryResult<arbitrage_results::Model> {
        let profit_amount = output_amount - input_amount;
        let profit_percentage = if input_amount > 0 {
            Decimal::from(profit_amount) / Decimal::from(input_amount) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        let net_profit = profit_amount - gas_cost;

        let result = arbitrage_results::ActiveModel {
            transaction_hash: Set(tx_hash),
            input_mint: Set(input_mint),
            output_mint: Set(output_mint),
            input_amount: Set(input_amount),
            output_amount: Set(output_amount),
            profit_amount: Set(profit_amount),
            profit_percentage: Set(profit_percentage),
            path: Set(serde_json::json!(path)),
            gas_cost: Set(gas_cost),
            net_profit: Set(net_profit),
            slot: Set(slot),
            timestamp: Set(Utc::now()),
            success: Set(success),
            error_message: Set(error_message),
            ..Default::default()
        };

        Ok(result.insert(self.db).await?)
    }

    pub async fn find_profitable(
        &self,
        min_profit: i64,
        limit: u64,
    ) -> RepositoryResult<Vec<arbitrage_results::Model>> {
        Ok(ArbitrageResults::find()
            .filter(arbitrage_results::Column::NetProfit.gte(min_profit))
            .filter(arbitrage_results::Column::Success.eq(true))
            .order_by_desc(arbitrage_results::Column::NetProfit)
            .limit(limit)
            .all(self.db)
            .await?)
    }

    pub async fn find_by_mints(
        &self,
        mint1: &str,
        mint2: &str,
    ) -> RepositoryResult<Vec<arbitrage_results::Model>> {
        Ok(ArbitrageResults::find()
            .filter(
                Condition::any()
                    .add(arbitrage_results::Column::InputMint.eq(mint1))
                    .add(arbitrage_results::Column::OutputMint.eq(mint1))
                    .add(arbitrage_results::Column::InputMint.eq(mint2))
                    .add(arbitrage_results::Column::OutputMint.eq(mint2))
            )
            .order_by_desc(arbitrage_results::Column::Timestamp)
            .all(self.db)
            .await?)
    }

    pub async fn get_daily_stats(
        &self,
        days_back: i64,
    ) -> RepositoryResult<Vec<DailyStats>> {
        let since = Utc::now() - chrono::Duration::days(days_back);
        
        // This would require raw SQL for proper GROUP BY date
        // For now, returning simplified version
        #[derive(FromQueryResult)]
        struct StatsResult {
            total_profit: Option<i64>,
            total_gas: Option<i64>,
            trade_count: Option<i64>,
            success_count: Option<i64>,
        }

        let stats = ArbitrageResults::find()
            .select_only()
            .column_as(arbitrage_results::Column::NetProfit.sum(), "total_profit")
            .column_as(arbitrage_results::Column::GasCost.sum(), "total_gas")
            .column_as(arbitrage_results::Column::Id.count(), "trade_count")
            .column_as(
                sea_orm::sea_query::Expr::cust("SUM(CASE WHEN success THEN 1 ELSE 0 END)"),
                "success_count"
            )
            .filter(arbitrage_results::Column::Timestamp.gte(since))
            .into_model::<StatsResult>()
            .one(self.db)
            .await?;

        let daily_stat = DailyStats {
            date: Utc::now().date_naive(),
            total_profit: stats.as_ref().and_then(|s| s.total_profit).unwrap_or(0),
            total_gas: stats.as_ref().and_then(|s| s.total_gas).unwrap_or(0),
            trade_count: stats.as_ref().and_then(|s| s.trade_count).unwrap_or(0),
            success_rate: if let Some(ref s) = stats {
                if s.trade_count.unwrap_or(0) > 0 {
                    s.success_count.unwrap_or(0) as f64 / s.trade_count.unwrap_or(1) as f64
                } else {
                    0.0
                }
            } else {
                0.0
            },
        };

        Ok(vec![daily_stat])
    }

    pub async fn find_failed(
        &self,
        limit: u64,
    ) -> RepositoryResult<Vec<arbitrage_results::Model>> {
        Ok(ArbitrageResults::find()
            .filter(arbitrage_results::Column::Success.eq(false))
            .order_by_desc(arbitrage_results::Column::Timestamp)
            .limit(limit)
            .all(self.db)
            .await?)
    }

    pub async fn get_top_paths(
        &self,
        limit: u64,
    ) -> RepositoryResult<Vec<(String, i64, i64)>> {
        // Returns (path_json, total_profit, trade_count)
        // This would require GROUP BY path which needs raw SQL
        
        let results = ArbitrageResults::find()
            .filter(arbitrage_results::Column::Success.eq(true))
            .order_by_desc(arbitrage_results::Column::NetProfit)
            .limit(limit)
            .all(self.db)
            .await?;

        Ok(results.into_iter().map(|r| {
            (r.path.to_string(), r.net_profit, 1)
        }).collect())
    }
}

impl<'a> WithConnection for ArbitrageRepository<'a> {
    fn connection(&self) -> &DatabaseConnection {
        self.db
    }
}

#[derive(Debug, Clone)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub total_profit: i64,
    pub total_gas: i64,
    pub trade_count: i64,
    pub success_rate: f64,
}