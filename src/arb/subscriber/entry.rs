use crate::arb::constant::mint::MintPair;
use crate::arb::tx::constants::DexType;
use crate::arb::tx::tx_parser::{convert_to_smb_ix, filter_swap_inner_ix, parse_swap_inner_ix};
use anyhow::Result;
use itertools::Itertools;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiInnerInstructions, UiPartiallyDecodedInstruction,
};

type Transaction = EncodedConfirmedTransactionWithStatusMeta;
type Instruction = UiPartiallyDecodedInstruction;

pub fn on_mev_bot_transaction(
    tx: &Transaction,
    ix: &UiPartiallyDecodedInstruction,
    inner: &UiInnerInstructions,
) -> Result<()> {
    let smb_ix = convert_to_smb_ix(ix)?;
    let swap_instructions = filter_swap_inner_ix(inner);

    let mapped = swap_instructions
        .values()
        .into_iter()
        .filter_map(|x| parse_swap_inner_ix(x, tx).ok())
        .collect::<Vec<_>>();

    let _ = mapped
        .iter()
        .map(|x| record_pool_and_mints(&x.pool_address, x.dex_type, &x.mints));

    Ok(())
}

fn record_pool_and_mints(pool: &Pubkey, dex_type: DexType, mints: &MintPair) {
    todo!()
}
