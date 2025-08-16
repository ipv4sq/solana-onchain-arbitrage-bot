use crate::arb::constant::pool_owner::PoolOwnerPrograms;
use solana_transaction_status::{
    UiInstruction, UiParsedInstruction, UiPartiallyDecodedInstruction,
};
use crate::arb::program::solana_mev_bot::ix_input_data::SolanaMevBotIxInputData;
use anyhow::Result;
use crate::arb::program::solana_mev_bot::ix_input::SolanaMevBotIxInput;
use crate::constants::helpers::ToPubkey;

pub fn is_meteora_damm_v2_swap(ix: &UiInstruction) -> Option<&UiPartiallyDecodedInstruction> {
    // METEORA_DAMM_V2 swap instructions have exactly 14 accounts
    is_program_ix(ix, PoolOwnerPrograms::METEORA_DAMM_V2, Some(14))
        .filter(|decoded| decoded.accounts.len() == 14)
}
pub fn is_meteora_dlmm_swap(ix: &UiInstruction) -> Option<&UiPartiallyDecodedInstruction> {
    is_program_ix(ix, PoolOwnerPrograms::METEORA_DLMM, Some(14))
        .filter(|decoded| decoded.accounts.len() > 14)
}

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

pub fn convert_to_smb_ix(ix: &UiPartiallyDecodedInstruction) -> Result<SolanaMevBotIxInput> {
    let data_bytes = bs58::decode(&ix.data)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Failed to decode instruction data: {}", e))?;
    let data = SolanaMevBotIxInputData::from_bytes(&data_bytes)?;
    let accounts = ix.accounts.iter().map(|acc| acc.to_pubkey()).collect();

    Ok(SolanaMevBotIxInput {
        program_id: ix.program_id.to_pubkey(),
        accounts,
        data,
    })
}