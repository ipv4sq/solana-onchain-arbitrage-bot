use crate::arb::convention::pool::interface::{PoolConfigInit, PoolDataLoader};
use crate::arb::convention::pool::pump_amm::data::{PumpAmmPoolConfig, PumpAmmPoolData};
use crate::arb::global::constant::token_program::TokenProgram;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

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
            desired_mint_token_program: TokenProgram::SPL_TOKEN,
            minor_mint_token_program: TokenProgram::TOKEN_2022,
            // readonly_accounts: vec![
            //     desired_mint,
            //     *PUMP_PROGRAM,
            // ],
            // partial_writeable_accounts: vec![
            //     *pool,
            //     account_data.pool_base_token_account,
            //     account_data.pool_quote_token_account,
            //     PumpAmmAccountData::get_creator_vault_ata(
            //         &PumpAmmAccountData::get_creator_vault_authority(&account_data.coin_creator),
            //         &account_data.base_mint,
            //     ),
            // ],
        })
    }
}
