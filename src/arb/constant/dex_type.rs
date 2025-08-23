use solana_program::pubkey::Pubkey;
use crate::arb::constant::pool_owner::PoolOwnerPrograms;

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
    
    // Convert from database string representation
    pub fn from_db_string(s: &str) -> Self {
        match s {
            "RaydiumV4" => DexType::RaydiumV4,
            "RaydiumCp" => DexType::RaydiumCp,
            "RaydiumClmm" => DexType::RaydiumClmm,
            "Pump" => DexType::Pump,
            "MeteoraDlmm" => DexType::MeteoraDlmm,
            "MeteoraDamm" => DexType::MeteoraDamm,
            "MeteoraDammV2" => DexType::MeteoraDammV2,
            "OrcaWhirlpool" => DexType::OrcaWhirlpool,
            "Solfi" => DexType::Solfi,
            "Vertigo" => DexType::Vertigo,
            _ => DexType::Unknown,
        }
    }
    
    // Convert to database string representation (matches Debug format)
    pub fn to_db_string(&self) -> &'static str {
        match self {
            DexType::RaydiumV4 => "RaydiumV4",
            DexType::RaydiumCp => "RaydiumCp",
            DexType::RaydiumClmm => "RaydiumClmm",
            DexType::Pump => "Pump",
            DexType::MeteoraDlmm => "MeteoraDlmm",
            DexType::MeteoraDamm => "MeteoraDamm",
            DexType::MeteoraDammV2 => "MeteoraDammV2",
            DexType::OrcaWhirlpool => "OrcaWhirlpool",
            DexType::Solfi => "Solfi",
            DexType::Vertigo => "Vertigo",
            DexType::Unknown => "Unknown",
        }
    }
}