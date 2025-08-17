use crate::arb::chain::instruction::InnerInstructions;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct TransactionMeta {
    pub fee: u64,
    pub compute_units_consumed: Option<u64>,
    pub log_messages: Vec<String>,
    pub inner_instructions: Vec<InnerInstructions>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub err: Option<String>,
    pub loaded_writable_addresses: Vec<Pubkey>,
    pub loaded_readonly_addresses: Vec<Pubkey>,
}


