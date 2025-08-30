use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::types::SwapInstruction;
use crate::arb::convention::chain::{AccountState, Transaction};
use crate::arb::dex::any_pool_config::AnyPoolConfig::{MeteoraDammV2, MeteoraDlmm, PumpAmm};
use crate::arb::dex::interface::{InputAccountUtil, PoolConfigInit, PoolDataLoader};
use crate::arb::dex::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::dex::meteora_damm_v2::legacy::pool_config::MeteoraDammV2Config;
use crate::arb::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::arb::dex::meteora_damm_v2::refined::MeteoraDammV2RefinedConfig;
use crate::arb::dex::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::dex::meteora_dlmm::legacy::pool_config::MeteoraDlmmPoolConfig;
use crate::arb::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::arb::dex::meteora_dlmm::refined::MeteoraDlmmRefinedConfig;
use crate::arb::dex::pump_amm::input_account::PumpAmmInputAccounts;
use crate::arb::dex::pump_amm::legacy::pool_config::PumpAmmPoolConfig;
use crate::arb::dex::pump_amm::refined::PumpAmmRefinedConfig;
use crate::arb::dex::refined_interface::RefinedPoolConfig;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::util::alias::{AResult, PoolAddress};
use crate::f;
use anyhow::{anyhow, Result};
use solana_program::pubkey::Pubkey;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum AnyPoolConfig {
    MeteoraDlmm(MeteoraDlmmRefinedConfig),
    MeteoraDammV2(MeteoraDammV2RefinedConfig),
    PumpAmm(PumpAmmRefinedConfig),
    Unsupported,
}

impl AnyPoolConfig {
    pub fn recognized(program: &Pubkey) -> bool {
        [
            PoolProgram::METEORA_DLMM,
            PoolProgram::PUMP_AMM,
            PoolProgram::METEORA_DAMM_V2,
        ]
        .contains(&program)
    }
    pub fn dex_type(&self) -> DexType {
        match self {
            MeteoraDlmm(_) => DexType::MeteoraDlmm,
            MeteoraDammV2(_) => DexType::MeteoraDammV2,
            PumpAmm(_) => DexType::PumpAmm,
            AnyPoolConfig::Unsupported => DexType::Unknown,
        }
    }
}

impl AnyPoolConfig {
    pub fn from_basis(
        pool_address: PoolAddress,
        dex_type: DexType,
        data: &[u8],
    ) -> AResult<AnyPoolConfig> {
        let r = match dex_type {
            DexType::MeteoraDlmm => MeteoraDlmm(MeteoraDlmmRefinedConfig::from_data(
                pool_address,
                dex_type,
                data,
            )?),
            DexType::MeteoraDammV2 => MeteoraDammV2(MeteoraDammV2RefinedConfig::from_data(
                pool_address,
                dex_type,
                data,
            )?),
            DexType::PumpAmm => PumpAmm(PumpAmmRefinedConfig::from_data(
                pool_address,
                dex_type,
                data,
            )?),
            _ => AnyPoolConfig::Unsupported,
        };
        Ok(r)
    }

    pub fn from_owner_and_data(
        pool_address: &PoolAddress,
        owner: &Pubkey,
        data: &[u8],
    ) -> AResult<AnyPoolConfig> {
        let dex_type = DexType::determine_from(owner);
        Self::from_basis(*pool_address, dex_type, data)
    }

    pub async fn from(pool_address: &Pubkey) -> Result<AnyPoolConfig> {
        let account = rpc_client().get_account(pool_address).await?;
        let dex_type = DexType::determine_from(&account.owner);
        Self::from_basis(*pool_address, dex_type, &account.data)
    }

    pub fn from_ix(ix: &Instruction) -> Result<SwapInstruction> {
        let program_id = ix.program_id;
        let dex_type = DexType::determine_from(&program_id);
        let (dex, address) = match dex_type {
            DexType::MeteoraDlmm => MeteoraDlmmRefinedConfig::extract_pool_from(ix),
            DexType::MeteoraDammV2 => MeteoraDammV2RefinedConfig::extract_pool_from(ix),
            DexType::PumpAmm => PumpAmmRefinedConfig::extract_pool_from(ix),
            _ => Err(anyhow!(f!("Unsppord dex {}", dex_type))),
        }?;

        Ok(SwapInstruction {
            dex_type: dex,
            pool_address: address,
        })
    }
}
