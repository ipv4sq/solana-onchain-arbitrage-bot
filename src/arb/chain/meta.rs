use crate::arb::chain::instruction::InnerInstructions;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub account_index: u8,
    pub mint: String,
    pub owner: Option<String>,
    pub program_id: Option<String>,
    pub ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Clone)]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: Option<f64>,
    pub ui_amount_string: String,
}

#[derive(Debug, Clone)]
pub struct TransactionMeta {
    pub fee: u64,
    pub compute_units_consumed: Option<u64>,
    pub log_messages: Vec<String>,
    pub inner_instructions: Vec<InnerInstructions>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalance>,
    pub post_token_balances: Vec<TokenBalance>,
    pub err: Option<String>,
    pub loaded_writable_addresses: Vec<Pubkey>,
    pub loaded_readonly_addresses: Vec<Pubkey>,
}


