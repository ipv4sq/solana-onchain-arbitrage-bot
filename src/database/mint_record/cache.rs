use crate::database::mint_record::loader::load_mint_from_address;
use crate::database::mint_record::model::Model as MintRecord;
use crate::database::mint_record::repository::MintRecordRepository;
use crate::util::alias::MintAddress;
use crate::util::cache::persistent_cache::PersistentCache;
use crate::util::structs::cache_type::CacheType;
use once_cell::sync::Lazy;

#[allow(non_upper_case_globals)]
pub static MintCachePrimary: Lazy<PersistentCache<MintAddress, MintRecord>> = Lazy::new(|| {
    PersistentCache::new_with_custom_db(
        CacheType::MintRecord,
        100000,
        3 * 24 * 60 * 60, // 3 days
        |mint: MintAddress| async move { load_mint_from_address(&mint).await.ok() },
        Some(|key: String| async move {
            if let Ok(mint) = key.parse::<MintAddress>() {
                MintRecordRepository::find_by_address(mint)
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            }
        }),
        Some(|_key: String, record: MintRecord, _ttl: i64| async move {
            let _ = MintRecordRepository::upsert_mint(record).await;
        }),
    )
});
