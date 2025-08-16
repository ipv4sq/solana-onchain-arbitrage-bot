use crate::arb::constant::pool_owner::PoolOwnerPrograms;
use crate::arb::pool::interface::InputAccountUtil;
use crate::constants::helpers::{ToPubkey, ToSignature};
use crate::constants::mev_bot::SMB_ONCHAIN_PROGRAM_ID;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiInnerInstructions,
    UiInstruction, UiMessage, UiParsedInstruction, UiPartiallyDecodedInstruction,
};
pub fn extract_mev_instruction(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Option<(&UiPartiallyDecodedInstruction, &UiInnerInstructions)> {
    let message = match &tx.transaction.transaction {
        EncodedTransaction::Json(t) => match &t.message {
            UiMessage::Parsed(msg) => msg,
            _ => return None,
        },
        _ => return None,
    };

    // Find the MEV instruction and its index
    let (mev_ix, mev_idx) = message
        .instructions
        .iter()
        .enumerate()
        .filter_map(|(idx, x)| match x {
            UiInstruction::Compiled(_) => None,
            UiInstruction::Parsed(it) => match it {
                UiParsedInstruction::Parsed(_) => None,
                UiParsedInstruction::PartiallyDecoded(i) => Some((i, idx)),
            },
        })
        .find(|(ix, _)| ix.program_id == SMB_ONCHAIN_PROGRAM_ID)?;

    // Get the inner instructions for this specific instruction index
    let inner_ixs = tx
        .transaction
        .meta
        .as_ref()
        .and_then(|meta| match &meta.inner_instructions {
            OptionSerializer::Some(inner) => Some(inner),
            _ => None,
        })
        .and_then(|inner| inner.iter().find(|i| i.index == mev_idx as u8));

    Some((mev_ix, inner_ixs?))
}

#[cfg(test)]
mod tests {
    use crate::arb::chain::ix::{extract_swap_inner_ix, parse_swap_inner_ix};
    use super::*;
    use crate::arb::chain::rpc::fetch_tx_sync;
    use crate::arb::constant::dex_type::DexType;
    use crate::arb::program::solana_mev_bot::ix::convert_to_smb_ix;
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

        let swap_ixs = extract_swap_inner_ix(inner_ixs);
        assert!(!swap_ixs.is_empty());

        for (program_id, ix) in swap_ixs.iter() {
            println!("Found swap instruction for program: {}", program_id);
            println!("Instruction has {} accounts", ix.accounts.len());

            if program_id == PoolOwnerPrograms::METEORA_DLMM && ix.accounts.len() >= 15 {
                let swap_ix =
                    parse_swap_inner_ix(ix, &tx).expect("Failed to parse swap instruction");
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
