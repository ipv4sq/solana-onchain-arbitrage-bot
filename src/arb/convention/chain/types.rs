use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::types::mint_pair::MintPair;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct SwapInstruction {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub mints: MintPair,
    pub amount_in: u64,
    pub amount_out: u64,
    pub trade_direction: (Pubkey, Pubkey),
}

#[derive(Debug, Clone)]
pub struct LitePool {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
    pub mints: MintPair,
}
