use crate::arb::chain::instruction::InnerInstructions;

#[derive(Debug, Clone)]
pub struct TransactionMeta {
    pub fee: u64,
    pub compute_units_consumed: Option<u64>,
    pub log_messages: Vec<String>,
    pub inner_instructions: Vec<InnerInstructions>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub err: Option<String>,
}