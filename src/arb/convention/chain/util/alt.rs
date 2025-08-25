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
    async fn test_fetch_address_lookup_tables() {
        let alt_keys = vec![
            "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
            "q52amtQzHcXs2PA3c4Xqv1LRRZCbFMzd4CGHu1tHdp1".to_pubkey(),
        ];

        let alts = fetch_address_lookup_tables(&alt_keys)
            .await
            .expect("Failed to fetch ALTs");

        assert_eq!(alts.len(), 2);
        assert_eq!(alts[0].key, alt_keys[0]);
        assert_eq!(alts[1].key, alt_keys[1]);

        assert!(
            !alts[0].addresses.is_empty(),
            "First ALT should have addresses"
        );
        assert!(
            !alts[1].addresses.is_empty(),
            "Second ALT should have addresses"
        );

        let expected_addresses = [
            (
                "Fast9vhcG2TsyBYmaZDXtXDsNfSDDu9fk9VTLEzpJu1i".to_pubkey(),
                0,
            ),
            ("SLowcKi5NofH9rjKQJBhyuFjqRvvdLGHnXrjYzzJ6fg".to_pubkey(), 0),
            (
                "BjJhrvVBtULMJDT9bCGKmDXz22YC1t75P1nF2yoRZj8E".to_pubkey(),
                1,
            ),
            (
                "CL17xAu4Jy6CTwWoMNUcCgS6P4T4VnaNCGtnDjX66Ui5".to_pubkey(),
                1,
            ),
        ];

        for (addr, alt_index) in expected_addresses.iter() {
            assert!(
                alts[*alt_index].addresses.contains(addr),
                "ALT {} should contain address {}",
                alt_keys[*alt_index],
                addr
            );
        }
    }
}
