use crate::arb::convention::chain::instruction::{InnerInstructions, Instruction};
use crate::arb::convention::chain::types::SwapInstruction;
use crate::arb::convention::chain::Transaction;
use crate::arb::dex::any_pool_config::AnyPoolConfig;
use solana_sdk::pubkey::Pubkey;

impl Transaction {
    pub fn just_inner(&self) -> Option<&Vec<InnerInstructions>> {
        self.meta.as_ref().map(|meta| &meta.inner_instructions)
    }

    pub fn all_instructions(&self) -> Vec<Instruction> {
        let mut all = self.message.instructions.clone();

        if let Some(meta) = &self.meta {
            meta.inner_instructions
                .iter()
                .flat_map(|inner| &inner.instructions)
                .for_each(|ix| all.push(ix.clone()));
        }

        all
    }

    pub fn find_top_ix_interact_with(
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

    pub fn extract_known_swap_ix(&self, ixs: &Vec<Instruction>) -> Option<Vec<SwapInstruction>> {
        let result: Vec<SwapInstruction> = ixs
            .iter()
            .filter(|ix| ix.accounts.len() > 3)
            .filter_map(|ix| AnyPoolConfig::from_ix_to_swap(ix).ok())
            .collect();
        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }
}
