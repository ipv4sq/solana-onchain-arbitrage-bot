use crate::arb::constant::dex_type::DexType;
use crate::arb::global::db::get_database;
use crate::arb::pool::register::AnyPoolConfig;
use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use futures::future::join_all;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

pub struct PoolsOfMint {
    pub minor_mint: Pubkey,
    pub pools: Vec<AnyPoolConfig>,
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
                (row.pool_id.to_pubkey(), dex_type),
            ))
        })
        .fold(
            HashMap::<Pubkey, Vec<(Pubkey, DexType)>>::new(),
            |mut map, (mint, pool_info)| {
                map.entry(mint).or_insert_with(Vec::new).push(pool_info);
                map
            },
        );

    let result = join_all(
        mint_pools_map
            .into_iter()
            .map(|(minor_mint, pool_infos)| async move {
                let pools = join_all(pool_infos.into_iter().map(
                    |(pool_id, dex_type)| async move {
                        AnyPoolConfig::from_address(&pool_id, dex_type).await
                    },
                ))
                .await
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<_>>();

                (!pools.is_empty()).then_some(PoolsOfMint { minor_mint, pools })
            }),
    )
    .await
    .into_iter()
    .filter_map(|x| x)
    .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_unmarshal() {}
}
