use crate::arb::constant::known_pool_program::KnownPoolPrograms;
use crate::arb::pool::interface::SwapAccountsToList;
use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmSwapAccounts;
use crate::arb::tx::types::{SmbInstruction, SmbIxParameter, SwapInstruction};
use crate::constants::helpers::{ToAccountMeta, ToPubkey, ToSignature};
use crate::constants::mev_bot::SMB_ONCHAIN_PROGRAM_ID;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiInnerInstructions,
    UiInstruction, UiMessage, UiParsedInstruction, UiPartiallyDecodedInstruction,
};
use std::collections::{HashMap, HashSet};



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
            solana_transaction_status::option_serializer::OptionSerializer::Some(inner) => {
                Some(inner)
            }
            _ => None,
        })
        .and_then(|inner| inner.iter().find(|i| i.index == mev_idx as u8));

    Some((mev_ix, inner_ixs?))
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

lazy_static::lazy_static! {
    static ref RECOGNIZED_POOL_PROGRAMS: HashSet<String> = {
        let mut set = HashSet::new();
        set.insert(KnownPoolPrograms::METEORA_DLMM.to_string());
        set.insert(KnownPoolPrograms::METEORA_DAMM_V2.to_string());
        set
    };
}

pub fn filter_swap_inner_ix(
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
        .filter(|ix| RECOGNIZED_POOL_PROGRAMS.contains(&ix.program_id))
        .map(|ix| (ix.program_id.clone(), ix))
        .collect()
}

pub fn parse_swap_inner_ix(
    ix: &UiPartiallyDecodedInstruction,
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<SwapInstruction> {
    use crate::arb::tx::constants::DexType;

    match ix.program_id.as_str() {
        KnownPoolPrograms::METEORA_DLMM => {
            let accounts = parse_meteora_dlmm(ix, tx)?;

            // Parse instruction data (assuming it's base58 encoded)
            let data = bs58::decode(&ix.data)
                .into_vec()
                .unwrap_or_else(|_| Vec::new());

            // Get pool address (the lb_pair account)
            let pool_address = accounts.lb_pair.pubkey;

            // Convert accounts to string format
            let account_strings = accounts
                .to_list()
                .into_iter()
                .map(|acc| acc.pubkey.to_string())
                .collect();

            Ok(SwapInstruction {
                dex_type: DexType::MeteoraDlmm,
                pool_address,
                accounts: account_strings,
                data,
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported program: {}", ix.program_id)),
    }
}

pub fn parse_meteora_dlmm(
    ix: &UiPartiallyDecodedInstruction,
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<MeteoraDlmmSwapAccounts> {
    use solana_program::instruction::AccountMeta;

    if ix.accounts.len() < 15 {
        return Err(anyhow::anyhow!(
            "Invalid number of accounts for Meteora DLMM swap"
        ));
    }

    let parsed_accounts = match &tx.transaction.transaction {
        EncodedTransaction::Json(t) => match &t.message {
            UiMessage::Parsed(msg) => &msg.account_keys,
            _ => return Err(anyhow::anyhow!("Transaction message is not parsed format")),
        },
        _ => return Err(anyhow::anyhow!("Transaction is not in JSON format")),
    };

    let create_account_meta = |index: usize| -> Result<AccountMeta> {
        let account_key = ix
            .accounts
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("Missing account at index {}", index))?;

        let parsed_acc = parsed_accounts
            .iter()
            .find(|acc| &acc.pubkey == account_key)
            .ok_or_else(|| {
                anyhow::anyhow!("Account {} not found in parsed accounts", account_key)
            })?;

        Ok(if parsed_acc.signer {
            account_key.to_signer()
        } else if parsed_acc.writable {
            account_key.to_writable()
        } else {
            account_key.to_readonly()
        })
    };

    Ok(MeteoraDlmmSwapAccounts {
        lb_pair: create_account_meta(0)?,
        bin_array_bitmap_extension: create_account_meta(1)?,
        reverse_x: create_account_meta(2)?,
        reverse_y: create_account_meta(3)?,
        user_token_in: create_account_meta(4)?,
        user_token_out: create_account_meta(5)?,
        token_x_mint: create_account_meta(6)?,
        token_y_mint: create_account_meta(7)?,
        oracle: create_account_meta(8)?,
        host_fee_in: create_account_meta(9)?,
        user: create_account_meta(10)?,
        token_x_program: create_account_meta(11)?,
        token_y_program: create_account_meta(12)?,
        event_authority: create_account_meta(13)?,
        program: create_account_meta(14)?,
        bin_arrays: (15..ix.accounts.len())
            .map(create_account_meta)
            .collect::<Result<Vec<_>>>()?,
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

        let swap_ixs = filter_swap_inner_ix(inner_ixs);
        assert!(!swap_ixs.is_empty());

        for (program_id, ix) in swap_ixs.iter() {
            println!("Found swap instruction for program: {}", program_id);
            println!("Instruction has {} accounts", ix.accounts.len());

            if program_id == KnownPoolPrograms::METEORA_DLMM && ix.accounts.len() >= 15 {
                let swap_ix =
                    parse_swap_inner_ix(ix, &tx).expect("Failed to parse swap instruction");
                assert_eq!(
                    swap_ix.dex_type,
                    crate::arb::tx::constants::DexType::MeteoraDlmm
                );
                assert!(swap_ix.accounts.len() >= 15);
                println!(
                    "Successfully parsed Meteora DLMM swap with {} accounts",
                    swap_ix.accounts.len()
                );
            }
        }
    }
}
