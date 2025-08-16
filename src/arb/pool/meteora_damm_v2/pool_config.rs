use crate::arb::pool::interface::{PoolConfig, PoolConfigInit, PoolDataLoader};
use crate::arb::pool::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
pub type MeteoraDammV2Config = PoolConfig<MeteoraDammV2PoolData>;

impl PoolConfigInit<MeteoraDammV2PoolData> for MeteoraDammV2Config {
    fn from_pool_data(pool: &Pubkey, pool_data: MeteoraDammV2PoolData, desired_mint: Pubkey) -> Result<Self> {
        pool_data.shall_contain(&desired_mint)?;
        let minor_mint = pool_data.the_other_mint(pool)?;
        Ok(MeteoraDammV2Config {
            pool: *pool,
            data: pool_data,
            desired_mint,
            minor_mint,
        })
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_build_accounts() {

    }
}
