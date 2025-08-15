use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use crate::arb::pool::meteora_damm_v2::account::MeteoraDammV2SwapAccount;
use crate::arb::pool::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
pub type MeteoraDammV2Config = PoolConfig<MeteoraDammV2PoolData>;

impl PoolConfigInit<MeteoraDammV2PoolData, MeteoraDammV2SwapAccount> for MeteoraDammV2Config {
    fn init(pool: &Pubkey, pool_data: MeteoraDammV2PoolData, desired_mint: Pubkey) -> Result<Self> {
        pool_data.shall_contain(&desired_mint)?;
        let minor_mint = pool_data.the_other_mint(pool)?;
        Ok(MeteoraDammV2Config {
            pool: *pool,
            data: pool_data,
            desired_mint,
            minor_mint,
        })
    }

    fn build_accounts(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> anyhow::Result<MeteoraDammV2SwapAccount> {
        todo!()
    }
}

