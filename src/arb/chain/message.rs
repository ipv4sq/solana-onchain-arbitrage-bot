use solana_sdk::pubkey::Pubkey;
use crate::arb::chain::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct Message {
    pub account_keys: Vec<Pubkey>,
    pub recent_blockhash: String,
    pub instructions: Vec<Instruction>,
    pub header: Option<MessageHeader>,
}

#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}