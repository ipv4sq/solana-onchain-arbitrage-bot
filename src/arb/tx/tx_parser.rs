use crate::constants::helpers::{ToPubkey, ToSignature};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use std::str::FromStr;

#[derive(Debug)]
pub struct ParsedArbitrageInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<ParsedAccount>,
    pub data: ParsedInstructionData,
}

#[derive(Debug)]
pub struct ParsedAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
    pub description: String,
}

#[derive(Debug)]
pub struct ParsedInstructionData {
    pub instruction_discriminator: u8,
    pub minimum_profit: u64,
    pub compute_unit_limit: u32,
    pub no_failure_mode: bool,
    pub reserved: u16,
    pub use_flashloan: bool,
    pub raw_data: Vec<u8>,
}

// Raw instruction data extracted from a transaction
#[derive(Debug, Clone)]
pub struct RawInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<u8>,  // Account indices
    pub data: Vec<u8>,      // Instruction data
}

/// Function 1: Fetch a transaction by signature
pub fn get_transaction_by_signature(
    client: &RpcClient, 
    signature: &str
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    // Convert signature string to Signature type
    let sig = signature.to_sig();
    
    // Configure to support versioned transactions
    let config = solana_client::rpc_config::RpcTransactionConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
        commitment: None,
        max_supported_transaction_version: Some(0),
    };
    
    // Fetch the transaction
    client.get_transaction_with_config(&sig, config)
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))
}

/// Function 2: Extract MEV instruction from transaction if present
pub fn extract_mev_instruction(
    tx: &EncodedConfirmedTransactionWithStatusMeta
) -> Result<Option<(RawInstruction, Vec<Pubkey>)>> {
    let mev_program_id = "MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz".to_pubkey();
    
    // Decode the transaction
    match &tx.transaction.transaction {
        solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
            match &ui_tx.message {
                solana_transaction_status::UiMessage::Raw(raw_msg) => {
                    // Get all account keys including loaded addresses
                    let mut all_account_keys = Vec::new();
                    
                    // Add static account keys
                    for key_str in &raw_msg.account_keys {
                        all_account_keys.push(key_str.as_str().to_pubkey());
                    }
                    
                    // Add loaded addresses if present
                    if let Some(meta) = &tx.transaction.meta {
                        if let solana_transaction_status::option_serializer::OptionSerializer::Some(loaded) = &meta.loaded_addresses {
                            for addr in &loaded.writable {
                                all_account_keys.push(addr.as_str().to_pubkey());
                            }
                            for addr in &loaded.readonly {
                                all_account_keys.push(addr.as_str().to_pubkey());
                            }
                        }
                    }
                    
                    // Look for MEV instruction
                    for instruction in &raw_msg.instructions {
                        let program_id = all_account_keys[instruction.program_id_index as usize];
                        
                        if program_id == mev_program_id {
                            // Decode the instruction data from base58
                            let data = bs58::decode(&instruction.data)
                                .into_vec()
                                .map_err(|e| anyhow::anyhow!("Failed to decode instruction data: {}", e))?;
                            
                            return Ok(Some((
                                RawInstruction {
                                    program_id,
                                    accounts: instruction.accounts.clone(),
                                    data,
                                },
                                all_account_keys,
                            )));
                        }
                    }
                    
                    Ok(None)
                }
                _ => Err(anyhow::anyhow!("Parsed transactions not supported")),
            }
        }
        _ => Err(anyhow::anyhow!("Binary encoded transactions not supported")),
    }
}

