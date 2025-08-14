use anyhow::Result;
use solana_program::pubkey::Pubkey;

pub trait PoolAccountDataLoader: Sized {
    fn load_data(data: &[u8]) -> Result<Self>;

    // mints
    fn get_base_mint(&self) -> Pubkey;
    fn get_quote_mint(&self) -> Pubkey;

    // vaults
    fn get_base_vault(&self) -> Pubkey;
    fn get_quote_vault(&self) -> Pubkey;

    //
    fn consists_of(&self, mint1: &Pubkey, mint2: &Pubkey) -> Result<()> {
        let base = self.get_base_mint();
        let quote = self.get_quote_mint();
        if base == *mint1 && quote == *mint2 {
            return Ok(());
        }
        if base == *mint2 && quote == *mint1 {
            return Ok(());
        }
        Err(anyhow::anyhow!(
            "Pool doesn't contain {} and {}, instead, it's {} and {}",
            mint1,
            mint2,
            base,
            quote
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PoolConfig<T> {
    pub pool_data: T,
    pub desired_mint: Pubkey,
    pub minor_mint: Pubkey,
}

pub trait PoolConfigInit: Sized {
    fn init<T>(account_data: T, desired_mint: Pubkey) -> Result<Self>;
}
