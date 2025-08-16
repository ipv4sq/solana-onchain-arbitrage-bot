use anyhow::{Result, bail};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransaction,
    UiMessage,
    UiParsedMessage,
    UiRawMessage,
    UiInstruction,
    UiParsedInstruction,
    option_serializer::OptionSerializer,
    parse_accounts::ParsedAccount,
};
use std::str::FromStr;

use crate::arb::chain::data::ToUnified;
use crate::arb::chain::data::Transaction;
use crate::arb::chain::data::Message;
use crate::arb::chain::data::instruction::{Instruction, InnerInstructions};
use crate::arb::chain::data::TransactionMeta;

impl ToUnified for EncodedConfirmedTransactionWithStatusMeta {
    fn to_unified(&self) -> Result<Transaction> {
        let signatures = match &self.transaction.transaction {
            EncodedTransaction::Json(tx) => &tx.signatures,
            _ => bail!("Only JSON encoded transactions are supported"),
        };
        
        let signature = signatures.first()
            .ok_or_else(|| anyhow::anyhow!("Transaction has no signatures"))?
            .clone();
        
        let message = match &self.transaction.transaction {
            EncodedTransaction::Json(tx) => match &tx.message {
                UiMessage::Parsed(msg) => convert_parsed_message(msg)?,
                UiMessage::Raw(msg) => convert_raw_message(msg)?,
            },
            _ => bail!("Only JSON encoded transactions are supported"),
        };
        
        let meta = self.transaction.meta.as_ref()
            .map(|m| convert_meta(m))
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
    let account_keys: Vec<Pubkey> = msg.account_keys
        .iter()
        .map(|k| Pubkey::from_str(&k.pubkey).map_err(Into::into))
        .collect::<Result<_>>()?;
    
    let instructions: Vec<Instruction> = msg.instructions
        .iter()
        .enumerate()
        .map(|(idx, ix)| convert_parsed_instruction(ix, idx, &account_keys, &msg.account_keys))
        .collect::<Result<_>>()?;
    
    Ok(Message {
        account_keys,
        recent_blockhash: msg.recent_blockhash.clone(),
        instructions,
    })
}

fn convert_raw_message(msg: &UiRawMessage) -> Result<Message> {
    let account_keys: Vec<Pubkey> = msg.account_keys
        .iter()
        .map(|k| Pubkey::from_str(k).map_err(Into::into))
        .collect::<Result<_>>()?;
    
    // Get header information for account metadata
    let num_required_signatures = msg.header.num_required_signatures as usize;
    let num_readonly_signed_accounts = msg.header.num_readonly_signed_accounts as usize;
    let num_readonly_unsigned_accounts = msg.header.num_readonly_unsigned_accounts as usize;
    
    let instructions: Vec<Instruction> = msg.instructions
        .iter()
        .enumerate()
        .map(|(idx, ix)| {
            let program_id = account_keys.get(ix.program_id_index as usize)
                .ok_or_else(|| anyhow::anyhow!("Invalid program_id_index"))?
                .clone();
            
            let accounts: Vec<AccountMeta> = ix.accounts
                .iter()
                .map(|&account_idx| {
                    let account_idx = account_idx as usize;
                    let pubkey = account_keys.get(account_idx)
                        .ok_or_else(|| anyhow::anyhow!("Invalid account index"))?
                        .clone();
                    
                    // Determine if account is signer/writable based on position
                    let is_signer = account_idx < num_required_signatures;
                    let is_writable = if is_signer {
                        account_idx < (num_required_signatures - num_readonly_signed_accounts)
                    } else {
                        let non_signer_idx = account_idx - num_required_signatures;
                        let num_writable_unsigned = account_keys.len() - num_required_signatures - num_readonly_unsigned_accounts;
                        non_signer_idx < num_writable_unsigned
                    };
                    
                    Ok(if is_signer && is_writable {
                        AccountMeta::new(pubkey, true)
                    } else if is_signer {
                        AccountMeta::new_readonly(pubkey, true)
                    } else if is_writable {
                        AccountMeta::new(pubkey, false)
                    } else {
                        AccountMeta::new_readonly(pubkey, false)
                    })
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

fn convert_parsed_instruction(ix: &UiInstruction, idx: usize, account_keys: &[Pubkey], parsed_accounts: &[ParsedAccount]) -> Result<Instruction> {
    match ix {
        UiInstruction::Compiled(compiled) => {
            let program_id = account_keys.get(compiled.program_id_index as usize)
                .ok_or_else(|| anyhow::anyhow!("Invalid program_id_index"))?
                .clone();
            
            let accounts: Vec<AccountMeta> = compiled.accounts
                .iter()
                .map(|&account_idx| {
                    let pubkey = account_keys.get(account_idx as usize)
                        .ok_or_else(|| anyhow::anyhow!("Invalid account index"))?
                        .clone();
                    let parsed_acc = parsed_accounts.get(account_idx as usize)
                        .ok_or_else(|| anyhow::anyhow!("Invalid account index in parsed_accounts"))?;
                    
                    Ok(if parsed_acc.signer {
                        AccountMeta::new(pubkey, true)
                    } else if parsed_acc.writable {
                        AccountMeta::new(pubkey, false)
                    } else {
                        AccountMeta::new_readonly(pubkey, false)
                    })
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
                    
                    let accounts: Vec<AccountMeta> = decoded.accounts
                        .iter()
                        .map(|acc_str| {
                            let pubkey = Pubkey::from_str(acc_str)?;
                            // Find the parsed account info for this pubkey
                            let parsed_acc = parsed_accounts.iter()
                                .find(|pa| pa.pubkey == *acc_str)
                                .ok_or_else(|| anyhow::anyhow!("Account {} not found in parsed_accounts", acc_str))?;
                            
                            Ok(if parsed_acc.signer {
                                AccountMeta::new(pubkey, true)
                            } else if parsed_acc.writable {
                                AccountMeta::new(pubkey, false)
                            } else {
                                AccountMeta::new_readonly(pubkey, false)
                            })
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

fn convert_meta(meta: &solana_transaction_status::UiTransactionStatusMeta) -> Result<TransactionMeta> {
    let inner_instructions = match &meta.inner_instructions {
        OptionSerializer::Some(inner) => {
            inner.iter()
                .map(|inner_ix| {
                    let instructions = inner_ix.instructions
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, ix)| {
                            convert_ui_instruction_to_unified(ix, idx).ok()
                        })
                        .collect();
                    
                    InnerInstructions {
                        parent_index: inner_ix.index,
                        instructions,
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    };
    
    let log_messages = match &meta.log_messages {
        OptionSerializer::Some(logs) => logs.clone(),
        _ => Vec::new(),
    };
    
    let compute_units_consumed = match meta.compute_units_consumed {
        OptionSerializer::Some(units) => Some(units),
        _ => None,
    };
    
    Ok(TransactionMeta {
        fee: meta.fee,
        compute_units_consumed,
        log_messages,
        inner_instructions,
        pre_balances: meta.pre_balances.clone(),
        post_balances: meta.post_balances.clone(),
        err: meta.err.as_ref().map(|e| format!("{:?}", e)),
    })
}

fn convert_ui_instruction_to_unified(ix: &UiInstruction, idx: usize) -> Result<Instruction> {
    match ix {
        UiInstruction::Compiled(compiled) => {
            let program_id = Pubkey::from_str("11111111111111111111111111111111")?;
            
            let accounts: Vec<AccountMeta> = compiled.accounts
                .iter()
                .map(|_| AccountMeta::new_readonly(Pubkey::default(), false))
                .collect();
            
            let data = bs58::decode(&compiled.data)
                .into_vec()
                .unwrap_or_default();
            
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
                    
                    // For inner instructions, we don't have account metadata
                    // so we default to readonly
                    let accounts: Vec<AccountMeta> = decoded.accounts
                        .iter()
                        .map(|acc_str| {
                            let pubkey = Pubkey::from_str(acc_str)?;
                            Ok(AccountMeta::new_readonly(pubkey, false))
                        })
                        .collect::<Result<_>>()?;
                    
                    let data = bs58::decode(&decoded.data)
                        .into_vec()
                        .unwrap_or_default();
                    
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