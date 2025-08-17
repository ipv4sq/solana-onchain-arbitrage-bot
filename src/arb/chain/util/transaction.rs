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

fn inner_to_filtered_map(inner_instructions: &InnerInstructions) -> HashMap<String, &Instruction> {
    inner_instructions
        .instructions
        .iter()
        .filter(|ix| (*RECOGNIZED_POOL_OWNER_PROGRAMS).contains(&ix.program_id))
        .filter(|ix| ix.accounts.len() >= 5)
        .map(|ix| (ix.program_id.to_string(), ix))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::arb::chain::util::transaction::inner_to_filtered_map;
    use crate::arb::constant::dex_type::DexType;
    use crate::arb::constant::pool_owner::PoolOwnerPrograms;
    use crate::arb::global::rpc::fetch_tx_sync;
    use crate::arb::pool::register::AnyPoolConfig;
    use crate::arb::program::solana_mev_bot::ix::convert_to_smb_ix;
    use crate::arb::subscriber::solana_mev_bot::entry::extract_mev_instruction;
    use crate::test::test_utils::get_test_rpc_client;

    #[test]
    fn test_modular_functions() {
        let client = get_test_rpc_client();
        let sig = "2GNmMyHst1qd9B6FLAwBqrD6VdpxzLVxTZBuNSGYHt3Y5KtX93W6WWZGbsTfKKkbZcGi1M4KZRPQcev2VNpxLyck";
        let tx = fetch_tx_sync(&client, sig).expect("Failed to fetch transaction");
        let (raw_instruction, inner_ixs) =
            extract_mev_instruction(&tx).expect("Failed to extract MEV instruction");
        let parsed = convert_to_smb_ix(raw_instruction).expect("Failed to parse raw instruction");

        assert_eq!(parsed.data.instruction_discriminator, 28);
        assert_eq!(parsed.data.minimum_profit, 253345);
        assert_eq!(parsed.data.compute_unit_limit, 580000);
        assert_eq!(parsed.data.no_failure_mode, false);
        assert_eq!(parsed.data.use_flashloan, true);
        assert_eq!(parsed.accounts.len(), 59);
        assert!(inner_ixs.instructions.len() > 0);

        let swap_ixs = inner_to_filtered_map(inner_ixs);
        assert!(!swap_ixs.is_empty());

        for (program_id, ix) in swap_ixs.iter() {
            if program_id == PoolOwnerPrograms::METEORA_DLMM && ix.accounts.len() >= 15 {
                let swap_ix =
                    AnyPoolConfig::from_ix(ix, &tx).expect("Failed to parse swap instruction");
                assert_eq!(swap_ix.dex_type, DexType::MeteoraDlmm);
                assert!(swap_ix.accounts.len() >= 15);
            }
        }
    }
}
