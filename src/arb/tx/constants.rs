use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;
use crate::arb::constant::known_pool_program::KnownPoolPrograms;
use crate::constants::helpers::ToPubkey;



// DEX types that can be identified in the transaction
#[derive(Debug, Clone, PartialEq)]
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
            KnownPoolPrograms::RAYDIUM_V4 => DexType::RaydiumV4,
            KnownPoolPrograms::RAYDIUM_CPMM => DexType::RaydiumCp,
            KnownPoolPrograms::RAYDIUM_CLMM => DexType::RaydiumClmm,
            KnownPoolPrograms::PUMP => DexType::Pump,
            KnownPoolPrograms::METEORA_DLMM => DexType::MeteoraDlmm,
            KnownPoolPrograms::METEORA_DAMM => DexType::MeteoraDamm,
            KnownPoolPrograms::METEORA_DAMM_V2 => DexType::MeteoraDammV2,
            KnownPoolPrograms::WHIRLPOOL => DexType::OrcaWhirlpool,
            KnownPoolPrograms::SOLFI => DexType::Solfi,
            KnownPoolPrograms::VERTIGO => DexType::Vertigo,
            _ => DexType::Unknown,
        }
    }
}
