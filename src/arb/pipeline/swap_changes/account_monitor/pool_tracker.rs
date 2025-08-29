use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::util::alias::{MintAddress, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;

#[allow(non_upper_case_globals)]
pub async fn list_all_pools() -> Result<HashSet<Pubkey>> {
    let mint_with_pools = MintRecordRepository::find_all_with_pools().await?;

    let all_pools: HashSet<PoolAddress> = mint_with_pools
        .iter()
        .flat_map(|(_mint, pools)| pools.iter().map(|pool| pool.address.into()))
        .collect();

    Ok(all_pools)
}

pub async fn get_minor_mint_for_pool(pool: &PoolAddress) -> Option<MintAddress> {
    let pool = PoolRecordRepository::get_pool_by_address(pool).await?;
    MintPair(pool.base_mint.into(), pool.quote_mint.into())
        .minor_mint()
        .ok()
}
