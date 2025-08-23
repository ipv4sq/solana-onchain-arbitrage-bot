use crate::arb::global::enums::dex_type::DexType;
use crate::arb::repository::get_repository_manager;
use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub pool_id: Pubkey,
    pub dex_type: DexType,
}

pub struct PoolsOfMint {
    pub minor_mint: Pubkey,
    pub pools: Vec<PoolInfo>,
}

pub async fn read_from_database() -> Result<Vec<PoolsOfMint>> {
    let manager = get_repository_manager().await?;
    let pool_mints = manager.pools().list_pool_mints().await?;

    let mint_pools_map = pool_mints
        .into_iter()
        .filter_map(|row| {
            let dex_type = row.dex_type;
            (dex_type != DexType::Unknown).then_some((
                row.the_other_mint.to_pubkey(),
                PoolInfo {
                    pool_id: row.pool_id.to_pubkey(),
                    dex_type,
                },
            ))
        })
        .fold(
            HashMap::<Pubkey, Vec<PoolInfo>>::new(),
            |mut map, (mint, pool_info)| {
                map.entry(mint).or_insert_with(Vec::new).push(pool_info);
                map
            },
        );

    let result = mint_pools_map
        .into_iter()
        .map(|(minor_mint, pools)| PoolsOfMint { minor_mint, pools })
        .filter(|item| !item.pools.is_empty())
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_read_from_database() {
        let result = read_from_database().await.expect("Failed to read from database");
        
        assert!(!result.is_empty(), "Should have at least one mint with pools");
        
        for pools_of_mint in &result {
            assert!(
                !pools_of_mint.pools.is_empty(),
                "Mint {} should have at least one pool",
                pools_of_mint.minor_mint
            );
        }
        
        let mints_with_multiple_pools = result
            .iter()
            .filter(|p| p.pools.len() > 1)
            .count();
        
        println!("Loaded {} mints, {} with multiple pools", result.len(), mints_with_multiple_pools);
    }
}

