use crate::arb::convention::pool::interface::{PoolConfig, PoolConfigInit, PoolDataLoader};
use crate::arb::convention::pool::pump_amm::pool_data::PumpAmmPoolData;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

pub type PumpAmmPoolConfig = PoolConfig<PumpAmmPoolData>;

impl PoolConfigInit<PumpAmmPoolData> for PumpAmmPoolConfig {
    fn from_pool_data(
        pool: &Pubkey,
        account_data: PumpAmmPoolData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;
        Ok(PumpAmmPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.pair().minor_mint()?,
        })
    }
}
