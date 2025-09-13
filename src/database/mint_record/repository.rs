use crate::database::columns::PubkeyTypeString;
use crate::database::mint_record::cache::MintCachePrimary;
use crate::database::mint_record::{model, MintRecord, MintRecordTable};
use crate::global::client::db::get_db;
use crate::lined_err;
use crate::util::traits::option::OptionExt;
use anyhow::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use solana_program::pubkey::Pubkey;

pub struct MintRecordRepository;

// cache related stuff
impl MintRecordRepository {
    pub async fn get(mint: &Pubkey) -> Option<MintRecord> {
        MintCachePrimary.get(mint).await
    }

    pub async fn get_mint_or_err(mint: &Pubkey) -> Result<MintRecord> {
        Self::get(mint).await.or_else_err(lined_err!(
            "Cannot get mint from cache and db and loader: {}",
            mint
        ))
    }

    pub async fn get_repr_if_present(mint: &Pubkey) -> String {
        MintCachePrimary
            .get_if_present(mint)
            .await
            .map(|record| record.repr)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub async fn get_decimal(mint: &Pubkey) -> Option<u8> {
        Self::get(mint)
            .await
            .and_then(|record| record.decimals.try_into().ok())
    }

    pub async fn get_batch(mints: &[Pubkey]) -> Result<Vec<MintRecord>> {
        use futures::future::try_join_all;

        let futures = mints.iter().map(|mint| Self::get_mint_or_err(mint));
        try_join_all(futures).await
    }

    pub async fn get_batch_as_tuple2(mint_a: &Pubkey, mint_b: &Pubkey) -> Result<(MintRecord, MintRecord)> {
        let records = Self::get_batch(&[*mint_a, *mint_b]).await?;
        match records.as_slice() {
            [a, b] => Ok((a.clone(), b.clone())),
            _ => Err(lined_err!("Expected exactly 2 mint records")),
        }
    }
}

impl MintRecordRepository {
    pub async fn upsert_mint(mint: MintRecord) -> Result<MintRecord> {
        let db = get_db().await;
        let active_model = model::ActiveModel {
            address: Set(mint.address.clone()),
            repr: Set(mint.repr.clone()),
            decimals: Set(mint.decimals),
            program: Set(mint.program.clone()),
            created_at: NotSet,
            updated_at: NotSet,
        };

        // Try insert with on_conflict do nothing
        let result = MintRecordTable::insert(active_model)
            .on_conflict(
                OnConflict::column(model::Column::Address)
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

    pub async fn find_by_address(address: Pubkey) -> Result<Option<MintRecord>> {
        let db = get_db().await;
        Ok(MintRecordTable::find()
            .filter(model::Column::Address.eq(PubkeyTypeString::from(address)))
            .one(db)
            .await?)
    }
}
