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

    fn shall_contain(&self, mint: &Pubkey) -> Result<()> {
        match self.get_base_mint() == *mint || self.get_quote_mint() == *mint {
            true => Ok(()),
            false => Err(anyhow::anyhow!(
                "This pool doesn't contain {} or {}",
                self.get_quote_mint(),
                self.get_base_mint()
            )),
        }
    }

    fn the_other_mint(&self, excluded_mint: &Pubkey) -> Result<Pubkey> {
        self.shall_contain(excluded_mint)?;
        let base = self.get_base_mint();
        let quote = self.get_quote_mint();
        if base == *excluded_mint {
            Ok(quote)
        } else {
            Ok(base)
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolConfig<T> {
    pub pool: Pubkey,
    pub pool_data: T,
    pub desired_mint: Pubkey,
    pub minor_mint: Pubkey,
    pub readonly_accounts: Vec<Pubkey>,
    pub writeable_accounts: Vec<Pubkey>,
}

pub trait PoolConfigInit<T>: Sized {
    fn init(pool: &Pubkey, account_data: T, desired_mint: Pubkey) -> Result<Self>;
}
