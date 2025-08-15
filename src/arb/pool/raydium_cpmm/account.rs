use crate::impl_swap_accounts_to_list;
use solana_program::instruction::AccountMeta;

#[derive(Debug, Clone, PartialEq)]
pub struct RaydiumCpmmSwapAccounts {
    pub payer: AccountMeta,
    pub authority: AccountMeta,
    pub amm_config: AccountMeta,
    pub pool_state: AccountMeta,
    pub input_token_account: AccountMeta,
    pub output_token_account: AccountMeta,
    pub input_vault: AccountMeta,
    pub output_vault: AccountMeta,
    pub input_token_program: AccountMeta,
    pub output_token_program: AccountMeta,
    pub input_token_mint: AccountMeta,
    pub output_token_mint: AccountMeta,
    pub observation_state: AccountMeta,
}

impl_swap_accounts_to_list!(
    RaydiumCpmmSwapAccounts,
    payer,
    authority,
    amm_config,
    pool_state,
    input_token_account,
    output_token_account,
    input_vault,
    output_vault,
    input_token_program,
    output_token_program,
    input_token_mint,
    output_token_mint,
    observation_state
);
