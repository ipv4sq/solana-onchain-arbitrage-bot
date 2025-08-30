use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{PoolConfig, PoolDataLoader};
use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

pub struct PoolBase<Data: PoolDataLoader> {
    pub pool_address: PoolAddress,
    pub base_mint: MintAddress,
    pub quote_mint: MintAddress,
    pub dex_type: DexType,
    pub pool_data: Data,
}

pub trait RefinedPoolConfig {
    fn extract_pool_from(ix: Instruction) -> AResult<(DexType, PoolAddress)>;

    fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> Vec<AccountMeta>;

    fn refresh_pool_data(data: &[u8]);
}

type PumpAmmRefinedConfig = PoolBase<PumpAmmPoolData>;

impl RefinedPoolConfig for PumpAmmRefinedConfig {
    fn extract_pool_from(ix: Instruction) -> AResult<(DexType, PoolAddress)> {
        todo!()
    }

    fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> Vec<AccountMeta> {
        todo!()
    }

    fn refresh_pool_data(data: &[u8]) {
        todo!()
    }
}
