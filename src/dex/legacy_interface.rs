use crate::convention::chain::instruction::Instruction;
use crate::convention::chain::Transaction;
use crate::global::enums::direction::TradeDirection;
use crate::util::alias::AResult;
use anyhow::Result;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub trait InputAccountUtil<Account, Data>: Sized {
    fn restore_from(ix: &Instruction, tx: &Transaction) -> Result<Account>;

    /*
    1. This is just for building the right list of accounts with correct permission set.
    2. If there is any bin array, it would quickly estimate.
     */
    async fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &Data,
    ) -> Result<Account>;

    // This is the most accurate version, for you to generate swap instructions directly in the future
    async fn build_accounts_with_direction_and_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &Data,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> Result<Account>;

    fn get_trade_direction(self) -> AResult<TradeDirection>;

    fn to_list(&self) -> Vec<&AccountMeta>;

    fn to_list_cloned(&self) -> Vec<AccountMeta> {
        self.to_list().into_iter().cloned().collect()
    }
}
