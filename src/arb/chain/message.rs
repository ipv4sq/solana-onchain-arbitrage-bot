use solana_sdk::pubkey::Pubkey;
use crate::arb::chain::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct Message {
    pub account_keys: Vec<Pubkey>,
    pub recent_blockhash: String,
    pub instructions: Vec<Instruction>,
}