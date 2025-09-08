use crate::database::mint_record::loader::load_mint_from_address;
use crate::database::mint_record::model::Model as MintRecord;
use crate::database::mint_record::repository::MintRecordRepository;
use crate::util::alias::MintAddress;
use crate::util::structs::persistent_cache::PersistentCache;
use once_cell::sync::Lazy;
use std::time::Duration;

#[allow(non_upper_case_globals)]
pub static MintCache: Lazy<PersistentCache<MintAddress, MintRecord>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        10000,
        Duration::from_secs(3 * 24 * 60 * 60),
        |mint: &MintAddress| {
            let mint = *mint;
            async move { load_mint_from_address(&mint).await.ok() }
        },
        |_, record, _| async move {
            let _ = MintRecordRepository::upsert_mint(record).await;
        },
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
