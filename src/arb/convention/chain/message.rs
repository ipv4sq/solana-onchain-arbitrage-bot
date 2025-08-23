use crate::arb::convention::chain::instruction::Instruction;
use solana_program::instruction::AccountMeta;

#[derive(Debug, Clone)]
pub struct Message {
    pub account_keys: Vec<AccountMeta>,
    pub recent_blockhash: String,
    pub instructions: Vec<Instruction>,
}