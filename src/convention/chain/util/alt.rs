use crate::global::constant::duration::Interval;
use crate::sdk::rpc::methods::account::buffered_get_account;
use crate::util::cache::loading_cache::LoadingCache;
use anyhow::Result;
use futures::future::try_join_all;
use once_cell::sync::Lazy;
use solana_sdk::address_lookup_table::state::AddressLookupTable;
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;

#[allow(non_upper_case_globals)]
pub static AltCache: Lazy<LoadingCache<Pubkey, AddressLookupTableAccount>> = Lazy::new(|| {
    LoadingCache::with_ttl(200, Interval::DAY, |key: &Pubkey| {
        let key = *key;
        async move { fetch_alt(&key).await.ok() }
    })
});

pub async fn get_alt(key: &Pubkey) -> Result<AddressLookupTableAccount> {
    AltCache
        .get(key)
        .await
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch ALT {}", key))
}

pub async fn get_alt_batch(alts: &[Pubkey]) -> Result<Vec<AddressLookupTableAccount>> {
    try_join_all(alts.iter().map(get_alt)).await
}

async fn fetch_alt(key: &Pubkey) -> Result<AddressLookupTableAccount> {
    let account = buffered_get_account(key)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch ALT {}: {}", key, e))?;

    let lookup_table = AddressLookupTable::deserialize(&account.data)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize ALT {}: {}", key, e))?;

    Ok(AddressLookupTableAccount {
        key: *key,
        addresses: lookup_table.addresses.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_fetch_valid_address_lookup_tables() {
        let alt_keys = vec![
            "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
            "EyFCXwfjTjYAZz7pz1fwiQfRq8YPUKotSNyCeihHMWgZ".to_pubkey(),
        ];

        match get_alt_batch(&alt_keys).await {
            Ok(tables) => {
                assert_eq!(tables.len(), alt_keys.len());
                for (i, alt) in tables.iter().enumerate() {
                    assert_eq!(alt.key, alt_keys[i]);
                    assert!(!alt.addresses.is_empty());
                }
            }
            Err(e) => {
                println!("Note: ALT test requires mainnet RPC connection: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_with_invalid_alt_should_fail() {
        let alt_keys = vec!["11111111111111111111111111111111".to_pubkey()];

        let result = get_alt_batch(&alt_keys).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_empty_alt_list() {
        let result = get_alt_batch(&[]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_single_alt() {
        let key = "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey();

        match get_alt(&key).await {
            Ok(alt) => {
                assert_eq!(alt.key, key);
                assert!(!alt.addresses.is_empty());
            }
            Err(e) => {
                println!(
                    "Note: Single ALT test requires mainnet RPC connection: {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let alt_key = "EyFCXwfjTjYAZz7pz1fwiQfRq8YPUKotSNyCeihHMWgZ".to_pubkey();

        if let Ok(_) = get_alt(&alt_key).await {
            assert!(AltCache.get_if_present(&alt_key).await.is_some());

            AltCache.invalidate(&alt_key).await;
            assert!(AltCache.get_if_present(&alt_key).await.is_none());

            let _ = get_alt(&alt_key).await;
            assert!(AltCache.get_if_present(&alt_key).await.is_some());
        }
    }

    #[tokio::test]
    async fn test_cache_all_invalidation() {
        let alt_keys = vec![
            "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
            "EyFCXwfjTjYAZz7pz1fwiQfRq8YPUKotSNyCeihHMWgZ".to_pubkey(),
        ];

        for key in &alt_keys {
            let _ = get_alt(key).await;
        }

        AltCache.invalidate_all();

        for key in &alt_keys {
            assert!(AltCache.get_if_present(key).await.is_none());
        }
    }
}
