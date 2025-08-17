use anyhow::{bail, Result};
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{
    option_serializer::OptionSerializer, parse_accounts::ParsedAccount,
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiInstruction, UiMessage,
    UiParsedInstruction, UiParsedMessage, UiRawMessage,
};
use std::str::FromStr;

use crate::arb::chain::instruction::{InnerInstructions, Instruction};
use crate::arb::chain::mapper::traits::ToUnified;
use crate::arb::chain::{Message, Transaction, TransactionMeta};

impl ToUnified for EncodedConfirmedTransactionWithStatusMeta {
    fn to_unified(&self) -> Result<Transaction> {
        let signatures = match &self.transaction.transaction {
            EncodedTransaction::Json(tx) => &tx.signatures,
            _ => bail!("Only JSON encoded transactions are supported"),
        };

        let signature = signatures
            .first()
            .ok_or_else(|| anyhow::anyhow!("Transaction has no signatures"))?
            .clone();

        // Get loaded addresses from meta if available
        let loaded_addresses =
            self.transaction
                .meta
                .as_ref()
                .and_then(|m| match &m.loaded_addresses {
                    OptionSerializer::Some(addrs) => Some(addrs),
                    _ => None,
                });

        let message = match &self.transaction.transaction {
            EncodedTransaction::Json(tx) => match &tx.message {
                UiMessage::Parsed(msg) => convert_parsed_message(msg)?,
                UiMessage::Raw(msg) => convert_raw_message_with_loaded(msg, loaded_addresses)?,
            },
            _ => bail!("Only JSON encoded transactions are supported"),
        };

        let meta = self
            .transaction
            .meta
            .as_ref()
            .map(|m| convert_meta(m, &message.account_keys))
            .transpose()?;

        Ok(Transaction {
            signature,
            slot: self.slot,
            message,
            meta,
        })
    }
}

fn convert_parsed_message(msg: &UiParsedMessage) -> Result<Message> {
    // Build account_keys as Vec<AccountMeta> from parsed accounts
    let account_keys: Vec<AccountMeta> = msg
        .account_keys
        .iter()
        .map(|parsed_acc| {
            let pubkey = Pubkey::from_str(&parsed_acc.pubkey)?;
            
            Ok(if parsed_acc.signer && parsed_acc.writable {
                AccountMeta::new(pubkey, true)
            } else if parsed_acc.signer {
                AccountMeta::new_readonly(pubkey, true)
            } else if parsed_acc.writable {
                AccountMeta::new(pubkey, false)
            } else {
                AccountMeta::new_readonly(pubkey, false)
            })
        })
        .collect::<Result<_>>()?;

    let instructions: Vec<Instruction> = msg
        .instructions
        .iter()
        .enumerate()
        .map(|(idx, ix)| convert_parsed_instruction(ix, idx, &account_keys))
        .collect::<Result<_>>()?;

    Ok(Message {
        account_keys,
        recent_blockhash: msg.recent_blockhash.clone(),
        instructions,
    })
}

fn convert_raw_message(msg: &UiRawMessage) -> Result<Message> {
    convert_raw_message_with_loaded(msg, None)
}

