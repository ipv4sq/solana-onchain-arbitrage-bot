use crate::arb::pool::example::whirlpool::data::WhirlpoolPoolData;
use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use anyhow::Result;
use solana_program::pubkey::Pubkey;

type WhirlpoolPoolConfig = PoolConfig<WhirlpoolPoolData>;
pub struct WhirlpoolSwapAccounts {}
impl PoolConfigInit<WhirlpoolPoolData, WhirlpoolSwapAccounts> for WhirlpoolPoolConfig {
    fn init(pool: &Pubkey, account_data: WhirlpoolPoolData, desired_mint: Pubkey) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(WhirlpoolPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
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

    fn build_accounts(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_in: Option<u64>,
        amount_out: Option<u64>,
    ) -> Result<WhirlpoolSwapAccounts> {
        todo!()
    }
}
