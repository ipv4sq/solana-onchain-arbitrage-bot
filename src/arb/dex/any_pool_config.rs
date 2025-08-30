use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::types::SwapInstruction;
use crate::arb::convention::chain::{AccountState, Transaction};
use crate::arb::dex::any_pool_config::AnyPoolConfig::{MeteoraDammV2, MeteoraDlmm, PumpAmm};
use crate::arb::dex::interface::{InputAccountUtil, PoolConfigInit, PoolDataLoader};
use crate::arb::dex::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::dex::meteora_damm_v2::pool_config::MeteoraDammV2Config;
use crate::arb::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::arb::dex::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::dex::meteora_dlmm::pool_config::MeteoraDlmmPoolConfig;
use crate::arb::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::arb::dex::pump_amm::input_account::PumpAmmInputAccounts;
use crate::arb::dex::pump_amm::input_data::PumpAmmIxData;
use crate::arb::dex::pump_amm::pool_config::PumpAmmPoolConfig;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::global::state::rpc::rpc_client;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum AnyPoolConfig {
    MeteoraDlmm(MeteoraDlmmPoolConfig),
    MeteoraDammV2(MeteoraDammV2Config),
    PumpAmm(PumpAmmPoolConfig),
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
    pub async fn from_account_update(
        update: &AccountState,
        desired_mint: &Pubkey,
    ) -> Result<AnyPoolConfig> {
        let dex_type = DexType::determine_from(&update.owner);
        let c = match dex_type {
            DexType::MeteoraDlmm => {
                let data = MeteoraDlmmPoolData::load_data(&update.data)?;
                let config =
                    MeteoraDlmmPoolConfig::from_pool_data(&update.pubkey, data, *desired_mint)?;
                MeteoraDlmm(config)
            }
            DexType::MeteoraDammV2 => {
                let data = MeteoraDammV2PoolData::load_data(&update.data)?;
                let config =
                    MeteoraDammV2Config::from_pool_data(&update.pubkey, data, *desired_mint)?;
                MeteoraDammV2(config)
            }
            _ => AnyPoolConfig::Unsupported,
        };
        Ok(c)
    }

    pub async fn from(pool_address: &Pubkey) -> Result<AnyPoolConfig> {
        let account = rpc_client().get_account(pool_address).await?;
        let dex_type = DexType::determine_from(&account.owner);
        Self::from_address(pool_address, dex_type).await
    }

    pub async fn from_address(pool: &Pubkey, dex_type: DexType) -> Result<AnyPoolConfig> {
        match dex_type {
            DexType::MeteoraDlmm => {
                let c = MeteoraDlmmPoolConfig::from_address(&pool).await?;
                Ok(MeteoraDlmm(c))
            }
            DexType::MeteoraDammV2 => {
                let config = MeteoraDammV2Config::from_address(&pool).await?;
                Ok(MeteoraDammV2(config))
            }
            DexType::PumpAmm => {
                let c = PumpAmmPoolConfig::from_address(&pool).await?;
                Ok(PumpAmm(c))
            }
            _ => Ok(AnyPoolConfig::Unsupported),
        }
    }

    pub fn from_ix(ix: &Instruction, tx: &Transaction) -> Result<SwapInstruction> {
        let program_id_str = ix.program_id.to_string();
        match program_id_str.as_str() {
            x if x == PoolProgram::METEORA_DLMM.to_string().as_str() => {
                let accounts = MeteoraDlmmInputAccounts::restore_from(ix, tx)?;

                Ok(SwapInstruction {
                    dex_type: DexType::MeteoraDlmm,
                    pool_address: accounts.lb_pair.pubkey,
                })
            }
            x if x == PoolProgram::METEORA_DAMM_V2.to_string().as_str() => {
                use crate::arb::dex::meteora_damm_v2::input_data::MeteoraDammV2InputData;

                let accounts = MeteoraDammV2InputAccount::restore_from(ix, tx)?;

                let data_hex = hex::encode(&ix.data);
                let ix_data = MeteoraDammV2InputData::load_from_hex(&data_hex)?;

                Ok(SwapInstruction {
                    dex_type: DexType::MeteoraDammV2,
                    pool_address: accounts.pool.pubkey,
                })
            }
            x if x == PoolProgram::PUMP_AMM.to_string().as_str() => {
                let accounts = PumpAmmInputAccounts::restore_from(ix, tx)?;

                let data_hex = hex::encode(&ix.data);
                Ok(SwapInstruction {
                    dex_type: DexType::MeteoraDammV2,
                    pool_address: accounts.pool.pubkey,
                })
            }
            _ => Err(anyhow::anyhow!("Unsupported program: {}", program_id_str)),
        }
    }
}
