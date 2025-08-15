use crate::arb::constant::known_pool_program::KnownPoolPrograms;
use solana_transaction_status::{UiInstruction, UiParsedInstruction, UiPartiallyDecodedInstruction};

pub fn is_meteora_damm_v2_ix(ix: &UiInstruction) -> Option<&UiPartiallyDecodedInstruction> {
    if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(decoded)) = ix {
        if decoded.program_id == KnownPoolPrograms::METEORA_DAMM_V2 && decoded.accounts.len() == 14 {
            return Some(decoded);
        }
    }
    None
}