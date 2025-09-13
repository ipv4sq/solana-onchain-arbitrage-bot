use crate::dex::raydium_clmm::pool_data::RaydiumClmmPoolData;
use crate::util::alias::AResult;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct RaydiumClmmIxAccount {
    pub payer: AccountMeta,
    pub amm_config: AccountMeta,
    pub pool_state: AccountMeta,
    pub input_token_account: AccountMeta,
    pub output_token_account: AccountMeta,
    pub input_vault: AccountMeta,
    pub output_vault: AccountMeta,
    pub observation_state: AccountMeta,
    pub token_program: AccountMeta,
    pub token_program_2022: AccountMeta,
    pub memo_program: AccountMeta,
    pub input_vault_mint: AccountMeta,
    pub output_vault_mint: AccountMeta,
    pub tick_array_0: AccountMeta,
    pub tick_array_1: AccountMeta,
    pub tick_array_2: AccountMeta,
    pub oracle: AccountMeta,
}

impl RaydiumClmmIxAccount {
    pub async fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &RaydiumClmmPoolData,
    ) -> AResult<RaydiumClmmIxAccount> {
        todo!()
    }
}
