use crate::arb::constant::client::rpc_client;
use crate::arb::constant::known_pool_program::METEORA_DLMM_PROGRAM;
use crate::arb::constant::mint::MintPair;
use crate::arb::pool::interface::{PoolConfig, PoolConfigInit, PoolDataLoader};
use crate::arb::pool::meteora_dlmm::bin_array;
use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::constants::addresses::SPL_TOKEN_KEY;
use crate::constants::helpers::ToAccountMeta;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

pub const DLMM_EVENT_AUTHORITY: &str = "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6";

pub type MeteoraDlmmPoolConfig = PoolConfig<MeteoraDlmmPoolData>;

impl PoolConfigInit<MeteoraDlmmPoolData, MeteoraDlmmInputAccounts> for MeteoraDlmmPoolConfig {
    fn build_from_pool_data(
        pool: &Pubkey,
        account_data: MeteoraDlmmPoolData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(MeteoraDlmmPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
        })
    }

    fn build_accounts(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_in: Option<u64>,
        amount_out: Option<u64>,
    ) -> Result<MeteoraDlmmInputAccounts> {
        // Default to small swap bin arrays
        // For actual swaps, should call build_accounts_with_amount
        self.build_accounts_with_amount(payer, input_mint, output_mint, 0)
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
