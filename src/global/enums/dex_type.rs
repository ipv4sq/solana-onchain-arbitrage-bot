use crate::global::constant::pool_program::PoolProgram;
use once_cell::sync::Lazy;
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveActiveEnum, EnumIter as SeaOrmEnumIter};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::Display;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    SeaOrmEnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    Display,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "PascalCase"
)]
#[strum(serialize_all = "PascalCase")]
pub enum DexType {
    RaydiumV4,
    RaydiumCpmm,
    RaydiumClmm,
    Pump,
    PumpAmm,
    MeteoraDlmm,
    MeteoraDamm,
    MeteoraDammV2,
    Whirlpool,
    Solfi,
    Vertigo,
    Unknown,
}

static PROGRAM_TO_DEX: Lazy<HashMap<Pubkey, DexType>> = Lazy::new(|| {
    DexType::iter()
        .filter_map(|dex| {
            let program = match dex {
                DexType::RaydiumV4 => PoolProgram::RAYDIUM_V4,
                DexType::RaydiumCpmm => PoolProgram::RAYDIUM_CPMM,
                DexType::RaydiumClmm => PoolProgram::RAYDIUM_CLMM,
                DexType::Pump => PoolProgram::PUMP,
                DexType::PumpAmm => PoolProgram::PUMP_AMM,
                DexType::MeteoraDlmm => PoolProgram::METEORA_DLMM,
                DexType::MeteoraDamm => PoolProgram::METEORA_DAMM,
                DexType::MeteoraDammV2 => PoolProgram::METEORA_DAMM_V2,
                DexType::Whirlpool => PoolProgram::WHIRLPOOL,
                DexType::Solfi => PoolProgram::SOLFI,
                DexType::Vertigo => PoolProgram::VERTIGO,
                DexType::Unknown => return None,
            };
            Some((program, dex))
        })
        .collect()
});

impl DexType {
    pub fn determine_from(program_id: &Pubkey) -> Self {
        PROGRAM_TO_DEX
            .get(program_id)
            .copied()
            .unwrap_or(DexType::Unknown)
    }

    pub fn owner_program_id(&self) -> Pubkey {
        PROGRAM_TO_DEX
            .iter()
            .find_map(|(program, dex)| (*dex == *self).then_some(*program))
            .unwrap_or_default()
    }
}
