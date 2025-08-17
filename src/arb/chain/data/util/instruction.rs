use crate::arb::chain::data::Transaction;
use crate::arb::chain::data::instruction::{Instruction, InnerInstructions};
use crate::arb::chain::types::SwapInstruction;
use crate::arb::pool::register::{AnyPoolConfig, RECOGNIZED_POOL_OWNER_PROGRAMS};
use std::collections::HashMap;

impl Instruction {
    
}

pub fn is_program_ix_with_min_accounts<'a>(
    ix: &'a Instruction,
    program_id: &str,
    min_accounts: usize,
) -> Option<&'a Instruction> {
    use crate::constants::helpers::ToPubkey;
    if ix.program_id == program_id.to_pubkey() {
        if ix.accounts.len() >= min_accounts {
            Some(ix)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn extract_known_swap_inner_ix(
    inners: &InnerInstructions,
    tx: &Transaction,
) -> Vec<SwapInstruction> {
    let filtered = known_swap_to_map(inners);

    filtered
        .values()
        .into_iter()
        .filter_map(|x| AnyPoolConfig::from_ix(x, tx).ok())
        .collect()
}

pub fn known_swap_to_map(
    inner_instructions: &InnerInstructions,
) -> HashMap<String, &Instruction> {
    inner_instructions
        .instructions
        .iter()
        .filter(|ix| {
            (*RECOGNIZED_POOL_OWNER_PROGRAMS).iter().any(|p| {
                use crate::constants::helpers::ToPubkey;
                ix.program_id == p.to_pubkey()
            }) && ix.accounts.len() >= 5
        })
        .map(|ix| (ix.program_id.to_string(), ix))
        .collect()
}
