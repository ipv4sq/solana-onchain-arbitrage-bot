use crate::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::util::alias::AResult;
use crate::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct RaydiumCammIxAccount {}

impl RaydiumCammIxAccount {
    async fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &MeteoraDammV2PoolData,
    ) -> AResult<RaydiumCammIxAccount> {
        todo!()
    }
}
