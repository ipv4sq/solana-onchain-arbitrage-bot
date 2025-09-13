use crate::convention::chain::instruction::Instruction;
use crate::convention::chain::types::SwapInstruction;
use crate::dex::any_pool_config::AnyPoolConfig::{
    MeteoraDammV2, MeteoraDlmm, PumpAmm, RaydiumClmm, RaydiumCpmm, Whirlpool,
};
use crate::dex::interface::PoolConfig;
use crate::dex::meteora_damm_v2::config::MeteoraDammV2Config;
use crate::dex::meteora_dlmm::config::MeteoraDlmmConfig;
use crate::dex::pump_amm::config::PumpAmmConfig;
use crate::dex::raydium_clmm::config::RaydiumClmmConfig;
use crate::dex::raydium_cpmm::config::RaydiumCpmmConfig;
use crate::dex::whirlpool::config::WhirlpoolConfig;
use crate::global::enums::dex_type::DexType;
use crate::global::state::account_balance_holder::get_balance_of_account;
use crate::pipeline::event_processor::token_balance::token_balance_processor::TokenAmount;
use crate::return_error;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::structs::mint_pair::MintPair;
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
    RaydiumClmm(RaydiumClmmConfig),
    Whirlpool(WhirlpoolConfig),
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
            DexType::RaydiumClmm => {
                RaydiumClmm(RaydiumClmmConfig::from_data(pool_address, dex_type, data)?)
            }
            DexType::Whirlpool => {
                Whirlpool(WhirlpoolConfig::from_data(pool_address, dex_type, data)?)
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
            DexType::RaydiumClmm => RaydiumClmmConfig::pase_swap_from_ix(ix),
            DexType::Whirlpool => WhirlpoolConfig::pase_swap_from_ix(ix),
            _ => return_error!("Unsupported dex {}", dex_type),
        }?;

        Ok(SwapInstruction {
            dex_type: dex,
            pool_address: address,
        })
    }
}

impl AnyPoolConfig {
    pub async fn get_reserves(&self) -> (Option<TokenAmount>, Option<TokenAmount>) {
        let base_reserve_addr = self.base_reserve_address();
        let quote_reserve_addr = self.quote_reserve_address();
        let base_mint = self.base_mint();
        let quote_mint = self.quote_mint();

        let (base_balance, quote_balance) = tokio::join!(
            get_balance_of_account(&base_reserve_addr, &base_mint),
            get_balance_of_account(&quote_reserve_addr, &quote_mint)
        );

        (base_balance, quote_balance)
    }

    delegate! {
        to match self {
            MeteoraDlmm(a) => a,
            MeteoraDammV2(b) => b,
            PumpAmm(c) => c,
            RaydiumCpmm(d) => d,
            RaydiumClmm(e) => e,
            Whirlpool(f) => f,
        } {
            pub async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>>;
            pub fn pool_address(&self) -> PoolAddress;
            pub fn base_mint(&self) -> MintAddress;
            pub fn quote_mint(&self) -> MintAddress;
            pub fn dex_type(&self) -> DexType;
            pub fn pool_data_json(&self) -> Value;
            pub async fn get_amount_out(&self,input_amount: u64,from_mint: &MintAddress,to_mint: &MintAddress,) -> AResult<u64>;
            pub fn mint_pair(&self) -> MintPair;
            pub fn base_reserve_address(&self) -> Pubkey;
            pub fn quote_reserve_address(&self) -> Pubkey;
        }
    }
}
