use anyhow::{Result, bail};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;

use crate::arb::sdk::yellowstone::GrpcTransactionUpdate;
use crate::arb::convention::chain::instruction::{Instruction, InnerInstructions};
use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::convention::chain::meta::{TokenBalance, UiTokenAmount};
use crate::arb::convention::chain::{Message, Transaction, TransactionMeta};

impl ToUnified for GrpcTransactionUpdate {
    fn to_unified(&self) -> Result<Transaction> {
        let transaction = self.transaction
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("TransactionUpdate has no transaction"))?;
        
        let message = transaction.message
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Transaction has no message"))?;
        
        // First, build the complete account_keys list
        let mut account_keys: Vec<Pubkey> = Vec::new();
        
        // Add static account keys from the message
        for key_bytes in &message.account_keys {
            if key_bytes.len() != 32 {
                bail!("Invalid pubkey length: {}", key_bytes.len());
            }
            let mut array = [0u8; 32];
            array.copy_from_slice(key_bytes);
            account_keys.push(Pubkey::from(array));
        }
        
        // Track boundaries for determining account properties
        let static_account_len = account_keys.len();
        let mut num_loaded_writable = 0;
        let mut num_loaded_readonly = 0;
        
        // Add loaded addresses from address lookup tables if present
        if let Some(meta) = self.meta.as_ref() {
            // Add loaded writable addresses
            for addr_bytes in &meta.loaded_writable_addresses {
                if addr_bytes.len() == 32 {
                    let mut array = [0u8; 32];
                    array.copy_from_slice(addr_bytes);
                    account_keys.push(Pubkey::from(array));
                    num_loaded_writable += 1;
                }
            }
            
            // Add loaded readonly addresses
            for addr_bytes in &meta.loaded_readonly_addresses {
                if addr_bytes.len() == 32 {
                    let mut array = [0u8; 32];
                    array.copy_from_slice(addr_bytes);
                    account_keys.push(Pubkey::from(array));
                    num_loaded_readonly += 1;
                }
            }
        }
        
        // Now build AccountMeta list with proper flags
        // Structure: [static_accounts..., loaded_writable..., loaded_readonly...]
        let mut account_metas: Vec<AccountMeta> = Vec::new();
        
