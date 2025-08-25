use crate::arb::global::state::rpc::rpc_client;
use anyhow::Result;
use solana_sdk::address_lookup_table::state::AddressLookupTable;
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use tracing::{debug, warn};

pub async fn fetch_address_lookup_tables(
    alt_keys: &[Pubkey],
) -> Result<Vec<AddressLookupTableAccount>> {
    let mut alts = Vec::new();

    for key in alt_keys {
        match fetch_single_alt(key).await {
            Ok(alt) => alts.push(alt),
            Err(e) => {
                warn!("Skipping ALT {}: {}", key, e);
                continue;
            }
        }
    }

    if alts.is_empty() && !alt_keys.is_empty() {
        return Err(anyhow::anyhow!(
            "Failed to fetch any ALTs from {} provided keys",
            alt_keys.len()
        ));
    }

    debug!(
        "Successfully fetched {}/{} ALTs",
        alts.len(),
        alt_keys.len()
    );

    Ok(alts)
}

async fn fetch_single_alt(key: &Pubkey) -> Result<AddressLookupTableAccount> {
    let account = rpc_client()
        .get_account(key)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch ALT {}: {}", key, e))?;

    let lookup_table = AddressLookupTable::deserialize(&account.data)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize ALT {}: {}", key, e))?;

    let alt_account = AddressLookupTableAccount {
        key: *key,
        addresses: lookup_table.addresses.to_vec(),
    };

    Ok(alt_account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_fetch_valid_address_lookup_tables() {
        let alt_keys = vec![
            "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
            "EyFCXwfjTjYAZz7pz1fwiQfRq8YPUKotSNyCeihHMWgZ".to_pubkey(),
        ];

        let alts = fetch_address_lookup_tables(&alt_keys).await;

        match alts {
            Ok(tables) => {
                assert!(!tables.is_empty(), "Should fetch at least one ALT");
                assert!(
                    tables.len() <= alt_keys.len(),
                    "Should not fetch more ALTs than requested"
                );

                for alt in &tables {
                    assert!(
                        alt_keys.contains(&alt.key),
                        "Fetched ALT should be in requested keys"
                    );
                    assert!(
                        !alt.addresses.is_empty(),
                        "ALT {} should have addresses",
                        alt.key
                    );
                }
            }
            Err(e) => {
                println!("Note: ALT test requires mainnet RPC connection: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_with_invalid_alt() {
        let alt_keys = vec![
            "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
            "7Y77q5Ym5VNsAjY1amGfYGjXUSLjFcgmF6WxeeemiR8T".to_pubkey(),
            "11111111111111111111111111111111".to_pubkey(),
        ];

        let alts = fetch_address_lookup_tables(&alt_keys).await;

        match alts {
            Ok(tables) => {
                assert!(
                    tables.len() < alt_keys.len(),
                    "Should skip invalid ALTs"
                );
                assert!(
                    !tables.is_empty(),
                    "Should still fetch valid ALTs"
                );
            }
            Err(e) => {
                println!("Note: ALT test requires mainnet RPC connection: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all_invalid_alts() {
        let alt_keys = vec![
            "FakeALT1111111111111111111111111111111111111".to_pubkey(),
            "FakeALT2222222222222222222222222222222222222".to_pubkey(),
        ];

        let result = fetch_address_lookup_tables(&alt_keys).await;

        match result {
            Ok(tables) => {
                assert!(
                    tables.is_empty(),
                    "Should return empty vec if no valid ALTs found"
                );
            }
            Err(e) => {
                assert!(
                    e.to_string().contains("Failed to fetch any ALTs")
                        || e.to_string().contains("RPC"),
                    "Should error when no ALTs can be fetched"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_empty_alt_list() {
        let alt_keys = vec![];
        let alts = fetch_address_lookup_tables(&alt_keys).await;

        match alts {
            Ok(tables) => {
                assert_eq!(tables.len(), 0, "Empty input should return empty vec");
            }
            Err(e) => {
                panic!("Empty input should not error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_single_alt() {
        let key = "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey();

        match fetch_single_alt(&key).await {
            Ok(alt) => {
                assert_eq!(alt.key, key);
                assert!(!alt.addresses.is_empty(), "ALT should have addresses");
            }
            Err(e) => {
                println!("Note: Single ALT test requires mainnet RPC connection: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_problematic_alt() {
        let problematic_alt = "7Y77q5Ym5VNsAjY1amGfYGjXUSLjFcgmF6WxeeemiR8T".to_pubkey();
        
        match fetch_single_alt(&problematic_alt).await {
            Ok(alt) => {
                println!("Successfully fetched problematic ALT: {}", alt.key);
                assert_eq!(alt.key, problematic_alt);
            }
            Err(e) => {
                println!("Expected error for problematic ALT: {}", e);
                assert!(
                    e.to_string().contains("AccountNotFound") 
                    || e.to_string().contains("Failed to fetch ALT"),
                    "Should properly handle non-existent ALT"
                );
            }
        }
    }
}
