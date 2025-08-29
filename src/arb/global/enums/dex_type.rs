use crate::arb::global::constant::pool_program::PoolProgram;
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveActiveEnum, EnumIter as SeaOrmEnumIter};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

// DEX types that can be identified in the transaction
#[derive(
    Debug, Clone, PartialEq, Eq, Copy, SeaOrmEnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum DexType {
    #[sea_orm(string_value = "RaydiumV4")]
    RaydiumV4,
    #[sea_orm(string_value = "RaydiumCp")]
    RaydiumCp,
    #[sea_orm(string_value = "RaydiumClmm")]
    RaydiumClmm,
    #[sea_orm(string_value = "Pump")]
    Pump,
    #[sea_orm(string_value = "PumpAmm")]
    PumpAmm,
    #[sea_orm(string_value = "MeteoraDlmm")]
    MeteoraDlmm,
    #[sea_orm(string_value = "MeteoraDamm")]
    MeteoraDamm,
    #[sea_orm(string_value = "MeteoraDammV2")]
    MeteoraDammV2,
    #[sea_orm(string_value = "OrcaWhirlpool")]
    OrcaWhirlpool,
    #[sea_orm(string_value = "Solfi")]
    Solfi,
    #[sea_orm(string_value = "Vertigo")]
    Vertigo,
    #[sea_orm(string_value = "Unknown")]
    Unknown,
}

impl DexType {
    // Determine DEX type from a program ID
    pub fn determine_from(program_id: &Pubkey) -> Self {
        let program_str = program_id.to_string();

        match program_str.as_str() {
            x if x == PoolProgram::RAYDIUM_V4.to_string() => DexType::RaydiumV4,
            x if x == PoolProgram::RAYDIUM_CPMM.to_string() => DexType::RaydiumCp,
            x if x == PoolProgram::RAYDIUM_CLMM.to_string() => DexType::RaydiumClmm,
            x if x == PoolProgram::PUMP.to_string() => DexType::Pump,
            x if x == PoolProgram::METEORA_DLMM.to_string() => DexType::MeteoraDlmm,
            x if x == PoolProgram::METEORA_DAMM.to_string() => DexType::MeteoraDamm,
            x if x == PoolProgram::METEORA_DAMM_V2.to_string() => DexType::MeteoraDammV2,
            x if x == PoolProgram::WHIRLPOOL.to_string() => DexType::OrcaWhirlpool,
            x if x == PoolProgram::SOLFI.to_string() => DexType::Solfi,
            x if x == PoolProgram::VERTIGO.to_string() => DexType::Vertigo,
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
            DexType::PumpAmm => "PumpAmm",
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

// in dex_type_def.rs

#[macro_export]
macro_rules! define_dex_types {
    (
        $( // name, db_string, program_id_const
            $name:ident => { db = $db:literal, program = $prog:path }
        ),* $(,)?
    ) => {
        use sea_orm::{DeriveActiveEnum, EnumIter as SeaOrmEnumIter};
        use serde::{Deserialize, Serialize};
        use phf::phf_map;
        use solana_program::pubkey::Pubkey;

        #[derive(
            Debug, Clone, PartialEq, Eq, Copy, SeaOrmEnumIter, DeriveActiveEnum, Serialize, Deserialize,
        )]
        #[sea_orm(rs_type = "String", db_type = "String(None)")]
        pub enum DexType {
            $(
                #[sea_orm(string_value = $db)]
                $name,
            )*
            #[sea_orm(string_value = "Unknown")]
            Unknown,
        }

        // 编译期常量表：program_id -> DexType
        static DEX_BY_PROGRAM: phf::Map<&'static str, DexType> = phf_map! {
            $(
                $prog => DexType::$name,
            )*
        };

        impl DexType {
            #[inline]
            pub fn determine_from(program_id: &Pubkey) -> Self {
                let s = program_id.to_string();
                DEX_BY_PROGRAM.get(s.as_str()).copied().unwrap_or(DexType::Unknown)
            }

            // 如需“转成 DB 字符串”，其实 SeaORM 已经管了；
            // 如果你仍想要：
            #[inline]
            pub fn as_db_str(&self) -> &'static str {
                match self {
                    $(
                        DexType::$name => $db,
                    )*
                    DexType::Unknown => "Unknown",
                }
            }
        }
    }
}
