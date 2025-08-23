use crate::arb::database::entity::mint_record::{self, Entity as MintRecord, Model};
use crate::arb::database::columns::PubkeyType;
use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use solana_program::pubkey::Pubkey;

pub struct MintRecordRepository {
    db: DatabaseConnection,
}

impl MintRecordRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert_mint(
        &self,
        address: Pubkey,
        symbol: String,
        decimals: i16,
        program: Pubkey,
    ) -> Result<Model> {
        let now = Utc::now();
        
        let mint = mint_record::ActiveModel {
            address: Set(PubkeyType::from(address)),
            symbol: Set(symbol),
            decimals: Set(decimals),
            program: Set(PubkeyType::from(program)),
            created_at: Set(now),
            updated_at: Set(now),
        };

        Ok(mint.insert(&self.db).await?)
    }

    pub async fn find_by_address(&self, address: Pubkey) -> Result<Option<Model>> {
        Ok(MintRecord::find()
            .filter(mint_record::Column::Address.eq(PubkeyType::from(address)))
            .one(&self.db)
            .await?)
    }
}