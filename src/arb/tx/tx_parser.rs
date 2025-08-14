use crate::arb::tx::types::{SmbInstruction, SmbIxParameter, SwapInstruction};
use crate::constants::helpers::{ToPubkey, ToSignature};
use crate::constants::mev_bot::SMB_ONCHAIN_PROGRAM_ID;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, InnerInstructions,
    UiInnerInstructions, UiInstruction, UiMessage, UiParsedInstruction,
    UiPartiallyDecodedInstruction,
};

pub fn get_tx_by_sig(
    client: &RpcClient,
    signature: &str,
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    let sig = signature.to_sig();

    let config = solana_client::rpc_config::RpcTransactionConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::JsonParsed),
        commitment: None,
        max_supported_transaction_version: Some(0),
    };

    client
        .get_transaction_with_config(&sig, config)
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))
}

pub fn extract_mev_instruction(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Option<(
    &UiPartiallyDecodedInstruction,
    usize,
    Option<&UiInnerInstructions>,
)> {
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
            solana_transaction_status::option_serializer::OptionSerializer::Some(inner) => {
                Some(inner)
            }
            _ => None,
        })
        .and_then(|inner| inner.iter().find(|i| i.index == mev_idx as u8));

    Some((mev_ix, mev_idx, inner_ixs))
}

pub fn convert_to_smb_ix(ix: &UiPartiallyDecodedInstruction) -> Result<SmbInstruction> {
    let data_bytes = bs58::decode(&ix.data)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Failed to decode instruction data: {}", e))?;
    let data = SmbIxParameter::from_bytes(&data_bytes)?;
    let accounts = ix.accounts.iter().map(|acc| acc.to_pubkey()).collect();

    Ok(SmbInstruction {
        program_id: ix.program_id.to_pubkey(),
        accounts,
        data,
    })
}

pub fn filter_swap_inner_ix(
    inner_instructions: &UiInnerInstructions,
) -> Option<Vec<&UiParsedInstruction>> {
    todo!()
}

pub fn parse_swap_inner_ix(inner_ix: &UiPartiallyDecodedInstruction) -> Result<SwapInstruction> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::test_utils::get_test_rpc_client;

    #[test]
    fn test_modular_functions() {
        let client = get_test_rpc_client();
        let sig = "2GNmMyHst1qd9B6FLAwBqrD6VdpxzLVxTZBuNSGYHt3Y5KtX93W6WWZGbsTfKKkbZcGi1M4KZRPQcev2VNpxLyck";
        let tx = get_tx_by_sig(&client, sig).expect("Failed to fetch transaction");
        let (raw_instruction, index, inner_ixs) =
            extract_mev_instruction(&tx).expect("Failed to extract MEV instruction");
        let parsed = convert_to_smb_ix(raw_instruction).expect("Failed to parse raw instruction");

        // Verify the instruction was found and parsed correctly
        assert!(index > 0, "MEV instruction should not be at index 0");
        assert_eq!(parsed.data.instruction_discriminator, 28);
        assert_eq!(parsed.data.minimum_profit, 253345);
        assert_eq!(parsed.data.compute_unit_limit, 580000);
        assert_eq!(parsed.data.no_failure_mode, false);
        assert_eq!(parsed.data.use_flashloan, true);
        assert_eq!(parsed.accounts.len(), 59);

        // Check if inner instructions were found
        if let Some(inner) = inner_ixs {
            println!(
                "Found {} inner instructions for MEV instruction",
                inner.instructions.len()
            );
        }
    }
}
