use crate::arb::chain::data::Transaction;
use crate::arb::chain::data::instruction::Instruction;
use crate::constants::helpers::ToAccountMeta;
use anyhow::Result;
use solana_program::instruction::AccountMeta;

pub fn get_account_keys(
    tx: &Transaction,
) -> &Vec<solana_sdk::pubkey::Pubkey> {
    &tx.message.account_keys
}

pub fn create_account_meta(
    ix: &Instruction,
    index: usize,
) -> Result<AccountMeta> {
    ix.accounts
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing account at index {}", index))
}
