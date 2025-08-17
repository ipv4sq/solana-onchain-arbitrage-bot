use crate::arb::program::solana_mev_bot::ix_input::SolanaMevBotIxInput;
use crate::arb::program::solana_mev_bot::ix_input_data::SolanaMevBotIxInputData;
use crate::arb::chain::data::instruction::Instruction;
use anyhow::Result;

pub fn convert_to_smb_ix(ix: &Instruction) -> Result<SolanaMevBotIxInput> {
    let data = SolanaMevBotIxInputData::from_bytes(&ix.data)?;
    let accounts = ix.accounts.iter().map(|acc| acc.pubkey).collect();

    Ok(SolanaMevBotIxInput {
        program_id: ix.program_id,
        accounts,
        data,
    })
}