        for (idx, pubkey) in account_keys.iter().enumerate() {
            let (is_writable, is_signer) = if idx >= static_account_len + num_loaded_writable {
                // This is a loaded readonly address
                (false, false)
            } else if idx >= static_account_len {
                // This is a loaded writable address
                (true, false)
            } else {
                // This is a static account - use header to determine properties
                let is_signer = message.header.as_ref()
                    .map(|h| idx < h.num_required_signatures as usize)
                    .unwrap_or(false);
                
                let is_writable = message.header.as_ref()
                    .map(|h| {
                        let num_signed = h.num_required_signatures as usize;
                        let num_signed_ro = h.num_readonly_signed_accounts as usize;
                        let num_unsigned_ro = h.num_readonly_unsigned_accounts as usize;
                        
                        if idx < num_signed - num_signed_ro {
                            true
                        } else if idx < num_signed {
                            false
                        } else if idx < static_account_len - num_unsigned_ro {
                            true
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);
                
                (is_writable, is_signer)
            };
            
            if is_writable {
                account_metas.push(AccountMeta::new(*pubkey, is_signer));
            } else {
                account_metas.push(AccountMeta::new_readonly(*pubkey, is_signer));
            }
        }
        
        let instructions: Vec<Instruction> = message.instructions
            .iter()
            .enumerate()
            .map(|(idx, ix)| {
                let program_id = account_metas.get(ix.program_id_index as usize)
                    .ok_or_else(|| anyhow::anyhow!("Invalid program_id_index"))?
                    .pubkey;
                
                let accounts: Vec<AccountMeta> = ix.accounts
                    .iter()
                    .map(|&account_idx| {
                        account_metas.get(account_idx as usize)
                            .ok_or_else(|| anyhow::anyhow!("Invalid account index"))
                            .map(|meta| meta.clone())
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
            account_keys: account_metas.clone(),
            recent_blockhash,
            instructions,
        };
        
        let meta = self.meta
            .as_ref()
            .map(|m| convert_grpc_meta(m, &unified_message.account_keys))
            .transpose()?;
        
        Ok(Transaction {
            signature: self.signature.clone(),
            slot: self.slot,
            message: unified_message,
            meta,
        })
    }
}

fn convert_grpc_meta(
    meta: &yellowstone_grpc_proto::prelude::TransactionStatusMeta,
    account_keys: &[AccountMeta],
) -> Result<TransactionMeta> {
    // Calculate boundaries - account_keys is already built as: [static, loaded_writable, loaded_readonly]
    let num_loaded_writable = meta.loaded_writable_addresses.len();
    let num_loaded_readonly = meta.loaded_readonly_addresses.len();
    let static_account_len = account_keys.len() - num_loaded_writable - num_loaded_readonly;
    
    // Extract the loaded addresses as Pubkey vectors for the TransactionMeta
    let loaded_writable_addresses: Vec<Pubkey> = account_keys[static_account_len..static_account_len + num_loaded_writable]
        .iter()
        .map(|meta| meta.pubkey)
        .collect();
    
    let loaded_readonly_addresses: Vec<Pubkey> = account_keys[static_account_len + num_loaded_writable..]
        .iter()
        .map(|meta| meta.pubkey)
        .collect();
    
    let inner_instructions: Vec<InnerInstructions> = meta.inner_instructions
        .iter()
        .map(|inner| {
            let instructions = inner.instructions
                .iter()
                .enumerate()
                .filter_map(|(idx, ix)| {
                    let program_id = account_keys.get(ix.program_id_index as usize)?
                        .pubkey;
                    
                    let accounts: Vec<AccountMeta> = ix.accounts
                        .iter()
                        .filter_map(|&account_idx| {
                            account_keys.get(account_idx as usize)
                                .map(|meta| meta.clone())
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
    
    let pre_token_balances = meta.pre_token_balances
        .iter()
        .filter_map(|balance| convert_grpc_token_balance(balance).ok())
        .collect();
    
    let post_token_balances = meta.post_token_balances
        .iter()
        .filter_map(|balance| convert_grpc_token_balance(balance).ok())
        .collect();

    Ok(TransactionMeta {
        fee: meta.fee,
        compute_units_consumed: meta.compute_units_consumed,
        log_messages: meta.log_messages.clone(),
        inner_instructions,
        pre_balances: meta.pre_balances.clone(),
        post_balances: meta.post_balances.clone(),
        pre_token_balances,
        post_token_balances,
        err: meta.err.as_ref().map(|e| format!("{:?}", e)),
        loaded_writable_addresses,
        loaded_readonly_addresses,
    })
}

fn convert_grpc_token_balance(
    balance: &yellowstone_grpc_proto::prelude::TokenBalance,
) -> Result<TokenBalance> {
    // The mint, owner, and program_id are already base58 encoded strings
    Ok(TokenBalance {
        account_index: balance.account_index as u8,
        mint: balance.mint.clone(),
        owner: if balance.owner.is_empty() {
            None
        } else {
            Some(balance.owner.clone())
        },
        program_id: if balance.program_id.is_empty() {
            None
        } else {
            Some(balance.program_id.clone())
        },
        ui_token_amount: UiTokenAmount {
            amount: balance.ui_token_amount.as_ref()
                .map(|amt| amt.amount.clone())
                .unwrap_or_default(),
            decimals: balance.ui_token_amount.as_ref()
                .map(|amt| amt.decimals as u8)
                .unwrap_or(0),
            ui_amount: balance.ui_token_amount.as_ref()
                .and_then(|amt| amt.ui_amount_string.parse::<f64>().ok()),
            ui_amount_string: balance.ui_token_amount.as_ref()
                .map(|amt| amt.ui_amount_string.clone())
                .unwrap_or_default(),
        },
    })
}