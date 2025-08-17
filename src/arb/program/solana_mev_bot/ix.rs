use crate::arb::chain::instruction::{InnerInstructions, Instruction};
use crate::arb::program::solana_mev_bot::ix_input::SolanaMevBotIxInput;
use crate::arb::program::solana_mev_bot::ix_input_data::SolanaMevBotIxInputData;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use crate::arb::chain::Transaction;
use crate::constants::helpers::ToPubkey;
use crate::constants::mev_bot::SMB_ONCHAIN_PROGRAM_ID;

pub fn convert_to_smb_ix(ix: &Instruction) -> Result<SolanaMevBotIxInput> {
    let data = SolanaMevBotIxInputData::from_bytes(&ix.data)?;
    let accounts = ix.accounts.iter().map(|acc| acc.pubkey).collect();

    Ok(SolanaMevBotIxInput {
        program_id: ix.program_id,
        accounts,
        data,
    })
}

pub struct ProfitabilityStatement {
    pub mint: Pubkey,
    pub amount: u64,
}

pub fn is_mev_box_ix_profitable(
    ix: &Instruction,
    inners: &InnerInstructions,
) -> Result<ProfitabilityStatement> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::global::rpc::fetch_tx;
    use crate::arb::program::solana_mev_bot::ix::extract_mev_instruction;

    #[tokio::test]

    async fn test_is_mev_box_ix_profitable() {
        let tx_hash = "3mDkuLRaZRuGDcHon9JFGikkb7YQnc8Ph4NBjUG1vrbWLpCDvgMbHMDFycvtvwQv6BU2aF6wQbmQjdVNzHRGTQKs";
        let tx = fetch_tx(tx_hash).await.unwrap();
        let (ix, inner) = extract_mev_instruction(&tx).unwrap();
        let result = is_mev_box_ix_profitable(&ix, &inner).unwrap();
        /*
        Copied from solscan, for claude code to create test.
        1. swap 7.107544925 wsol -> 1,684,417.981584314 meme coin
        2. swap 1,684,417.981584314 wsol -> 7.343898162 wsol
        result  +0.236353237 sol after this arbitrage
         */
        todo!()
    }
}

pub fn extract_mev_instruction(tx: &Transaction) -> Option<(&Instruction, &InnerInstructions)> {
    tx.extract_ix_and_inners(|program_id| *program_id == SMB_ONCHAIN_PROGRAM_ID.to_pubkey())
}