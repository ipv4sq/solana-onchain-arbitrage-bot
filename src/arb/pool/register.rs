use crate::arb::chain::data::Transaction;
use crate::arb::chain::data::instruction::Instruction;
use crate::arb::chain::types::SwapInstruction;
use crate::arb::constant::dex_type::DexType;
use crate::arb::constant::mint::MintPair;
use crate::arb::constant::pool_owner::PoolOwnerPrograms;
use crate::arb::pool::interface::{InputAccountUtil, PoolConfigInit};
use crate::arb::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::pool::meteora_damm_v2::pool_config::MeteoraDammV2Config;
use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::pool::meteora_dlmm::pool_config::MeteoraDlmmPoolConfig;
use crate::arb::pool::register::AnyPoolConfig::{MeteoraDammV2, MeteoraDlmm};
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;

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
                Ok(MeteoraDlmm(c))
            }
            DexType::MeteoraDammV2 => {
                let config = MeteoraDammV2Config::from_address(&pool).await?;
                Ok(MeteoraDammV2(config))
            }
            _ => Ok(AnyPoolConfig::Unsupported),
        }
    }

    pub fn from_ix(
        ix: &Instruction,
        tx: &Transaction,
    ) -> Result<SwapInstruction> {
        let program_id_str = ix.program_id.to_string();
        match program_id_str.as_str() {
            PoolOwnerPrograms::METEORA_DLMM => {
                let accounts = MeteoraDlmmInputAccounts::restore_from(ix, tx)?;
                Ok(SwapInstruction {
                    dex_type: DexType::MeteoraDlmm,
                    pool_address: accounts.lb_pair.pubkey,
                    accounts: accounts.to_list().into_iter().cloned().collect(),
                    mints: MintPair(accounts.token_x_mint.pubkey, accounts.token_y_mint.pubkey),
                })
            }
            PoolOwnerPrograms::METEORA_DAMM_V2 => {
                let accounts = MeteoraDammV2InputAccount::restore_from(ix, tx)?;
                Ok(SwapInstruction {
                    dex_type: DexType::MeteoraDammV2,
                    pool_address: accounts.pool.pubkey,
                    accounts: accounts.to_list().into_iter().cloned().collect(),
                    mints: MintPair(accounts.token_a_mint.pubkey, accounts.token_b_mint.pubkey),
                })
            }
            _ => Err(anyhow::anyhow!("Unsupported program: {}", program_id_str)),
        }
    }
}
