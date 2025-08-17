use crate::arb::chain::data::instruction::{InnerInstructions, Instruction};
use crate::arb::chain::data::Transaction;
use solana_sdk::pubkey::Pubkey;

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
}
