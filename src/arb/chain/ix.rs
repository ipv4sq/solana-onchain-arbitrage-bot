use crate::arb::chain::types::SwapInstruction;
use crate::arb::pool::register::{AnyPoolConfig, RECOGNIZED_POOL_OWNER_PROGRAMS};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction,
    UiParsedInstruction, UiPartiallyDecodedInstruction,
};
use std::collections::HashMap;

pub fn is_program_ix_with_min_accounts<'a>(
    ix: &'a UiInstruction,
    program_id: &str,
    min_accounts: usize,
) -> Option<&'a UiPartiallyDecodedInstruction> {
    if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(decoded)) = ix {
        if decoded.program_id == program_id {
            return if decoded.accounts.len() >= min_accounts {
                Some(decoded)
            } else {
                None
            };
        }
    }
    None
}

pub fn extract_known_swap_inner_ix(
    inners: &UiInnerInstructions,
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Vec<SwapInstruction> {
    let filtered = known_swap_to_map(inners);

    filtered
        .values()
        .into_iter()
        .filter_map(|x| AnyPoolConfig::from_ix(x, tx).ok())
        .collect()
}

pub fn known_swap_to_map(
    inner_instructions: &UiInnerInstructions,
) -> HashMap<String, &UiPartiallyDecodedInstruction> {
    inner_instructions
        .instructions
        .iter()
        .filter_map(|x| match x {
            UiInstruction::Parsed(i) => match i {
                UiParsedInstruction::PartiallyDecoded(i) => Some(i),
                _ => None,
            },
            UiInstruction::Compiled(_) => None,
        })
        .filter(|ix| {
            // Only include recognized programs with sufficient accounts for a swap
            RECOGNIZED_POOL_OWNER_PROGRAMS.contains(&ix.program_id) && ix.accounts.len() >= 5
        })
        .map(|ix| (ix.program_id.clone(), ix))
        .collect()
}
