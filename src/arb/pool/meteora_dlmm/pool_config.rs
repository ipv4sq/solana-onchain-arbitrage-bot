use crate::arb::constant::known_pool_program::METEORA_DLMM_PROGRAM;
use crate::arb::pool::interface::{
    PoolAccountDataLoader, PoolConfig, PoolConfigInit,
};
use crate::constants::addresses::SPL_TOKEN_KEY;
use crate::constants::helpers::ToAccountMeta;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use crate::arb::pool::meteora_dlmm::account::MeteoraDlmmSwapAccounts;
use crate::arb::pool::meteora_dlmm::data::MeteoraDlmmAccountData;

const DLMM_EVENT_AUTHORITY: &str = "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6";

pub type MeteoraDlmmPoolConfig = PoolConfig<MeteoraDlmmAccountData>;

impl MeteoraDlmmPoolConfig {
    /// Build swap accounts with specific amount for accurate bin array calculation
    pub fn build_accounts_with_amount(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_in: u64,
    ) -> Result<MeteoraDlmmSwapAccounts> {
        // Determine swap direction
        let is_a_to_b = input_mint == &self.data.token_x_mint;

        // Calculate required bin arrays based on amount
        let bin_arrays = self.data.calculate_bin_arrays_for_swap(
            &self.pool,
            self.data.active_id,
            amount_in,
            is_a_to_b,
        );

        Ok(MeteoraDlmmSwapAccounts {
            lb_pair: self.pool.to_writable(),
            bin_array_bitmap_extension: METEORA_DLMM_PROGRAM.to_program(),
            reverse_x: self.data.reserve_x.to_writable(),
            reverse_y: self.data.reserve_y.to_writable(),
            user_token_in: Self::ata(payer, input_mint, &*SPL_TOKEN_KEY).to_writable(),
            user_token_out: Self::ata(payer, output_mint, &*SPL_TOKEN_KEY).to_writable(),
            token_x_mint: self.data.token_x_mint.to_readonly(),
            token_y_mint: self.data.token_y_mint.to_readonly(),
            oracle: self.data.oracle.to_writable(),
            host_fee_in: METEORA_DLMM_PROGRAM.to_program(),
            user: payer.to_signer(),
            token_x_program: SPL_TOKEN_KEY.to_program(),
            token_y_program: SPL_TOKEN_KEY.to_program(),
            event_authority: Pubkey::find_program_address(&[b"__event_authority"], &*METEORA_DLMM_PROGRAM).0.to_readonly(),
            program: METEORA_DLMM_PROGRAM.to_program(),
            bin_arrays: bin_arrays.iter().map(|a| a.to_writable()).collect(),
        })
    }
}

impl PoolConfigInit<MeteoraDlmmAccountData, MeteoraDlmmSwapAccounts> for MeteoraDlmmPoolConfig {
    fn init(
        pool: &Pubkey,
        account_data: MeteoraDlmmAccountData,
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
    ) -> Result<MeteoraDlmmSwapAccounts> {
        // Default to small swap bin arrays
        // For actual swaps, should call build_accounts_with_amount
        self.build_accounts_with_amount(payer, input_mint, output_mint, 0)
    }
}

