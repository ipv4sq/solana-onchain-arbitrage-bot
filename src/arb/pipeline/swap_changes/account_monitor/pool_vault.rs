use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::pipeline::swap_changes::cache::MintWithPools;
use crate::arb::util::alias::{MintAddress, PoolAddress, VaultAddress};
use crate::arb::util::structs::lazy_cache::LazyCache;
use anyhow::Result;
use sea_orm::EntityTrait;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;

#[allow(non_upper_case_globals)]
static VAULT_TO_POOL: LazyCache<VaultAddress, (MintAddress, PoolAddress)> = LazyCache::new();

pub async fn list_all_vaults() -> Result<HashSet<Pubkey>> {
    let mint_with_pools = MintRecordRepository::find_all_with_pools().await?;

    let all_vaults: HashSet<VaultAddress> = mint_with_pools
        .iter()
        .flat_map(|(mint, pools)| {
            MintWithPools.insert(*mint, pools.iter().map(|pool| pool.clone()).collect());
            for pool in pools {
                VAULT_TO_POOL.insert(pool.base_vault.into(), (*mint, pool.address.into()));
                VAULT_TO_POOL.insert(pool.quote_vault.into(), (*mint, pool.address.into()));
            }
            pools
        })
        .flat_map(|pool| vec![pool.base_vault.into(), pool.quote_vault.into()])
        .collect();

    Ok(all_vaults)
}

pub fn get_mint_and_pool_of_vault(vault: &VaultAddress) -> Option<(MintAddress, PoolAddress)> {
    VAULT_TO_POOL.get(vault)
}
