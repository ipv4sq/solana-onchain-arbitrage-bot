use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::types::SwapInstruction;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::types::mint_pair::MintPair;
use crate::arb::global::constant::pool_program::PoolPrograms;
use crate::arb::convention::pool::interface::{InputAccountUtil, PoolConfigInit};
use crate::arb::convention::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::convention::pool::meteora_damm_v2::pool_config::MeteoraDammV2Config;
use crate::arb::convention::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::convention::pool::meteora_dlmm::pool_config::MeteoraDlmmPoolConfig;
use crate::arb::convention::pool::register::AnyPoolConfig::{MeteoraDammV2, MeteoraDlmm};
use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use crate::arb::convention::chain::Transaction;

lazy_static::lazy_static! {
    pub static ref RECOGNIZED_POOL_OWNER_PROGRAMS: HashSet<Pubkey> = {
        let mut set = HashSet::new();
        set.insert(PoolPrograms::METEORA_DLMM);
        set.insert(PoolPrograms::METEORA_DAMM_V2);
        set
    };
}

#[derive(Clone)]
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

    pub fn from_ix(ix: &Instruction, tx: &Transaction) -> Result<SwapInstruction> {
        let program_id_str = ix.program_id.to_string();
        match program_id_str.as_str() {
            x if x == PoolPrograms::METEORA_DLMM.to_string().as_str() => {
                use crate::arb::convention::pool::meteora_dlmm::input_data::MeteoraDlmmIxData;
                
                let accounts = MeteoraDlmmInputAccounts::restore_from(ix, tx)?;
                let trade_direction = accounts.clone().get_trade_direction();
                
                let data_hex = hex::encode(&ix.data);
                let ix_data = MeteoraDlmmIxData::load_ix_data(&data_hex);
                
                Ok(SwapInstruction {
                    dex_type: DexType::MeteoraDlmm,
                    pool_address: accounts.lb_pair.pubkey,
                    accounts: accounts.to_list().into_iter().cloned().collect(),
                    mints: MintPair(accounts.token_x_mint.pubkey, accounts.token_y_mint.pubkey),
                    amount_in: ix_data.amount_in,
                    amount_out: ix_data.min_amount_out,
                    trade_direction: (trade_direction.from, trade_direction.to),
                })
            }
            x if x == PoolPrograms::METEORA_DAMM_V2.to_string().as_str() => {
                use crate::arb::convention::pool::meteora_damm_v2::input_data::MeteoraDammV2InputData;
                
                let accounts = MeteoraDammV2InputAccount::restore_from(ix, tx)?;
                let trade_direction = accounts.clone().get_trade_direction();
                
                let data_hex = hex::encode(&ix.data);
                let ix_data = MeteoraDammV2InputData::load_from_hex(&data_hex)?;
                
                Ok(SwapInstruction {
                    dex_type: DexType::MeteoraDammV2,
                    pool_address: accounts.pool.pubkey,
                    accounts: accounts.to_list().into_iter().cloned().collect(),
                    mints: MintPair(accounts.token_a_mint.pubkey, accounts.token_b_mint.pubkey),
                    amount_in: ix_data.amount_in,
                    amount_out: ix_data.minimum_amount_out,
                    trade_direction: (trade_direction.from, trade_direction.to),
                })
            }
            _ => Err(anyhow::anyhow!("Unsupported program: {}", program_id_str)),
        }
    }
}
