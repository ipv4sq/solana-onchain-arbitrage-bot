use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::OnceCell;
use sea_orm::*;
use sea_orm::ActiveValue::Set;
use crate::arb::repository::{DatabaseManager, entity::{pool_mints, prelude::*}};

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
    manager: Arc<DatabaseManager>,
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
        let manager = DatabaseManager::new().await?;
        Ok(Self {
            manager: Arc::new(manager),
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

        let db = self.manager.connection();

        // Check if pool exists
        let existing = PoolMints::find()
            .filter(pool_mints::Column::PoolId.eq(&pool_id_str))
            .one(db)
            .await
            .context("Failed to check existing pool")?;

        if let Some(existing_model) = existing {
            // Update existing record
            let mut active: pool_mints::ActiveModel = existing_model.into();
            active.updated_at = Set(Some(Utc::now()));
            active.update(db).await.context("Failed to update pool")?;
        } else {
            // Insert new record
            let new_pool = pool_mints::ActiveModel {
                pool_id: Set(pool_id_str),
                desired_mint: Set(desired_mint_str),
                the_other_mint: Set(the_other_mint_str),
                dex_type: Set(dex_type.to_string()),
                created_at: Set(Some(Utc::now())),
                updated_at: Set(Some(Utc::now())),
                ..Default::default()
            };
            new_pool.insert(db).await.context("Failed to insert pool")?;
        }

        Ok(())
    }

    pub async fn list_pool_mints(&self) -> Result<Vec<PoolMint>> {
        let db = self.manager.connection();
        
        let records = PoolMints::find()
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(db)
            .await
            .context("Failed to fetch pool mints")?;

        Ok(records.into_iter().map(PoolMint::from).collect())
    }

    pub async fn list_pool_mints_by_dex(&self, dex_type: &str) -> Result<Vec<PoolMint>> {
        let db = self.manager.connection();
        
        let records = PoolMints::find()
            .filter(pool_mints::Column::DexType.eq(dex_type))
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(db)
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

        let db = self.manager.connection();
        
        let records = PoolMints::find()
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(pool_mints::Column::DesiredMint.eq(&desired_mint_str))
                            .add(pool_mints::Column::TheOtherMint.eq(&the_other_mint_str))
                    )
                    .add(
                        Condition::all()
                            .add(pool_mints::Column::DesiredMint.eq(&the_other_mint_str))
                            .add(pool_mints::Column::TheOtherMint.eq(&desired_mint_str))
                    )
            )
            .order_by_desc(pool_mints::Column::CreatedAt)
            .all(db)
            .await
            .context("Failed to find pools by mints")?;

        Ok(records.into_iter().map(PoolMint::from).collect())
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
}