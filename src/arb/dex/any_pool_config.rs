use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::types::SwapInstruction;
use crate::arb::dex::any_pool_config::AnyPoolConfig::{
    MeteoraDammV2, MeteoraDlmm, PumpAmm, RaydiumCpmm,
};
use crate::arb::dex::interface::PoolConfig;
use crate::arb::dex::meteora_damm_v2::config::MeteoraDammV2Config;
use crate::arb::dex::meteora_dlmm::config::MeteoraDlmmConfig;
use crate::arb::dex::meteora_dlmm::price::price_calculator::DlmmQuote;
use crate::arb::dex::pump_amm::config::PumpAmmConfig;
use crate::arb::dex::raydium_cpmm::config::RaydiumCpmmConfig;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;
use crate::return_error;
use anyhow::Result;
use delegate::delegate;
use serde_json::Value;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum AnyPoolConfig {
    MeteoraDlmm(MeteoraDlmmConfig),
    MeteoraDammV2(MeteoraDammV2Config),
    PumpAmm(PumpAmmConfig),
    RaydiumCpmm(RaydiumCpmmConfig),
}

impl AnyPoolConfig {
    pub fn new(
        pool_address: PoolAddress,
        dex_type: DexType,
        data: &[u8],
    ) -> AResult<AnyPoolConfig> {
        let r = match dex_type {
            DexType::MeteoraDlmm => {
                MeteoraDlmm(MeteoraDlmmConfig::from_data(pool_address, dex_type, data)?)
            }
            DexType::MeteoraDammV2 => MeteoraDammV2(MeteoraDammV2Config::from_data(
                pool_address,
                dex_type,
                data,
            )?),
            DexType::PumpAmm => PumpAmm(PumpAmmConfig::from_data(pool_address, dex_type, data)?),
            DexType::RaydiumCpmm => {
                RaydiumCpmm(RaydiumCpmmConfig::from_data(pool_address, dex_type, data)?)
            }
            _ => return_error!("unsupported dex type {:?}", dex_type),
        };
        Ok(r)
    }

    pub fn parse_swap_from_ix(ix: &Instruction) -> Result<SwapInstruction> {
        let program_id = ix.program_id;
        let dex_type = DexType::determine_from(&program_id);
        let (dex, address) = match dex_type {
            DexType::MeteoraDlmm => MeteoraDlmmConfig::pase_swap_from_ix(ix),
            DexType::MeteoraDammV2 => MeteoraDammV2Config::pase_swap_from_ix(ix),
            DexType::PumpAmm => PumpAmmConfig::pase_swap_from_ix(ix),
            DexType::RaydiumCpmm => RaydiumCpmmConfig::pase_swap_from_ix(ix),
            _ => return_error!("Unsupported dex {}", dex_type),
        }?;

        Ok(SwapInstruction {
            dex_type: dex,
            pool_address: address,
        })
    }
}

impl AnyPoolConfig {
    delegate! {
        to match self {
            MeteoraDlmm(a) => a,
            MeteoraDammV2(b) => b,
            PumpAmm(c) => c,
            RaydiumCpmm(d) => d,
        } {
            pub async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>>;
            pub fn pool(&self) -> PoolAddress;
            pub fn base_mint(&self) -> MintAddress;
            pub fn quote_mint(&self) -> MintAddress;
            pub fn dex_type(&self) -> DexType;
            pub fn pool_data_json(&self) -> Value;
            pub async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote>;
            pub async fn get_amount_out(&self,input_amount: u64,from_mint: &MintAddress,to_mint: &MintAddress,) -> AResult<u64>;
            pub fn mint_pair(&self) -> MintPair;
        }
    }
}
