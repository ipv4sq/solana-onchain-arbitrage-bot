use crate::arb::convention::pool::interface::{PoolConfig, PoolConfigInit, PoolDataLoader};
use crate::arb::convention::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::convention::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

pub const DLMM_EVENT_AUTHORITY: &str = "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6";

pub type MeteoraDlmmPoolConfig = PoolConfig<MeteoraDlmmPoolData>;

impl PoolConfigInit<MeteoraDlmmPoolData> for MeteoraDlmmPoolConfig {
    fn from_pool_data(
        pool: &Pubkey,
        account_data: MeteoraDlmmPoolData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(MeteoraDlmmPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.pair().minor_mint()?,
        })
    }
}

impl MeteoraDlmmPoolConfig {
    pub fn build_accounts_with_amount(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_in: u64,
    ) -> Result<MeteoraDlmmInputAccounts> {
        todo!()
    }
}
