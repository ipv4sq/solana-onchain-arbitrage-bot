use crate::arb::global::enums::dex_type::DexType;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct SwapInstruction {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
}
