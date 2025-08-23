use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::mint_record::{self, Entity as MintRecord, Model};
use anyhow::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use solana_program::pubkey::Pubkey;

pub struct MintRecordRepository {
    db: DatabaseConnection,
}

impl MintRecordRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert_mint(&self, mut mint: Model) -> Result<Model> {
        let active_model = mint_record::ActiveModel {
            address: Set(mint.address.clone()),
            symbol: Set(mint.symbol.clone()),
            decimals: Set(mint.decimals),
            program: Set(mint.program.clone()),
            created_at: NotSet,
            updated_at: NotSet,
        };

        // Try insert with on_conflict do nothing
        let result = MintRecord::insert(active_model)
            .on_conflict(
                OnConflict::column(mint_record::Column::Address)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.db)
            .await;

        match result {
            Ok(_) => Ok(mint), // Successfully inserted, return the model we built
            Err(_) => {
                // Conflict occurred, fetch existing record
                self.find_by_address(mint.address.0)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Failed to fetch existing mint"))
            }
        }
    }

    pub async fn find_by_address(&self, address: Pubkey) -> Result<Option<Model>> {
        Ok(MintRecord::find()
            .filter(mint_record::Column::Address.eq(PubkeyType::from(address)))
            .one(&self.db)
            .await?)
    }
}
