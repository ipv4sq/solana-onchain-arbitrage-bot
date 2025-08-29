use crate::arb::dex::interface::{PoolConfig, PoolConfigInit, PoolDataLoader};
use crate::arb::dex::raydium_cpmm::data::RaydiumCpmmAPoolData;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

const RAYDIUM_CPMM_AUTHORITY: &str = "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL";

pub type RaydiumCpmmPoolConfig = PoolConfig<RaydiumCpmmAPoolData>;

impl PoolConfigInit<RaydiumCpmmAPoolData> for RaydiumCpmmPoolConfig {
    fn from_pool_data(
        pool: &Pubkey,
        account_data: RaydiumCpmmAPoolData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(RaydiumCpmmPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.pair().minor_mint()?,
        })
    }

    // fn build_accounts(
    //     &self,
    //     payer: &Pubkey,
    //     input_mint: &Pubkey,
    //     output_mint: &Pubkey,
    //     amount_in: Option<u64>,
    //     amount_out: Option<u64>,
    // ) -> Result<RaydiumCpmmSwapAccounts> {
    //     Ok(RaydiumCpmmSwapAccounts {
    //         payer: payer.to_signer(),
    //         authority: RAYDIUM_CPMM_AUTHORITY.to_pubkey().to_writable(),
    //         amm_config: self.data.amm_config.to_writable(),
    //         pool_state: self.pool.to_writable(),
    //         input_token_account: ata(payer, input_mint, &*SPL_TOKEN_KEY).to_writable(),
    //         output_token_account: ata(payer, output_mint, &*SPL_TOKEN_KEY).to_writable(),
    //         input_vault: self.data.token_0_vault.to_writable(),
    //         output_vault: self.data.token_1_vault.to_writable(),
    //         input_token_program: SPL_TOKEN_KEY.to_readonly(),
    //         output_token_program: SPL_TOKEN_KEY.to_readonly(),
    //         input_token_mint: input_mint.to_writable(),
    //         output_token_mint: output_mint.to_writable(),
    //         observation_state: self.data.observation_key.to_writable(),
    //     })
    // }
}
