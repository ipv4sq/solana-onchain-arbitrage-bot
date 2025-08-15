use crate::arb::pool::interface::{SwapAccountsToList, SwapInputAccountUtil};
use crate::arb::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use anyhow::Result;
use itertools::concat;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiPartiallyDecodedInstruction,
};
#[derive(Debug, Clone, PartialEq)]
pub struct MeteoraDlmmInputAccounts {
    pub lb_pair: AccountMeta,
    pub bin_array_bitmap_extension: AccountMeta,
    pub reverse_x: AccountMeta,
    pub reverse_y: AccountMeta,
    pub user_token_in: AccountMeta,
    pub user_token_out: AccountMeta,
    pub token_x_mint: AccountMeta,
    pub token_y_mint: AccountMeta,
    pub oracle: AccountMeta,
    pub host_fee_in: AccountMeta,
    pub user: AccountMeta,
    pub token_x_program: AccountMeta,
    pub token_y_program: AccountMeta,
    pub event_authority: AccountMeta,
    pub program: AccountMeta,
    pub bin_arrays: Vec<AccountMeta>,
}

impl SwapAccountsToList for MeteoraDlmmInputAccounts {
    fn to_list(&self) -> Vec<&AccountMeta> {
        concat(vec![
            vec![
                &self.lb_pair,
                &self.bin_array_bitmap_extension,
                &self.reverse_x,
                &self.reverse_y,
                &self.user_token_in,
                &self.user_token_out,
                &self.token_x_mint,
                &self.token_y_mint,
                &self.oracle,
                &self.host_fee_in,
                &self.user,
                &self.token_x_program,
                &self.token_y_program,
                &self.event_authority,
                &self.program,
            ],
            self.bin_arrays.iter().collect(),
        ])
    }
}

impl SwapInputAccountUtil<MeteoraDlmmInputAccounts, MeteoraDlmmPoolData>
    for MeteoraDlmmInputAccounts
{
    fn restore_from(
        ix: &UiPartiallyDecodedInstruction,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<MeteoraDlmmInputAccounts> {
        todo!()
    }

    fn build_accounts(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: MeteoraDlmmPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
        rpc: &RpcClient,
    ) -> anyhow::Result<MeteoraDlmmInputAccounts> {
        todo!()
    }

    fn to_list(&self) -> Vec<&AccountMeta> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;

    fn expected_result() -> MeteoraDlmmInputAccounts {
        
    }
    #[test]
    fn test_build_accounts() {

    }

    fn test_restore_from() {

    }
}
