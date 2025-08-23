use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use solana_sdk::pubkey::Pubkey;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PoolMint {
    pub id: i32,
    pub pool_id: String,
    pub desired_mint: String,
    pub the_other_mint: String,
    pub dex_type: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct Database {
    pool: Arc<PgPool>,
}

static DATABASE: OnceCell<Arc<Database>> = OnceCell::const_new();
pub(in crate::arb) async fn get_database() -> Result<Arc<Database>> {
    DATABASE
        .get_or_init(|| async {
            Arc::new(
                Database::new()
                    .await
                    .expect("Failed to initialize database"),
            )
        })
        .await
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to get database instance"))
}
impl Database {
    pub async fn new() -> Result<Self> {
        dotenv::dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set in environment or .env file")?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .context("Failed to connect to database")?;

        Ok(Self {
            pool: Arc::new(pool),
        })
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

        sqlx::query!(
            r#"
            INSERT INTO pool_mints (pool_id, desired_mint, the_other_mint, dex_type)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (pool_id) 
            DO UPDATE SET 
                updated_at = CURRENT_TIMESTAMP
            "#,
            pool_id_str,
            desired_mint_str,
            the_other_mint_str,
            dex_type
        )
        .execute(self.pool.as_ref())
        .await
        .context("Failed to insert pool and mints")?;

        Ok(())
    }

    pub async fn list_pool_mints(&self) -> Result<Vec<PoolMint>> {
        let records = sqlx::query_as::<_, PoolMint>(
            r#"
            SELECT 
                id,
                pool_id,
                desired_mint,
                the_other_mint,
                dex_type,
                created_at,
                updated_at
            FROM pool_mints
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .context("Failed to fetch pool mints")?;

        Ok(records)
    }

    pub async fn list_pool_mints_by_dex(&self, dex_type: &str) -> Result<Vec<PoolMint>> {
        let records = sqlx::query_as::<_, PoolMint>(
            r#"
            SELECT 
                id,
                pool_id,
                desired_mint,
                the_other_mint,
                dex_type,
                created_at,
                updated_at
            FROM pool_mints
            WHERE dex_type = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(dex_type)
        .fetch_all(self.pool.as_ref())
        .await
        .context("Failed to fetch pool mints by dex")?;

        Ok(records)
    }

    pub async fn find_pools_by_mints(
        &self,
        desired_mint: &Pubkey,
        the_other_mint: &Pubkey,
    ) -> Result<Vec<PoolMint>> {
        let desired_mint_str = desired_mint.to_string();
        let the_other_mint_str = the_other_mint.to_string();

        let records = sqlx::query_as::<_, PoolMint>(
            r#"
            SELECT 
                id,
                pool_id,
                desired_mint,
                the_other_mint,
                dex_type,
                created_at,
                updated_at
            FROM pool_mints
            WHERE (desired_mint = $1 AND the_other_mint = $2) 
               OR (desired_mint = $2 AND the_other_mint = $1)
            ORDER BY created_at DESC
            "#,
        )
        .bind(&desired_mint_str)
        .bind(&the_other_mint_str)
        .fetch_all(self.pool.as_ref())
        .await
        .context("Failed to find pools by mints")?;

        Ok(records)
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
        let dex_type = "raydium_v4";

        db.record_pool_and_mints(&pool_id, &desired_mint, &the_other_mint, dex_type)
            .await?;

        let all_pools = db.list_pool_mints().await?;
        assert!(!all_pools.is_empty());

        let raydium_pools = db.list_pool_mints_by_dex("raydium_v4").await?;
        assert!(!raydium_pools.is_empty());

        let found_pools = db
            .find_pools_by_mints(&desired_mint, &the_other_mint)
            .await?;
        assert!(!found_pools.is_empty());
        assert_eq!(found_pools[0].pool_id, pool_id.to_string());

        Ok(())
    }
}
