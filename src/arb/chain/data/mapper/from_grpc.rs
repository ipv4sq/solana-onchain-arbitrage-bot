use anyhow::{Result, bail};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use std::str::FromStr;

use crate::arb::subscriber::grpc_subscription::TransactionUpdate;
use crate::arb::chain::data::ToUnified;
use crate::arb::chain::data::UnifiedTransaction;
use crate::arb::chain::data::Message;
use crate::arb::chain::data::instruction::{Instruction, InnerInstructions};
use crate::arb::chain::data::TransactionMeta;

impl ToUnified for TransactionUpdate {
    fn to_unified(&self) -> Result<UnifiedTransaction> {
        let transaction = self.transaction
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("TransactionUpdate has no transaction"))?;
        
        let message = transaction.message
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Transaction has no message"))?;
        
        let account_keys: Vec<Pubkey> = message.account_keys
            .iter()
            .map(|key_bytes| {
                if key_bytes.len() != 32 {
                    bail!("Invalid pubkey length: {}", key_bytes.len());
                }
                let mut array = [0u8; 32];
                array.copy_from_slice(key_bytes);
                Ok(Pubkey::from(array))
            })
            .collect::<Result<_>>()?;
        
        let instructions: Vec<Instruction> = message.instructions
            .iter()
            .enumerate()
            .map(|(idx, ix)| {
                let program_id = account_keys.get(ix.program_id_index as usize)
                    .ok_or_else(|| anyhow::anyhow!("Invalid program_id_index"))?
                    .clone();
                
                let accounts: Vec<AccountMeta> = ix.accounts
                    .iter()
                    .map(|&account_idx| {
                        let pubkey = account_keys.get(account_idx as usize)
                            .ok_or_else(|| anyhow::anyhow!("Invalid account index"))?
                            .clone();
                        
                        let is_signer = message.header.as_ref()
                            .map(|h| (account_idx as usize) < h.num_required_signatures as usize)
                            .unwrap_or(false);
                        
                        let is_writable = message.header.as_ref()
                            .map(|h| {
                                let num_signed = h.num_required_signatures as usize;
                                let num_signed_ro = h.num_readonly_signed_accounts as usize;
                                let num_unsigned_ro = h.num_readonly_unsigned_accounts as usize;
                                let idx = account_idx as usize;
                                
                                if idx < num_signed - num_signed_ro {
                                    true
                                } else if idx < num_signed {
                                    false
                                } else if idx < account_keys.len() - num_unsigned_ro {
                                    true
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);
                        
                        if is_writable {
                            Ok(AccountMeta::new(pubkey, is_signer))
                        } else {
                            Ok(AccountMeta::new_readonly(pubkey, is_signer))
                        }
                    })
                    .collect::<Result<_>>()?;
                
                Ok(Instruction {
                    program_id,
                    accounts,
                    data: ix.data.clone(),
                    instruction_index: idx,
                })
            })
            .collect::<Result<_>>()?;
        
        let recent_blockhash = bs58::encode(&message.recent_blockhash).into_string();
        
        let unified_message = Message {
            account_keys,
            recent_blockhash,
            instructions,
        };
        
        let meta = self.meta
            .as_ref()
            .map(|m| convert_grpc_meta(m, &unified_message.account_keys))
            .transpose()?;
        
        Ok(UnifiedTransaction {
            signature: self.signature.clone(),
            slot: self.slot,
            message: unified_message,
            meta,
        })
    }
}

fn convert_grpc_meta(
    meta: &yellowstone_grpc_proto::prelude::TransactionStatusMeta,
    account_keys: &[Pubkey],
) -> Result<TransactionMeta> {
    let inner_instructions: Vec<InnerInstructions> = meta.inner_instructions
        .iter()
        .map(|inner| {
            let instructions = inner.instructions
                .iter()
                .enumerate()
                .filter_map(|(idx, ix)| {
                    let program_id = account_keys.get(ix.program_id_index as usize)?
                        .clone();
                    
                    let accounts: Vec<AccountMeta> = ix.accounts
                        .iter()
                        .filter_map(|&account_idx| {
                            let pubkey = account_keys.get(account_idx as usize)?
                                .clone();
                            Some(AccountMeta::new_readonly(pubkey, false))
                        })
                        .collect();
                    
                    Some(Instruction {
                        program_id,
                        accounts,
                        data: ix.data.clone(),
                        instruction_index: idx,
                    })
                })
                .collect();
            
            InnerInstructions {
                parent_index: inner.index as u8,
                instructions,
            }
        })
        .collect();
    
    let log_messages = meta.log_messages.clone();
    
    Ok(TransactionMeta {
        fee: meta.fee,
        compute_units_consumed: meta.compute_units_consumed,
        log_messages,
        inner_instructions,
        pre_balances: meta.pre_balances.clone(),
        post_balances: meta.post_balances.clone(),
        err: meta.err.as_ref().map(|e| format!("{:?}", e)),
    })
}