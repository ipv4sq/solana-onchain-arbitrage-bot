use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::pool_record::{
    self, Entity as PoolRecord, Model,
};
use crate::arb::global::db::get_db;
use anyhow::Result;
use sea_orm::{ActiveValue::{NotSet, Set}, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::sea_query::OnConflict;
use solana_program::pubkey::Pubkey;

pub struct PoolRecordRepository;

impl PoolRecordRepository {
    pub async fn upsert_pool(pool: Model) -> Result<Model> {
        let db = get_db();
        let active_model = pool_record::ActiveModel {
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
                OnConflict::column(pool_record::Column::Address)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;

        match result {
            Ok(_) => Ok(pool),
            Err(_) => {
                Self::find_by_address(&pool.address.0)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Failed to fetch existing pool"))
            }
        }
    }

    pub async fn find_by_mints(mint1: &Pubkey, mint2: &Pubkey) -> Result<Vec<Model>> {
        let db = get_db();
        Ok(PoolRecord::find()
            .filter(
                pool_record::Column::BaseMint
                    .eq(PubkeyType::from(*mint1))
                    .and(pool_record::Column::QuoteMint.eq(PubkeyType::from(*mint2)))
                    .or(pool_record::Column::BaseMint
                        .eq(PubkeyType::from(*mint2))
                        .and(pool_record::Column::QuoteMint.eq(PubkeyType::from(*mint1)))),
            )
            .all(db)
            .await?)
    }

    pub async fn find_by_base_mint(base_mint: &Pubkey) -> Result<Vec<Model>> {
        let db = get_db();
        Ok(PoolRecord::find()
            .filter(pool_record::Column::BaseMint.eq(PubkeyType::from(*base_mint)))
            .all(db)
            .await?)
    }

    pub async fn find_by_quote_mint(quote_mint: &Pubkey) -> Result<Vec<Model>> {
        let db = get_db();
        Ok(PoolRecord::find()
            .filter(pool_record::Column::QuoteMint.eq(PubkeyType::from(*quote_mint)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::database::entity::pool_record::PoolRecordDescriptor;
    use crate::arb::global::enums::dex_type::DexType;
    use serde_json::json;
    use std::str::FromStr;

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
