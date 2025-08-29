use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::structs::mint_pair::MintPair;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct SwapInstruction {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
}