fn convert_raw_message_with_loaded(
    msg: &UiRawMessage,
    loaded_addresses: Option<&solana_transaction_status::UiLoadedAddresses>,
) -> Result<Message> {
    // Parse static account keys first
    let static_pubkeys: Vec<Pubkey> = msg
        .account_keys
        .iter()
        .map(|k| Pubkey::from_str(k).map_err(Into::into))
        .collect::<Result<_>>()?;

    // Parse loaded addresses if available
    let loaded_writable: Vec<Pubkey> = loaded_addresses
        .map(|la| la.writable.iter().filter_map(|s| Pubkey::from_str(s).ok()).collect())
        .unwrap_or_default();
    
    let loaded_readonly: Vec<Pubkey> = loaded_addresses
        .map(|la| la.readonly.iter().filter_map(|s| Pubkey::from_str(s).ok()).collect())
        .unwrap_or_default();

    // Get header information for determining permissions
    let num_required_signatures = msg.header.num_required_signatures as usize;
    let num_readonly_signed_accounts = msg.header.num_readonly_signed_accounts as usize;
    let num_readonly_unsigned_accounts = msg.header.num_readonly_unsigned_accounts as usize;
    let static_account_len = static_pubkeys.len();

    // Build account_keys as Vec<AccountMeta> with proper permissions
    let mut account_keys: Vec<AccountMeta> = Vec::new();
    
    // Process static accounts with header-based permissions
    for (idx, pubkey) in static_pubkeys.iter().enumerate() {
        let is_signer = idx < num_required_signatures;
        
        let is_writable = if idx < num_required_signatures - num_readonly_signed_accounts {
            true  // Writable signer
        } else if idx < num_required_signatures {
            false  // Readonly signer
        } else if idx < static_account_len - num_readonly_unsigned_accounts {
            true  // Writable non-signer
        } else {
            false  // Readonly non-signer
        };
        
        if is_writable {
            account_keys.push(AccountMeta::new(*pubkey, is_signer));
        } else {
            account_keys.push(AccountMeta::new_readonly(*pubkey, is_signer));
        }
    }
    
    // Add loaded writable addresses (never signers)
    for pubkey in loaded_writable {
        account_keys.push(AccountMeta::new(pubkey, false));
    }
    
    // Add loaded readonly addresses (never signers)
    for pubkey in loaded_readonly {
        account_keys.push(AccountMeta::new_readonly(pubkey, false));
    }

    let instructions: Vec<Instruction> = msg
        .instructions
        .iter()
        .enumerate()
        .map(|(idx, ix)| {
            let program_account = account_keys
                .get(ix.program_id_index as usize)
                .ok_or_else(|| anyhow::anyhow!("Invalid program_id_index"))?;
            let program_id = program_account.pubkey;

            let accounts: Vec<AccountMeta> = ix
                .accounts
                .iter()
                .map(|&account_idx| {
                    account_keys
                        .get(account_idx as usize)
                        .ok_or_else(|| anyhow::anyhow!("Invalid account index"))
                        .map(|meta| meta.clone())
                })
                .collect::<Result<_>>()?;

            let data = bs58::decode(&ix.data)
                .into_vec()
                .map_err(|e| anyhow::anyhow!("Failed to decode instruction data: {}", e))?;

            Ok(Instruction {
                program_id,
                accounts,
                data,
                instruction_index: idx,
            })
        })
        .collect::<Result<_>>()?;

    Ok(Message {
        account_keys,
        recent_blockhash: msg.recent_blockhash.clone(),
        instructions,
    })
}

fn convert_parsed_instruction(
    ix: &UiInstruction,
    idx: usize,
    account_keys: &[AccountMeta],
) -> Result<Instruction> {
    match ix {
        UiInstruction::Compiled(compiled) => {
            let program_account = account_keys
                .get(compiled.program_id_index as usize)
                .ok_or_else(|| anyhow::anyhow!("Invalid program_id_index"))?;
            let program_id = program_account.pubkey;

            let accounts: Vec<AccountMeta> = compiled
                .accounts
                .iter()
                .map(|&account_idx| {
                    account_keys
                        .get(account_idx as usize)
                        .ok_or_else(|| anyhow::anyhow!("Invalid account index"))
                        .map(|meta| meta.clone())
                })
                .collect::<Result<_>>()?;

            let data = bs58::decode(&compiled.data)
                .into_vec()
                .map_err(|e| anyhow::anyhow!("Failed to decode instruction data: {}", e))?;

            Ok(Instruction {
                program_id,
                accounts,
                data,
                instruction_index: idx,
            })
        }
        UiInstruction::Parsed(parsed) => {
            match parsed {
                UiParsedInstruction::PartiallyDecoded(decoded) => {
                    let program_id = Pubkey::from_str(&decoded.program_id)?;

                    let accounts: Vec<AccountMeta> = decoded
                        .accounts
                        .iter()
                        .map(|acc_str| {
                            let target_pubkey = Pubkey::from_str(acc_str)?;
                            // Find the account meta for this pubkey
                            account_keys
                                .iter()
                                .find(|meta| meta.pubkey == target_pubkey)
                                .ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Account {} not found in account_keys",
                                        acc_str
                                    )
                                })
                                .map(|meta| meta.clone())
                        })
                        .collect::<Result<_>>()?;

                    let data = bs58::decode(&decoded.data)
                        .into_vec()
                        .map_err(|e| anyhow::anyhow!("Failed to decode instruction data: {}", e))?;

                    Ok(Instruction {
                        program_id,
                        accounts,
                        data,
                        instruction_index: idx,
                    })
                }
                UiParsedInstruction::Parsed(_) => {
                    bail!("Fully parsed instructions are not supported")
                }
            }
        }
    }
}

