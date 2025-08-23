use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::OnceCell;
use crate::arb::repository::{
    RepositoryManager,
    get_repository_manager,
    entity::pool_mints,
};

#[derive(Debug, Clone)]
pub struct PoolMint {
    pub id: i32,
    pub pool_id: String,
    pub desired_mint: String,
    pub the_other_mint: String,
    pub dex_type: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<pool_mints::Model> for PoolMint {
    fn from(model: pool_mints::Model) -> Self {
        Self {
            id: model.id,
            pool_id: model.pool_id,
            desired_mint: model.desired_mint,
            the_other_mint: model.the_other_mint,
            dex_type: model.dex_type,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

pub struct Database {
    manager: Arc<RepositoryManager>,
}

static DATABASE: OnceCell<Arc<Database>> = OnceCell::const_new();

pub(in crate::arb) async fn get_database() -> Result<Arc<Database>> {
    DATABASE
        .get_or_init(|| async {
            let manager = get_repository_manager()
                .await
                .expect("Failed to initialize repository manager");
            Arc::new(Database { manager })
        })
        .await
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to get database instance"))
}

impl Database {
    pub async fn new() -> Result<Self> {
        let manager = get_repository_manager().await?;
        Ok(Self { manager })
    }

    pub async fn record_pool_and_mints(
        &self,
        pool_id: &Pubkey,
        desired_mint: &Pubkey,
        the_other_mint: &Pubkey,
        dex_type: &str,
    ) -> Result<()> {
        let pool_id_str = pool_id.to_string();
        let desired_mint_str = desired_mint.to_string();
        let the_other_mint_str = the_other_mint.to_string();

        self.manager
            .pools()
            .upsert(
                pool_id_str,
                desired_mint_str,
                the_other_mint_str,
                dex_type.to_string(),
            )
            .await
            .context("Failed to record pool and mints")?;

        Ok(())
    }

    pub async fn list_pool_mints(&self) -> Result<Vec<PoolMint>> {
        let records = self.manager
            .pools()
            .find_all()
            .await
            .context("Failed to fetch pool mints")?;

        Ok(records.into_iter().map(PoolMint::from).collect())
    }

    pub async fn list_pool_mints_by_dex(&self, dex_type: &str) -> Result<Vec<PoolMint>> {
        let records = self.manager
            .pools()
            .find_by_dex_types(vec![dex_type.to_string()])
            .await
            .context("Failed to fetch pool mints by dex")?;

        Ok(records.into_iter().map(PoolMint::from).collect())
    }

    pub async fn find_pools_by_mints(
        &self,
        desired_mint: &Pubkey,
        the_other_mint: &Pubkey,
    ) -> Result<Vec<PoolMint>> {
        let desired_mint_str = desired_mint.to_string();
        let the_other_mint_str = the_other_mint.to_string();

        let records = self.manager
            .pools()
            .find_by_mints(&desired_mint_str, &the_other_mint_str)
            .await
            .context("Failed to find pools by mints")?;

        Ok(records.into_iter().map(PoolMint::from).collect())
    }

    // Additional methods that utilize the new repository features
    
    pub async fn record_swap(
        &self,
        tx_hash: &str,
        pool_id: &str,
        dex_type: &str,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_in: u64,
        amount_out: u64,
        slot: u64,
        success: bool,
        error_message: Option<String>,
    ) -> Result<()> {
        self.manager
            .swaps()
            .record_swap(
                tx_hash.to_string(),
                pool_id.to_string(),
                dex_type.to_string(),
                input_mint.to_string(),
                output_mint.to_string(),
                amount_in as i64,
                amount_out as i64,
                slot as i64,
                success,
                error_message,
            )
            .await
            .context("Failed to record swap")?;

        Ok(())
    }

    pub async fn record_arbitrage_result(
        &self,
        tx_hash: &str,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: u64,
        output_amount: u64,
        path: Vec<String>,
        gas_cost: u64,
        slot: u64,
        success: bool,
        error_message: Option<String>,
    ) -> Result<()> {
        self.manager
            .arbitrage()
            .record_arbitrage(
                tx_hash.to_string(),
                input_mint.to_string(),
                output_mint.to_string(),
                input_amount as i64,
                output_amount as i64,
                path,
                gas_cost as i64,
                slot as i64,
                success,
                error_message,
            )
            .await
            .context("Failed to record arbitrage result")?;

        Ok(())
    }

    pub async fn update_pool_metrics(
        &self,
        pool_id: &str,
        dex_type: &str,
        tvl_usd: f64,
        volume_24h_usd: f64,
        fee_24h_usd: f64,
        swap_count_24h: u64,
    ) -> Result<()> {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        self.manager
            .metrics()
            .update_pool_metrics(
                pool_id.to_string(),
                dex_type.to_string(),
                Decimal::from_str(&tvl_usd.to_string()).unwrap_or(Decimal::ZERO),
                Decimal::from_str(&volume_24h_usd.to_string()).unwrap_or(Decimal::ZERO),
                Decimal::from_str(&fee_24h_usd.to_string()).unwrap_or(Decimal::ZERO),
                swap_count_24h as i64,
            )
            .await
            .context("Failed to update pool metrics")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::helpers::ToPubkey;

    #[tokio::test]
    async fn test_database_operations() -> Result<()> {
        let db = Database::new().await?;

        let pool_id = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_pubkey();
        let desired_mint = "So11111111111111111111111111111111111111112".to_pubkey();
        let the_other_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();
        let dex_type = "RaydiumV4";

        db.record_pool_and_mints(&pool_id, &desired_mint, &the_other_mint, dex_type)
            .await?;

        let all_pools = db.list_pool_mints().await?;
        assert!(!all_pools.is_empty());

        let raydium_pools = db.list_pool_mints_by_dex("RaydiumV4").await?;
        assert!(!raydium_pools.is_empty());

        let found_pools = db
            .find_pools_by_mints(&desired_mint, &the_other_mint)
            .await?;
        assert!(!found_pools.is_empty());
        assert_eq!(found_pools[0].pool_id, pool_id.to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_new_repository_features() -> Result<()> {
        let db = Database::new().await?;

        // Test pagination
        let (pools, total_pages) = db.manager.pools().paginate(1, 10).await?;
        assert!(total_pages >= 0);

        // Test search
        let search_results = db.manager.pools().search("Raydium").await?;
        
        // Test batch operations
        use sea_orm::ActiveValue::Set;
        use chrono::Utc;

        let test_pools = vec![
            ("test_pool_1".to_string(), "mint1".to_string(), "mint2".to_string(), "TestDex".to_string()),
            ("test_pool_2".to_string(), "mint3".to_string(), "mint4".to_string(), "TestDex".to_string()),
        ];

        let models: Vec<pool_mints::ActiveModel> = test_pools
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

        db.manager.pools().batch_create(models).await?;

        Ok(())
    }
}