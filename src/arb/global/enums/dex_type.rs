use crate::arb::global::constant::pool_program::PoolProgram;
use once_cell::sync::Lazy;
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveActiveEnum, EnumIter as SeaOrmEnumIter};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use strum_macros::Display;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq,
    SeaOrmEnumIter, DeriveActiveEnum,
    Serialize, Deserialize,
    Display
)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "PascalCase"
)]
#[strum(serialize_all = "PascalCase")]
pub enum DexType {
    RaydiumV4,
    RaydiumCp,
    RaydiumClmm,
    Pump,
    PumpAmm,
    MeteoraDlmm,
    MeteoraDamm,
    MeteoraDammV2,
    OrcaWhirlpool,
    Solfi,
    Vertigo,
    Unknown,
}

static PROGRAM_TO_DEX: Lazy<HashMap<Pubkey, DexType>> = Lazy::new(|| {
    [
        (PoolProgram::RAYDIUM_V4, DexType::RaydiumV4),
        (PoolProgram::RAYDIUM_CPMM, DexType::RaydiumCp),
        (PoolProgram::RAYDIUM_CLMM, DexType::RaydiumClmm),
        (PoolProgram::PUMP, DexType::Pump),
        (PoolProgram::PUMP_AMM, DexType::PumpAmm),
        (PoolProgram::METEORA_DLMM, DexType::MeteoraDlmm),
        (PoolProgram::METEORA_DAMM, DexType::MeteoraDamm),
        (PoolProgram::METEORA_DAMM_V2, DexType::MeteoraDammV2),
        (PoolProgram::WHIRLPOOL, DexType::OrcaWhirlpool),
        (PoolProgram::SOLFI, DexType::Solfi),
        (PoolProgram::VERTIGO, DexType::Vertigo),
    ]
    .into_iter()
    .collect()
});

impl DexType {
    pub fn determine_from(program_id: &Pubkey) -> Self {
        PROGRAM_TO_DEX.get(program_id).copied().unwrap_or(DexType::Unknown)
    }
}