/// Function 3: Parse raw instruction into ParsedArbitrageInstruction
pub fn parse_raw_instruction(
    raw_instruction: &RawInstruction,
    all_account_keys: &[Pubkey],
) -> Result<ParsedArbitrageInstruction> {
    // Parse accounts
    let mut accounts = Vec::new();
    for (i, account_idx) in raw_instruction.accounts.iter().enumerate() {
        let account_key = all_account_keys[*account_idx as usize];
        
        // For simplicity, we'll set signer/writable based on position
        // In a real implementation, you'd need the full message to determine this
        let is_signer = i == 0;  // First account is usually the signer
        let is_writable = true;   // Most accounts in arb tx are writable
        
        let description = match i {
            0 => "Wallet (signer)",
            1 => "SOL mint",
            2 => "Fee collector",
            3 => "Wallet SOL account",
            4 => "Token program",
            5 => "System program",
            6 => "Associated Token program",
            _ => "Pool or DEX account",
        }.to_string();
        
        accounts.push(ParsedAccount {
            pubkey: account_key,
            is_signer,
            is_writable,
            description,
        });
    }
    
    // Parse instruction data
    let data = &raw_instruction.data;
    let parsed_data = if data.len() >= 17 {
        ParsedInstructionData {
            instruction_discriminator: data[0],
            minimum_profit: u64::from_le_bytes(data[1..9].try_into()?),
            compute_unit_limit: u32::from_le_bytes(data[9..13].try_into()?),
            no_failure_mode: data[13] != 0,
            reserved: u16::from_le_bytes(data[14..16].try_into()?),
            use_flashloan: data[16] != 0,
            raw_data: data.to_vec(),
        }
    } else {
        ParsedInstructionData {
            instruction_discriminator: if data.len() > 0 { data[0] } else { 0 },
            minimum_profit: 0,
            compute_unit_limit: 0,
            no_failure_mode: false,
            reserved: 0,
            use_flashloan: false,
            raw_data: data.to_vec(),
        }
    };
    
    Ok(ParsedArbitrageInstruction {
        program_id: raw_instruction.program_id,
        accounts,
        data: parsed_data,
    })
}

