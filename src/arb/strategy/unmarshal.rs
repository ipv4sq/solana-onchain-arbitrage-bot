use crate::arb::constant::dex_type::DexType;
use crate::arb::global::db::get_database;
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
    let db = get_database().await?;
    let pool_mints = db.list_pool_mints().await?;

    let mint_pools_map = pool_mints
        .into_iter()
        .filter_map(|row| {
            let dex_type = DexType::from_db_string(&row.dex_type);
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

