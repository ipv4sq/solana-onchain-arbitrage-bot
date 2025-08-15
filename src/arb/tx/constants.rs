use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;
use crate::arb::constant::known_pool_program::PoolOwnerPrograms;
use crate::constants::helpers::ToPubkey;



// DEX types that can be identified in the transaction
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum DexType {
    RaydiumV4,
    RaydiumCp,
    RaydiumClmm,
    Pump,
    MeteoraDlmm,
    MeteoraDamm,
    MeteoraDammV2,
    OrcaWhirlpool,
    Solfi,
    Vertigo,
    Unknown,
}

impl DexType {
    // Determine DEX type from a program ID
    pub fn determine_from(program_id: &Pubkey) -> Self {
        let program_str = program_id.to_string();

        match program_str.as_str() {
            PoolOwnerPrograms::RAYDIUM_V4 => DexType::RaydiumV4,
            PoolOwnerPrograms::RAYDIUM_CPMM => DexType::RaydiumCp,
            PoolOwnerPrograms::RAYDIUM_CLMM => DexType::RaydiumClmm,
            PoolOwnerPrograms::PUMP => DexType::Pump,
            PoolOwnerPrograms::METEORA_DLMM => DexType::MeteoraDlmm,
            PoolOwnerPrograms::METEORA_DAMM => DexType::MeteoraDamm,
            PoolOwnerPrograms::METEORA_DAMM_V2 => DexType::MeteoraDammV2,
            PoolOwnerPrograms::WHIRLPOOL => DexType::OrcaWhirlpool,
            PoolOwnerPrograms::SOLFI => DexType::Solfi,
            PoolOwnerPrograms::VERTIGO => DexType::Vertigo,
            _ => DexType::Unknown,
        }
    }
}
