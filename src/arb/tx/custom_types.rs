use crate::arb::constant::dex_type::DexType;
use crate::arb::constant::mint::MintPair;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct SwapInstruction {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub mints: MintPair,
}

#[derive(Debug, Clone)]
pub struct LitePool {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
    pub mints: MintPair,
}
