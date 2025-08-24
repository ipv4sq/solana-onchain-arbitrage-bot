use crate::arb::database::entity::MintRecord;
use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::util::types::cache::LazyCache;
use anyhow::Result;
use sea_orm::EntityTrait;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;

type VaultAddress = Pubkey;
type MintAddress = Pubkey;
type PoolAddress = Pubkey;

#[allow(non_upper_case_globals)]
static cache: LazyCache<VaultAddress, (MintAddress, PoolAddress)> = LazyCache::new();

pub async fn list_all_vaults() -> Result<HashSet<Pubkey>> {
    let mint_with_pools = MintRecordRepository::find_all_with_pools().await?;

    let all_vaults: HashSet<VaultAddress> = mint_with_pools
        .iter()
        .flat_map(|(mint, pools)| {
            for pool in pools {
                cache.insert(pool.base_vault.into(), (*mint, pool.address.into()));
                cache.insert(pool.quote_vault.into(), (*mint, pool.address.into()));
            }
            pools
        })
        .flat_map(|pool| vec![pool.base_vault.into(), pool.quote_vault.into()])
        .collect();

    Ok(all_vaults)
}

pub fn get_mint_and_pool_of_vault(vault: &VaultAddress) -> Option<(MintAddress, PoolAddress)> {
    cache.get(vault)
}
