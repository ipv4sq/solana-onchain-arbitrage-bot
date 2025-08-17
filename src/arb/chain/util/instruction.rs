use solana_program::instruction::AccountMeta;
use crate::arb::chain::instruction::{Instruction, ParsedTransferChecked};
use crate::arb::chain::meta::TransactionMeta;
use crate::constants::addresses::TOKEN_2022_KEY;
use spl_token::instruction::TokenInstruction;
use solana_sdk::pubkey::Pubkey;

impl Instruction {
    pub fn as_sol_token_transfer_checked(&self) -> Option<ParsedTransferChecked> {
        // Verify this is a token program instruction (supports both Token and Token-2022)
        if self.program_id != spl_token::ID && self.program_id != *TOKEN_2022_KEY {
            return None;
        }
        
        // transfer_checked requires exactly 4 accounts:
        // 1. Source token account (writable)
        // 2. Token mint (read-only) 
        // 3. Destination token account (writable)
        // 4. Authority/owner (signer or read-only if multisig)
        if self.accounts.len() < 4 {
            return None;
        }

        // Parse and validate the instruction data
        if self.data.is_empty() {
            return None;
        }
        
        // The first byte should be the instruction discriminator (12 for TransferChecked)
        match TokenInstruction::unpack(&self.data) {
            Ok(TokenInstruction::TransferChecked { amount, decimals }) => {
                // For inner instructions, the writable/signer flags might not be set correctly
                // We'll be more lenient and just check the instruction format
                
                // Extract account pubkeys and return parsed data
                Some(ParsedTransferChecked {
                    source: self.accounts[0].pubkey,
                    mint: self.accounts[1].pubkey,
                    destination: self.accounts[2].pubkey,
                    authority: self.accounts[3].pubkey,
                    amount,
                    decimals,
                })
            }
            _ => None,
        }
    }
}

pub fn is_program_ix_with_min_accounts<'a>(
    ix: &'a Instruction,
    program_id: &str,
    min_accounts: usize,
) -> Option<&'a Instruction> {
    use crate::constants::helpers::ToPubkey;
    if ix.program_id == program_id.to_pubkey() {
        if ix.accounts.len() >= min_accounts {
            Some(ix)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn create_account_meta(
    ix: &Instruction,
    index: usize,
) -> anyhow::Result<AccountMeta> {
    let account = ix.accounts
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("Missing account at index {}", index))?;
    
    // The AccountMeta in the instruction already has the correct writability
    // information after our refactoring, so we can just return it directly
    Ok(account.clone())
}
