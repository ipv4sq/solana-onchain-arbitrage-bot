use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::mint_do::{self, Entity as MintRecord, Model};
use crate::arb::global::db::get_db;
use anyhow::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use solana_program::pubkey::Pubkey;

pub struct MintRecordRepository;

impl MintRecordRepository {
    pub async fn upsert_mint(mint: Model) -> Result<Model> {
        let db = get_db();
        let active_model = mint_do::ActiveModel {
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
                OnConflict::column(mint_do::Column::Address)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;

        match result {
            Ok(_) => Ok(mint), // Successfully inserted, return the model we built
            Err(_) => {
                // Conflict occurred, fetch existing record
                Self::find_by_address(mint.address.0)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Failed to fetch existing mint"))
            }
        }
    }

    pub async fn find_by_address(address: Pubkey) -> Result<Option<Model>> {
        let db = get_db();
        Ok(MintRecord::find()
            .filter(mint_do::Column::Address.eq(PubkeyType::from(address)))
            .one(db)
            .await?)
    }
}
