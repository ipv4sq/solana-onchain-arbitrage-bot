use anyhow::Result;
use solana_transaction_status::UiPartiallyDecodedInstruction;
use crate::arb::program::solana_mev_bot::ix_input::SolanaMevBotIxInput;
use crate::arb::program::solana_mev_bot::ix_input_data::SolanaMevBotIxInputData;
use crate::constants::helpers::ToPubkey;

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