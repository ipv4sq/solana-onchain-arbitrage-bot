use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::{mint_do, pool_do, MintRecord, MintRecordTable};
use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
use crate::arb::global::db::get_db;
use crate::arb::pipeline::pool_indexer::token_recorder::ensure_mint_record_exist;
use crate::arb::util::alias::MintAddress;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use anyhow::Result;
use once_cell::sync::Lazy;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
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
        let active_model = mint_do::ActiveModel {
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

    pub async fn find_by_address(address: Pubkey) -> Result<Option<MintRecord>> {
        let db = get_db();
        Ok(MintRecordTable::find()
            .filter(mint_do::Column::Address.eq(PubkeyType::from(address)))
            .one(db)
            .await?)
    }

    pub async fn find_all_with_pools() -> Result<HashMap<Pubkey, Vec<pool_do::Model>>> {
        const PAGE_SIZE: u64 = 50;
        let db = get_db();
        let mut result = HashMap::new();
        let mut page = 0u64;

        loop {
            let mints = MintRecordTable::find()
                .paginate(db, PAGE_SIZE)
                .fetch_page(page)
                .await?;

            if mints.is_empty() {
                break;
            }

            for mint in mints {
                let pubkey: Pubkey = mint.address.into();
                let pools = PoolRecordRepository::find_by_any_mint(&pubkey).await?;
                if !pools.is_empty() {
                    result.insert(pubkey, pools);
                }
            }

            page += 1;
        }

        Ok(result)
    }
}
