use crate::arb::constant::known_pool_program::KnownPoolPrograms;
use solana_transaction_status::{UiInstruction, UiParsedInstruction, UiPartiallyDecodedInstruction};

pub fn is_meteora_damm_v2_ix(ix: &UiInstruction) -> Option<&UiPartiallyDecodedInstruction> {
    // METEORA_DAMM_V2 swap instructions have exactly 14 accounts
    is_program_ix(ix, KnownPoolPrograms::METEORA_DAMM_V2, Some(14))
        .filter(|decoded| decoded.accounts.len() == 14) // Ensure exactly 14, not just minimum
}

// Generic function to check if an instruction matches a specific program with optional account count check
pub fn is_program_ix<'a>(
    ix: &'a UiInstruction,
    program_id: &str,
    min_accounts: Option<usize>,
) -> Option<&'a UiPartiallyDecodedInstruction> {
    if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(decoded)) = ix {
        if decoded.program_id == program_id {
            if let Some(min) = min_accounts {
                if decoded.accounts.len() >= min {
                    return Some(decoded);
                }
            } else {
                return Some(decoded);
            }
        }
    }
    None
}