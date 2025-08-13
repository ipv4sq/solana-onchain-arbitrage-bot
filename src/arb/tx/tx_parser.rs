use crate::arb::tx::types::{SmbInstruction, SmbIxParameter};
use crate::constants::helpers::{ToPubkey, ToSignature};
use crate::constants::mev_bot::SMB_ONCHAIN_PROGRAM;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiInstruction, UiMessage,
    UiParsedInstruction,
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
) -> Option<&solana_transaction_status::UiPartiallyDecodedInstruction> {
    Some(tx)
        .and_then(|t| match &t.transaction.transaction {
            EncodedTransaction::Json(t) => match &t.message {
                UiMessage::Parsed(msg) => Some(msg),
                _ => None,
            },
            _ => None,
        })
        .and_then(|m| {
            m.instructions
                .iter()
                .filter_map(|x| match x {
                    UiInstruction::Compiled(_) => None,
                    UiInstruction::Parsed(it) => match it {
                        UiParsedInstruction::Parsed(_) => None,
                        UiParsedInstruction::PartiallyDecoded(i) => Some(i),
                    },
                })
                .find(|ix| ix.program_id.to_pubkey() == *SMB_ONCHAIN_PROGRAM)
        })
}

pub fn convert_to_smb_ix(
    ix: &solana_transaction_status::UiPartiallyDecodedInstruction,
) -> Result<SmbInstruction> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::test_utils::get_test_rpc_client;

    #[test]
    fn test_modular_functions() {
        let client = get_test_rpc_client();
        let sig = "2GNmMyHst1qd9B6FLAwBqrD6VdpxzLVxTZBuNSGYHt3Y5KtX93W6WWZGbsTfKKkbZcGi1M4KZRPQcev2VNpxLyck";
        let tx = get_tx_by_sig(&client, sig).expect("Failed to fetch transaction");
        let raw_instruction = extract_mev_instruction(&tx).expect("Failed to extract MEV instruction");
        let parsed = convert_to_smb_ix(&raw_instruction).expect("Failed to parse raw instruction");

        assert_eq!(parsed.data.instruction_discriminator, 28);
        assert_eq!(parsed.data.minimum_profit, 253345);
        assert_eq!(parsed.data.compute_unit_limit, 580000);
        assert_eq!(parsed.data.no_failure_mode, false);
        assert_eq!(parsed.data.use_flashloan, true);
        assert_eq!(parsed.accounts.len(), 59);
    }
}
