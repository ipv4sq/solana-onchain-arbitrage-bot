use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfigInit};
use crate::arb::pool::pump_amm::account::PumpAmmAccountSwapAccounts;
use crate::arb::pool::pump_amm::data::{PumpAmmPoolConfig, PumpAmmPoolData};
use anyhow::Result;
use solana_program::pubkey::Pubkey;

impl PoolConfigInit<PumpAmmPoolData, PumpAmmAccountSwapAccounts> for PumpAmmPoolConfig {
    fn init(pool: &Pubkey, account_data: PumpAmmPoolData, desired_mint: Pubkey) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(PumpAmmPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
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

    fn build_accounts(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount_in: Option<u64>,
        amount_out: Option<u64>,
    ) -> Result<PumpAmmAccountSwapAccounts> {
        todo!()
    }
}
