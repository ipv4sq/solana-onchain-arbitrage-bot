use crate::arb::pool::interface::PoolConfigInit;
use crate::arb::pool::meteora_damm_v2::pool_config::MeteoraDammV2Config;
use crate::arb::pool::meteora_dlmm::pool_config::MeteoraDlmmPoolConfig;
use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use crate::arb::constant::dex_type::DexType;

pub struct PoolOwnerPrograms;

impl PoolOwnerPrograms {
    pub const RAYDIUM_V4: &'static str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
    pub const RAYDIUM_CPMM: &'static str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
    pub const RAYDIUM_CLMM: &'static str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
    pub const PUMP: &'static str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
    pub const METEORA_DLMM: &'static str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";
    pub const METEORA_DAMM: &'static str = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB";
    pub const METEORA_DAMM_V2: &'static str = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG";
    pub const WHIRLPOOL: &'static str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
    pub const SOLFI: &'static str = "SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe";
    pub const VERTIGO: &'static str = "vrTGoBuy5rYSxAfV3jaRJWHH6nN9WK4NRExGxsk1bCJ";
}

lazy_static! {
    pub static ref RAYDIUM_V4_PROGRAM: Pubkey = PoolOwnerPrograms::RAYDIUM_V4.to_pubkey();
    pub static ref RAYDIUM_CPMM_PROGRAM: Pubkey = PoolOwnerPrograms::RAYDIUM_CPMM.to_pubkey();
    pub static ref RAYDIUM_CLMM_PROGRAM: Pubkey = PoolOwnerPrograms::RAYDIUM_CLMM.to_pubkey();
    pub static ref PUMP_PROGRAM: Pubkey = PoolOwnerPrograms::PUMP.to_pubkey();
    pub static ref METEORA_DLMM_PROGRAM: Pubkey = PoolOwnerPrograms::METEORA_DLMM.to_pubkey();
    pub static ref METEORA_DAMM_PROGRAM: Pubkey = PoolOwnerPrograms::METEORA_DAMM.to_pubkey();
    pub static ref METEORA_DAMM_V2_PROGRAM: Pubkey = PoolOwnerPrograms::METEORA_DAMM_V2.to_pubkey();
    pub static ref WHIRLPOOL_PROGRAM: Pubkey = PoolOwnerPrograms::WHIRLPOOL.to_pubkey();
    pub static ref SOLFI_PROGRAM: Pubkey = PoolOwnerPrograms::SOLFI.to_pubkey();
    pub static ref VERTIGO_PROGRAM: Pubkey = PoolOwnerPrograms::VERTIGO.to_pubkey();
}

lazy_static::lazy_static! {
    pub static ref RECOGNIZED_POOL_OWNER_PROGRAMS: HashSet<String> = {
        let mut set = HashSet::new();
        set.insert(PoolOwnerPrograms::METEORA_DLMM.to_string());
        set.insert(PoolOwnerPrograms::METEORA_DAMM_V2.to_string());
        set
    };
}

pub enum AnyPoolConfig {
    MeteoraDlmm(MeteoraDlmmPoolConfig),
    MeteoraDammV2(MeteoraDammV2Config),
    Unsupported,
}

impl AnyPoolConfig {
    pub async fn from_address(pool: &Pubkey, dex_type: DexType) -> Result<AnyPoolConfig> {
        match dex_type {
            DexType::MeteoraDlmm => {
                let c = MeteoraDlmmPoolConfig::from_address(&pool).await?;
                Ok(AnyPoolConfig::MeteoraDlmm(c))
            }
            DexType::MeteoraDammV2 => {
                let config = MeteoraDammV2Config::from_address(&pool).await?;
                Ok(AnyPoolConfig::MeteoraDammV2(config))
            }
            _ => Ok(AnyPoolConfig::Unsupported),
        }
    }
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