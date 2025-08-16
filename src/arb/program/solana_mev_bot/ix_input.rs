use solana_program::pubkey::Pubkey;
use crate::arb::program::solana_mev_bot::ix_input_data::SolanaMevBotIxInputData;

#[derive(Debug)]
pub struct SolanaMevBotIxInput {
    pub program_id: Pubkey,
    pub accounts: Vec<Pubkey>,
    pub data: SolanaMevBotIxInputData,
}