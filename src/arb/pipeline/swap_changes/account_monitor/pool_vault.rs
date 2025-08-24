use crate::arb::database::entity::MintRecord;
use crate::arb::database::repositories::MintRecordRepository;
use anyhow::Result;
use sea_orm::EntityTrait;
use solana_program::pubkey::Pubkey;

pub async fn list_all_vaults() -> Result<Vec<Pubkey>> {
    let mint_with_pools = MintRecordRepository::find_all_with_pools().await?;

    let all_vaults = mint_with_pools
        .iter()
        .flat_map(|(mint, pools)| pools)
        .flat_map(|pool| vec![pool.base_vault.into(), pool.quote_vault.into()])
        .collect();

    Ok(all_vaults)
}
