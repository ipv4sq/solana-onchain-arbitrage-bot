use crate::arb::chain::instruction::{InnerInstructions, Instruction};
use crate::arb::chain::Transaction;
use crate::arb::chain::types::SwapInstruction;
use crate::arb::pool::register::{AnyPoolConfig, RECOGNIZED_POOL_OWNER_PROGRAMS};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

impl Transaction {
    pub fn extract_ix_and_inners(
        &self,
        mut interested_in: impl FnMut(&Pubkey) -> bool,
    ) -> Option<(&Instruction, &InnerInstructions)> {
        let (ix_index, ix) = self
            .message
            .instructions
            .iter()
            .enumerate()
            .find(|(_, ix)| interested_in(&ix.program_id))?;

        let inner_ixs = self.meta.as_ref().and_then(|meta| {
            meta.inner_instructions
                .iter()
                .find(|inner| inner.parent_index == ix_index as u8)
        })?;

        Some((ix, inner_ixs))
    }

    pub fn extract_known_swap_inner_ix(&self, inners: &InnerInstructions) -> Vec<SwapInstruction> {
        let filtered = inner_to_filtered_map(inners);

        filtered
            .values()
            .into_iter()
            .filter_map(|x| AnyPoolConfig::from_ix(x, self).ok())
            .collect()
    }
}

pub fn inner_to_filtered_map(inner_instructions: &InnerInstructions) -> HashMap<String, &Instruction> {
    inner_instructions
        .instructions
        .iter()
        .filter(|ix| (*RECOGNIZED_POOL_OWNER_PROGRAMS).contains(&ix.program_id))
        .filter(|ix| ix.accounts.len() >= 5)
        .map(|ix| (ix.program_id.to_string(), ix))
        .collect()
}
