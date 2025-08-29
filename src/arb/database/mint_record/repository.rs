use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::{MintRecord, MintRecordTable};
use crate::arb::database::mint_record::model;
use crate::arb::global::db::get_db;
use crate::arb::pipeline::pool_indexer::token_recorder::ensure_mint_record_exist;
use crate::arb::util::alias::MintAddress;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use anyhow::Result;
use once_cell::sync::Lazy;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use solana_program::pubkey::Pubkey;
use std::time::Duration;

static MINT_CACHE: Lazy<PersistentCache<MintAddress, MintRecord>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        10000,
        Duration::from_secs(3 * 24 * 60 * 60),
        |mint: &MintAddress| {
            let mint = *mint;
            async move { ensure_mint_record_exist(&mint).await.ok() }
        },
        |_, _, _| async move {},
        |mint: &MintAddress| {
            let mint = *mint;
            async move {
                MintRecordRepository::find_by_address(mint)
                    .await
                    .ok()
                    .flatten()
            }
        },
    )
});

pub struct MintRecordRepository;

// cache related stuff
impl MintRecordRepository {
    pub async fn get_mint_from_cache(mint: &Pubkey) -> Result<Option<MintRecord>> {
        Ok(MINT_CACHE.get(mint).await)
    }

    pub fn get_mint_from_cache_sync(mint: &Pubkey) -> Option<MintRecord> {
        (*MINT_CACHE).get_if_present(mint)
    }

    pub fn get_symbol_from_cache_sync(mint: &Pubkey) -> String {
        (*MINT_CACHE)
            .get_if_present(mint)
            .map(|record| record.symbol)
            .unwrap_or("Unknown".parse().unwrap())
    }

    pub async fn get_decimal_from_cache(mint: &Pubkey) -> Result<Option<u8>> {
        Ok(MINT_CACHE
            .get(mint)
            .await
            .and_then(|record| record.decimals.try_into().ok()))
    }

    pub async fn invalidate_cache(mint: &Pubkey) -> Result<()> {
        MINT_CACHE.evict(mint).await;
        Ok(())
    }
}

impl MintRecordRepository {
    pub async fn upsert_mint(mint: MintRecord) -> Result<MintRecord> {
        let db = get_db();
        let active_model = model::ActiveModel {
            address: Set(mint.address.clone()),
            symbol: Set(mint.symbol.clone()),
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
        let db = get_db();
        Ok(MintRecordTable::find()
            .filter(model::Column::Address.eq(PubkeyType::from(address)))
            .one(db)
            .await?)
    }
}
