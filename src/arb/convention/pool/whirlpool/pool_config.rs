use crate::arb::convention::pool::interface::{PoolConfig, PoolConfigInit, PoolDataLoader};
use crate::arb::convention::pool::whirlpool::data::WhirlpoolPoolData;
use crate::arb::global::constant::token_program::TokenProgram;
use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

type WhirlpoolPoolConfig = PoolConfig<WhirlpoolPoolData>;
pub struct WhirlpoolSwapAccounts {}
impl PoolConfigInit<WhirlpoolPoolData> for WhirlpoolPoolConfig {
    fn from_pool_data(
        pool: &Pubkey,
        account_data: WhirlpoolPoolData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(WhirlpoolPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
            desired_mint_token_program: TokenProgram::SPL_TOKEN.to_pubkey(),
            minor_mint_token_program: TokenProgram::TOKEN_2022.to_pubkey(),
            // readonly_accounts: vec![
            //     // TODO memo program
            //     desired_mint,
            //     *WHIRLPOOL_PROGRAM,
            // ],
            // partial_writeable_accounts: concat(vec![
            //     vec![
            //         *pool,
            //         WhirlpoolAccountData::get_oracle(pool),
            //         account_data.token_vault_a,
            //         account_data.token_vault_b,
            //     ],
            //     account_data.get_tick_arrays(pool),
            // ]),
        })
    }
}