fn convert_meta(
    meta: &solana_transaction_status::UiTransactionStatusMeta,
    account_keys: &[AccountMeta],
) -> Result<TransactionMeta> {
    // Extract loaded addresses from the meta for backward compatibility
    let loaded_writable_addresses: Vec<Pubkey> = match &meta.loaded_addresses {
        OptionSerializer::Some(la) => la.writable.iter().filter_map(|s| Pubkey::from_str(s).ok()).collect(),
        _ => Vec::new(),
    };
    
    let loaded_readonly_addresses: Vec<Pubkey> = match &meta.loaded_addresses {
        OptionSerializer::Some(la) => la.readonly.iter().filter_map(|s| Pubkey::from_str(s).ok()).collect(),
        _ => Vec::new(),
    };
    
    let inner_instructions = match &meta.inner_instructions {
        OptionSerializer::Some(inner) => inner
            .iter()
            .map(|inner_ix| {
                let instructions = inner_ix
                    .instructions
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, ix)| {
                        convert_ui_instruction_to_unified(ix, idx, account_keys).ok()
                    })
                    .collect();

                InnerInstructions {
                    parent_index: inner_ix.index,
                    instructions,
                }
            })
            .collect(),
        _ => Vec::new(),
    };

    Ok(TransactionMeta {
        fee: meta.fee,
        compute_units_consumed: match meta.compute_units_consumed {
            OptionSerializer::Some(units) => Some(units),
            _ => None,
        },
        log_messages: match &meta.log_messages {
            OptionSerializer::Some(logs) => logs.clone(),
            _ => Vec::new(),
        },
        inner_instructions,
        pre_balances: meta.pre_balances.clone(),
        post_balances: meta.post_balances.clone(),
        err: meta.err.as_ref().map(|e| format!("{:?}", e)),
        loaded_writable_addresses,
        loaded_readonly_addresses,
    })
}

fn convert_ui_instruction_to_unified(
    ix: &UiInstruction,
    idx: usize,
    account_keys: &[AccountMeta],
) -> Result<Instruction> {
    match ix {
        UiInstruction::Compiled(compiled) => {
            let program_account = account_keys
                .get(compiled.program_id_index as usize)
                .ok_or_else(|| anyhow::anyhow!("Invalid program_id_index in inner instruction"))?;
            let program_id = program_account.pubkey;

            let data = bs58::decode(&compiled.data).into_vec().unwrap_or_default();
            
            let accounts: Vec<AccountMeta> = compiled
                .accounts
                .iter()
                .map(|&account_idx| {
                    account_keys
                        .get(account_idx as usize)
                        .ok_or_else(|| anyhow::anyhow!("Invalid account index in inner instruction"))
                        .map(|meta| meta.clone())
                })
                .collect::<Result<_>>()?;

            Ok(Instruction {
                program_id,
                accounts,
                data,
                instruction_index: idx,
            })
        }
        UiInstruction::Parsed(parsed) => {
            match parsed {
                UiParsedInstruction::PartiallyDecoded(decoded) => {
                    let program_id = Pubkey::from_str(&decoded.program_id)?;

                    // For inner instructions, find matching account from account_keys
                    let accounts: Vec<AccountMeta> = decoded
                        .accounts
                        .iter()
                        .map(|acc_str| {
                            let target_pubkey = Pubkey::from_str(acc_str)?;
                            // Try to find the account in account_keys
                            account_keys
                                .iter()
                                .find(|meta| meta.pubkey == target_pubkey)
                                .ok_or_else(|| {
                                    // If not found, default to readonly for inner instructions
                                    Ok::<AccountMeta, anyhow::Error>(AccountMeta::new_readonly(target_pubkey, false))
                                })
                                .and_then(|meta| Ok(meta.clone()))
                                .or_else(|_| Ok(AccountMeta::new_readonly(target_pubkey, false)))
                        })
                        .collect::<Result<_>>()?;

                    let data = bs58::decode(&decoded.data).into_vec().unwrap_or_default();

                    Ok(Instruction {
                        program_id,
                        accounts,
                        data,
                        instruction_index: idx,
                    })
                }
                UiParsedInstruction::Parsed(_) => {
                    bail!("Fully parsed instructions are not supported")
                }
            }
        }
    }
}