pub fn parse_arbitrage_tx(tx: &EncodedConfirmedTransactionWithStatusMeta) -> Result<Vec<ParsedArbitrageInstruction>> {
    let mut parsed_instructions = Vec::new();
    
    // Get the transaction - handle both encoded and decoded formats
    let encoded_tx = &tx.transaction;
    
    // Decode the transaction
    let transaction = match &encoded_tx.transaction {
        solana_transaction_status::EncodedTransaction::LegacyBinary(_) |
        solana_transaction_status::EncodedTransaction::Binary(_, _) => {
            return Err(anyhow::anyhow!("Binary encoded transactions not supported"));
        }
        solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
            // Parse the JSON encoded transaction
            match &ui_tx.message {
                solana_transaction_status::UiMessage::Parsed(_) => {
                    return Err(anyhow::anyhow!("Parsed transactions not supported"));
                }
                solana_transaction_status::UiMessage::Raw(raw_msg) => {
                    // Build a transaction from the raw message
                    use solana_sdk::message::v0::Message as MessageV0;
                    use solana_sdk::message::Message as LegacyMessage;
                    use solana_sdk::transaction::VersionedTransaction;
                    
                    // Convert account keys from strings to Pubkeys
                    let account_keys: Vec<Pubkey> = raw_msg.account_keys
                        .iter()
                        .map(|k| k.as_str().to_pubkey())
                        .collect();
                    
                    // Parse instructions
                    let instructions: Vec<solana_sdk::instruction::CompiledInstruction> = raw_msg.instructions
                        .iter()
                        .map(|ix| solana_sdk::instruction::CompiledInstruction {
                            program_id_index: ix.program_id_index,
                            accounts: ix.accounts.clone(),
                            data: bs58::decode(&ix.data).into_vec().unwrap_or_default(),
                        })
                        .collect();
                    
                    // Create the appropriate message type
                    let message = if let Some(address_table_lookups) = &raw_msg.address_table_lookups {
                        // V0 message
                        let lookups = address_table_lookups.iter()
                            .map(|lookup| solana_sdk::message::v0::MessageAddressTableLookup {
                                account_key: lookup.account_key.as_str().to_pubkey(),
                                writable_indexes: lookup.writable_indexes.clone(),
                                readonly_indexes: lookup.readonly_indexes.clone(),
                            })
                            .collect();
                        
                        let v0_msg = MessageV0 {
                            header: solana_sdk::message::MessageHeader {
                                num_required_signatures: raw_msg.header.num_required_signatures,
                                num_readonly_signed_accounts: raw_msg.header.num_readonly_signed_accounts,
                                num_readonly_unsigned_accounts: raw_msg.header.num_readonly_unsigned_accounts,
                            },
                            account_keys,
                            recent_blockhash: solana_sdk::hash::Hash::from_str(&raw_msg.recent_blockhash).unwrap_or_default(),
                            instructions,
                            address_table_lookups: lookups,
                        };
                        solana_sdk::message::VersionedMessage::V0(v0_msg)
                    } else {
                        // Legacy message
                        let legacy_msg = LegacyMessage {
                            header: solana_sdk::message::MessageHeader {
                                num_required_signatures: raw_msg.header.num_required_signatures,
                                num_readonly_signed_accounts: raw_msg.header.num_readonly_signed_accounts,
                                num_readonly_unsigned_accounts: raw_msg.header.num_readonly_unsigned_accounts,
                            },
                            account_keys,
                            recent_blockhash: solana_sdk::hash::Hash::from_str(&raw_msg.recent_blockhash).unwrap_or_default(),
                            instructions,
                        };
                        solana_sdk::message::VersionedMessage::Legacy(legacy_msg)
                    };
                    
                    // Create versioned transaction
                    VersionedTransaction {
                        signatures: ui_tx.signatures.iter()
                            .map(|s| solana_sdk::signature::Signature::from_str(s).unwrap_or_default())
                            .collect(),
                        message,
                    }
                }
            }
        }
        solana_transaction_status::EncodedTransaction::Accounts(_) => {
            return Err(anyhow::anyhow!("Accounts format not supported"));
        }
    };
    
    // MEV bot onchain program ID
    let mev_program_id = "MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz".to_pubkey();
    
    // Get all account keys including those from address lookup tables
    // For V0 transactions, we need to get the full list from the transaction metadata
    let all_account_keys = if let Some(meta) = &tx.transaction.meta {
        if let solana_transaction_status::option_serializer::OptionSerializer::Some(loaded_addresses) = &meta.loaded_addresses {
            // Combine static keys with loaded addresses
            let mut keys = Vec::new();
            
            // Add static keys from the message
            match &transaction.message {
                solana_sdk::message::VersionedMessage::Legacy(legacy) => {
                    keys.extend_from_slice(&legacy.account_keys);
                }
                solana_sdk::message::VersionedMessage::V0(v0) => {
                    keys.extend_from_slice(&v0.account_keys);
                }
            }
            
            // Add writable loaded addresses
            for addr in &loaded_addresses.writable {
                keys.push(addr.as_str().to_pubkey());
            }
            
            // Add readonly loaded addresses
            for addr in &loaded_addresses.readonly {
                keys.push(addr.as_str().to_pubkey());
            }
            
            keys
        } else {
            // No loaded addresses, just use static keys
            match &transaction.message {
                solana_sdk::message::VersionedMessage::Legacy(legacy) => {
                    legacy.account_keys.clone()
                }
                solana_sdk::message::VersionedMessage::V0(v0) => {
                    v0.account_keys.clone()
                }
            }
        }
    } else {
        // No metadata, just use static keys
        match &transaction.message {
            solana_sdk::message::VersionedMessage::Legacy(legacy) => {
                legacy.account_keys.clone()
            }
            solana_sdk::message::VersionedMessage::V0(v0) => {
                v0.account_keys.clone()
            }
        }
    };
    
    // Get instructions based on message version
    let instructions = match &transaction.message {
        solana_sdk::message::VersionedMessage::Legacy(legacy) => &legacy.instructions,
        solana_sdk::message::VersionedMessage::V0(v0) => &v0.instructions,
    };
    
    // Find instructions that match our MEV program
    for (idx, instruction) in instructions.iter().enumerate() {
        let program_id = all_account_keys[instruction.program_id_index as usize];
        
        if program_id == mev_program_id {
            println!("Found MEV instruction at index {}", idx);
            
            // Parse accounts
            let mut accounts = Vec::new();
            for (i, account_idx) in instruction.accounts.iter().enumerate() {
                let account_key = all_account_keys[*account_idx as usize];
                let is_signer = (*account_idx as usize) < transaction.message.static_account_keys().len() && 
                                transaction.message.is_signer(*account_idx as usize);
                let is_writable = transaction.message.is_maybe_writable(*account_idx as usize);
                
                // Add description based on position (from transaction.rs structure)
                let description = match i {
                    0 => "Wallet (signer)",
                    1 => "SOL mint",
                    2 => "Fee collector",
                    3 => "Wallet SOL account",
                    4 => "Token program",
                    5 => "System program",
                    6 => "Associated Token program",
                    _ => "Pool or DEX account",
                }.to_string();
                
                accounts.push(ParsedAccount {
                    pubkey: account_key,
                    is_signer,
                    is_writable,
                    description,
                });
            }
            
            // Parse instruction data
            let data = &instruction.data;
            let parsed_data = if data.len() >= 17 {
                // Expected format from transaction.rs:
                // [0]: discriminator (1 byte) - should be 28
                // [1..9]: minimum_profit (8 bytes) 
                // [9..13]: compute_unit_limit (4 bytes)
                // [13]: no_failure_mode (1 byte)
                // [14..16]: reserved (2 bytes)
                // [16]: use_flashloan (1 byte)
                ParsedInstructionData {
                    instruction_discriminator: data[0],
                    minimum_profit: u64::from_le_bytes(data[1..9].try_into()?),
                    compute_unit_limit: u32::from_le_bytes(data[9..13].try_into()?),
                    no_failure_mode: data[13] != 0,
                    reserved: u16::from_le_bytes(data[14..16].try_into()?),
                    use_flashloan: data[16] != 0,
                    raw_data: data.to_vec(),
                }
            } else {
                ParsedInstructionData {
                    instruction_discriminator: if data.len() > 0 { data[0] } else { 0 },
                    minimum_profit: 0,
                    compute_unit_limit: 0,
                    no_failure_mode: false,
                    reserved: 0,
                    use_flashloan: false,
                    raw_data: data.to_vec(),
                }
            };
            
            parsed_instructions.push(ParsedArbitrageInstruction {
                program_id,
                accounts,
                data: parsed_data,
            });
        }
    }
    
    Ok(parsed_instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::helpers::{ToPubkey, ToSignature};
    use crate::test::test_utils::get_test_rpc_client;
    use solana_transaction_status::UiTransactionEncoding;

    #[test]
    fn test_modular_functions() {
        let client = get_test_rpc_client();
        let sig = "2GNmMyHst1qd9B6FLAwBqrD6VdpxzLVxTZBuNSGYHt3Y5KtX93W6WWZGbsTfKKkbZcGi1M4KZRPQcev2VNpxLyck";
        
        // Test function 1: Get transaction by signature
        let tx = get_transaction_by_signature(&client, sig).expect("Failed to fetch transaction");
        println!("✓ Successfully fetched transaction");
        
        // Test function 2: Extract MEV instruction
        let mev_result = extract_mev_instruction(&tx).expect("Failed to extract MEV instruction");
        
        if let Some((raw_instruction, all_account_keys)) = mev_result {
            println!("✓ Found MEV instruction");
            println!("  Program ID: {}", raw_instruction.program_id);
            println!("  Data length: {} bytes", raw_instruction.data.len());
            println!("  Number of accounts: {}", raw_instruction.accounts.len());
            println!("  Total account keys available: {}", all_account_keys.len());
            
            // Test function 3: Parse raw instruction
            let parsed = parse_raw_instruction(&raw_instruction, &all_account_keys)
                .expect("Failed to parse raw instruction");
            
            println!("\n✓ Successfully parsed instruction:");
            println!("  Discriminator: {}", parsed.data.instruction_discriminator);
            println!("  Minimum Profit: {} lamports", parsed.data.minimum_profit);
            println!("  Compute Unit Limit: {}", parsed.data.compute_unit_limit);
            println!("  No Failure Mode: {}", parsed.data.no_failure_mode);
            println!("  Use Flashloan: {}", parsed.data.use_flashloan);
            println!("  Accounts parsed: {}", parsed.accounts.len());
            
            // Verify the data matches expected values
            assert_eq!(parsed.data.instruction_discriminator, 28);
            assert_eq!(parsed.data.minimum_profit, 253345);
            assert_eq!(parsed.data.compute_unit_limit, 580000);
            assert_eq!(parsed.data.no_failure_mode, false);
            assert_eq!(parsed.data.use_flashloan, true);
            assert_eq!(parsed.accounts.len(), 59);
            
            println!("\n✓ All assertions passed!");
        } else {
            panic!("No MEV instruction found in transaction");
        }
    }
    
    #[test]
    fn test_parse_tx() {
        let client = get_test_rpc_client();
        let sig = "2GNmMyHst1qd9B6FLAwBqrD6VdpxzLVxTZBuNSGYHt3Y5KtX93W6WWZGbsTfKKkbZcGi1M4KZRPQcev2VNpxLyck";
        
        // Use get_transaction_with_config to support versioned transactions
        let config = solana_client::rpc_config::RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: None,
            max_supported_transaction_version: Some(0),
        };
        
        let tx = client
            .get_transaction_with_config(&sig.to_sig(), config)
            .unwrap();

        // Parse the transaction
        let parsed_instructions = parse_arbitrage_tx(&tx).unwrap();
        
        println!("\n=== Transaction Analysis ===");
        println!("Transaction signature: {}", sig);
        println!("Found {} MEV instructions\n", parsed_instructions.len());
        
        for (i, instruction) in parsed_instructions.iter().enumerate() {
            println!("\n--- Instruction {} ---", i);
            println!("Program ID: {}", instruction.program_id);
            
            println!("\nInstruction Data:");
            // Convert to hex manually
            let hex_str: String = instruction.data.raw_data.iter()
                .map(|b| format!("{:02x}", b))
                .collect();
            println!("  Raw data (hex): {}", hex_str);
            println!("  Raw data (base58): {}", bs58::encode(&instruction.data.raw_data).into_string());
            println!("  Raw data length: {} bytes", instruction.data.raw_data.len());
            println!("\n  Parsed fields:");
            println!("    Discriminator: {} (0x{:02x})", instruction.data.instruction_discriminator, instruction.data.instruction_discriminator);
            println!("    Minimum Profit: {} lamports", instruction.data.minimum_profit);
            println!("    Compute Unit Limit: {}", instruction.data.compute_unit_limit);
            println!("    No Failure Mode: {}", instruction.data.no_failure_mode);
            println!("    Use Flashloan: {}", instruction.data.use_flashloan);
            
            println!("\nAccounts ({} total):", instruction.accounts.len());
            for (j, account) in instruction.accounts.iter().enumerate() {
                println!("  [{}] {}", j, account.pubkey);
                println!("      Signer: {}, Writable: {}", account.is_signer, account.is_writable);
                println!("      Description: {}", account.description);
                
                // Identify known accounts
                if account.pubkey == "So11111111111111111111111111111111111111112".to_pubkey() {
                    println!("      -> This is the SOL mint");
                } else if account.pubkey == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_pubkey() {
                    println!("      -> This is the Token Program");
                } else if account.pubkey == "11111111111111111111111111111111".to_pubkey() {
                    println!("      -> This is the System Program");
                } else if account.pubkey == "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_pubkey() {
                    println!("      -> This is the Associated Token Program");
                }
            }
            
            // Try to identify pool types based on account patterns
            println!("\n=== Pool Analysis ===");
            analyze_pools(&instruction.accounts);
        }
    }
    
    fn analyze_pools(accounts: &[ParsedAccount]) {
        // Known DEX program IDs
        let raydium_v4 = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_pubkey();
        let raydium_cp = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C".to_pubkey();
        let raydium_clmm = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK".to_pubkey();
        let pump = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_pubkey();
        let meteora_dlmm = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_pubkey();  // Fixed: LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo
        let whirlpool = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_pubkey();
        
        let mut pool_count = std::collections::HashMap::new();
        
        for account in accounts {
            if account.pubkey == raydium_v4 {
                *pool_count.entry("Raydium V4").or_insert(0) += 1;
            } else if account.pubkey == raydium_cp {
                *pool_count.entry("Raydium CP").or_insert(0) += 1;
            } else if account.pubkey == raydium_clmm {
                *pool_count.entry("Raydium CLMM").or_insert(0) += 1;
            } else if account.pubkey == pump {
                *pool_count.entry("Pump.fun").or_insert(0) += 1;
            } else if account.pubkey == meteora_dlmm {
                *pool_count.entry("Meteora DLMM").or_insert(0) += 1;
            } else if account.pubkey == whirlpool {
                *pool_count.entry("Orca Whirlpool").or_insert(0) += 1;
            }
        }
        
        if !pool_count.is_empty() {
            println!("Detected DEX pools:");
            for (dex, count) in pool_count {
                println!("  - {}: {} occurrence(s)", dex, count);
            }
        }
    }
}
