use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::pool_record::{
    self, Entity as PoolRecord, Model, PoolRecordDescriptor,
};
use crate::arb::global::enums::dex_type::DexType;
use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

pub struct PoolRecordRepository {
    db: DatabaseConnection,
}

impl PoolRecordRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn insert(&self, mut pool: Model) -> Result<Model> {
        let active_model = pool_record::ActiveModel {
            address: Set(pool.address),
            name: Set(pool.name),
            dex_type: Set(pool.dex_type),
            base_mint: Set(pool.base_mint),
            quote_mint: Set(pool.quote_mint),
            base_vault: Set(pool.base_vault),
            quote_vault: Set(pool.quote_vault),
            description: Set(pool.description),
            data_snapshot: Set(pool.data_snapshot),
            created_at: Set(pool.created_at),
            updated_at: Set(pool.updated_at),
        };

        Ok(active_model.insert(&self.db).await?)
    }

    pub async fn find_by_mints(&self, mint1: &Pubkey, mint2: &Pubkey) -> Result<Vec<Model>> {
        let pools = PoolRecord::find()
            .filter(
                pool_record::Column::BaseMint
                    .eq(PubkeyType::from(*mint1))
                    .and(pool_record::Column::QuoteMint.eq(PubkeyType::from(*mint2)))
                    .or(pool_record::Column::BaseMint
                        .eq(PubkeyType::from(*mint2))
                        .and(pool_record::Column::QuoteMint.eq(PubkeyType::from(*mint1)))),
            )
            .all(&self.db)
            .await?;

        Ok(pools)
    }

    pub async fn find_by_base_mint(&self, base_mint: &Pubkey) -> Result<Vec<Model>> {
        let pools = PoolRecord::find()
            .filter(pool_record::Column::BaseMint.eq(PubkeyType::from(*base_mint)))
            .all(&self.db)
            .await?;

        Ok(pools)
    }

    pub async fn find_by_quote_mint(&self, quote_mint: &Pubkey) -> Result<Vec<Model>> {
        let pools = PoolRecord::find()
            .filter(pool_record::Column::QuoteMint.eq(PubkeyType::from(*quote_mint)))
            .all(&self.db)
            .await?;

        Ok(pools)
    }

    pub async fn find_by_address(&self, address: &Pubkey) -> Result<Option<Model>> {
        let pool = PoolRecord::find_by_id(PubkeyType::from(*address))
            .one(&self.db)
            .await?;

        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use sqlx::encode::IsNull::No;

    #[test]
    fn test_pool_record_model_creation() {
        let pool_address = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let wsol = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let usdc = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let vault1 = Pubkey::from_str("11111111111111111111111111111113").unwrap();
        let vault2 = Pubkey::from_str("11111111111111111111111111111114").unwrap();

        let model = Model {
            address: pool_address.into(),
            name: "Test Pool".to_string(),
            dex_type: DexType::RaydiumV4,
            base_mint: wsol.into(),
            quote_mint: usdc.into(),
            base_vault: vault1.into(),
            quote_vault: vault2.into(),
            description: PoolRecordDescriptor {
                base_symbol: "SOL".to_string(),
                quote_symbol: "USDC".to_string(),
                base: wsol,
                quote: usdc,
            },
            data_snapshot: json!({"test": "data"}),
            created_at: None,
            updated_at: None,
        };

        assert_eq!(model.address.0, pool_address);
        assert_eq!(model.name, "Test Pool");
        assert_eq!(model.base_mint.0, wsol);
        assert_eq!(model.quote_mint.0, usdc);
    }

    #[test]
    fn test_pool_record_descriptor() {
        let wsol = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let usdc = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let descriptor = PoolRecordDescriptor {
            base_symbol: "SOL".to_string(),
            quote_symbol: "USDC".to_string(),
            base: wsol,
            quote: usdc,
        };

        let json_value = serde_json::to_value(descriptor.clone()).unwrap();
        let deserialized: PoolRecordDescriptor = serde_json::from_value(json_value).unwrap();

        assert_eq!(deserialized.base_symbol, "SOL");
        assert_eq!(deserialized.quote_symbol, "USDC");
        assert_eq!(deserialized.base, wsol);
        assert_eq!(deserialized.quote, usdc);
    }
}
