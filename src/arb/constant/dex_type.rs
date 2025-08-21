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
            "raydium_v4" => DexType::RaydiumV4,
            "raydium_cp" => DexType::RaydiumCp,
            "raydium_clmm" => DexType::RaydiumClmm,
            "pump" => DexType::Pump,
            "meteora_dlmm" => DexType::MeteoraDlmm,
            "meteora_damm" => DexType::MeteoraDamm,
            "meteora_damm_v2" => DexType::MeteoraDammV2,
            "orca_whirlpool" => DexType::OrcaWhirlpool,
            "solfi" => DexType::Solfi,
            "vertigo" => DexType::Vertigo,
            _ => DexType::Unknown,
        }
    }
    
    // Convert to database string representation
    pub fn to_db_string(&self) -> &'static str {
        match self {
            DexType::RaydiumV4 => "raydium_v4",
            DexType::RaydiumCp => "raydium_cp",
            DexType::RaydiumClmm => "raydium_clmm",
            DexType::Pump => "pump",
            DexType::MeteoraDlmm => "meteora_dlmm",
            DexType::MeteoraDamm => "meteora_damm",
            DexType::MeteoraDammV2 => "meteora_damm_v2",
            DexType::OrcaWhirlpool => "orca_whirlpool",
            DexType::Solfi => "solfi",
            DexType::Vertigo => "vertigo",
            DexType::Unknown => "unknown",
        }
    }
}