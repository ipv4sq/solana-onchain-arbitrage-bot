use solana_program::instruction::AccountMeta;
use itertools::concat;
use crate::arb::pool::interface::SwapAccountsToList;

#[derive(Debug, Clone, PartialEq)]
pub struct MeteoraDlmmSwapAccounts {
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

impl SwapAccountsToList for MeteoraDlmmSwapAccounts {
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