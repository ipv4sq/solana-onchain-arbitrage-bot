use crate::convention::chain::instruction::{Instruction, ParsedTransferChecked};
use crate::global::constant::token_program::TokenProgram;
use crate::util::alias::AResult;
use crate::{f, unit_ok};
use anyhow::anyhow;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction::TokenInstruction;

impl Instruction {
    pub fn as_sol_token_transfer_checked(&self) -> Option<ParsedTransferChecked> {
        // Verify this is a token program instruction (supports both Token and Token-2022)
        if self.program_id != spl_token::ID && self.program_id != TokenProgram::TOKEN_2022 {
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

    pub fn account_at(&self, index: usize) -> anyhow::Result<AccountMeta> {
        let account = self
            .accounts
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("Missing account at index {}", index))?;

        // The AccountMeta in the instruction already has the correct writability
        // information after our refactoring, so we can just return it directly
        Ok(account.clone())
    }

    pub fn expect_program_id(&self, expected_program_id: &Pubkey) -> AResult<()> {
        if self.program_id != *expected_program_id {
            Err(anyhow!(f!(
                "program_id not correct, expected: {:?}, actual: {:?}",
                expected_program_id,
                self.program_id
            )))
        } else {
            unit_ok!()
        }
    }
}
