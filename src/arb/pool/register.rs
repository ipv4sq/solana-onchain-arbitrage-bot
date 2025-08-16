use crate::arb::constant::dex_type::DexType;
use crate::arb::constant::pool_owner::PoolOwnerPrograms;
use crate::arb::pool::interface::{InputAccountUtil, PoolConfigInit};
use crate::arb::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::pool::meteora_damm_v2::pool_config::MeteoraDammV2Config;
use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::pool::meteora_dlmm::pool_config::MeteoraDlmmPoolConfig;
use crate::arb::pool::register::AnyPoolConfig::{MeteoraDammV2, MeteoraDlmm};
use anyhow::Result;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiPartiallyDecodedInstruction,
};
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
        ix: &UiPartiallyDecodedInstruction,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<(DexType, Vec<AccountMeta>)> {
        let result = match ix.program_id.as_str() {
            PoolOwnerPrograms::METEORA_DLMM => (
                DexType::MeteoraDlmm,
                MeteoraDlmmInputAccounts::restore_from(ix, tx)?.to_list_cloned(),
            ),
            PoolOwnerPrograms::METEORA_DAMM_V2 => (
                DexType::MeteoraDammV2,
                MeteoraDammV2InputAccount::restore_from(ix, tx)?.to_list_cloned(),
            ),
            _ => return Err(anyhow::anyhow!("Unknown pool owner program")),
        };
        Ok(result)
    }
}
