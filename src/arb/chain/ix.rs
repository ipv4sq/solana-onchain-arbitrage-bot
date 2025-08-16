use crate::arb::chain::types::SwapInstruction;
use crate::arb::pool::interface::InputAccountUtil;
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

fn known_swap_to_map(
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

#[cfg(test)]
mod tests {
    use crate::arb::chain::ix::known_swap_to_map;
    use crate::arb::constant::dex_type::DexType;
    use crate::arb::constant::pool_owner::PoolOwnerPrograms;
    use crate::arb::global::rpc::fetch_tx_sync;
    use crate::arb::pool::register::AnyPoolConfig;
    use crate::arb::program::solana_mev_bot::ix::{convert_to_smb_ix, extract_mev_instruction};
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

        let swap_ixs = known_swap_to_map(inner_ixs);
        assert!(!swap_ixs.is_empty());

        for (program_id, ix) in swap_ixs.iter() {
            println!("Found swap instruction for program: {}", program_id);
            println!("Instruction has {} accounts", ix.accounts.len());

            if program_id == PoolOwnerPrograms::METEORA_DLMM && ix.accounts.len() >= 15 {
                let swap_ix =
                    AnyPoolConfig::from_ix(ix, &tx).expect("Failed to parse swap instruction");
                assert_eq!(swap_ix.dex_type, DexType::MeteoraDlmm);
                assert!(swap_ix.accounts.len() >= 15);
                println!(
                    "Successfully parsed Meteora DLMM swap with {} accounts",
                    swap_ix.accounts.len()
                );
            }
        }
    }
}
