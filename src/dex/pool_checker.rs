use crate::constants::addresses::TokenMint;
use crate::constants::helpers::ToPubkey;
use crate::in_list;
use solana_program::pubkey::Pubkey;

pub trait PoolChecker: Sized {
    fn get_base_mint(&self) -> Pubkey;
    fn get_token_mint(&self) -> Pubkey;

    fn get_base_vault(&self) -> Pubkey;
    fn get_token_vault(&self) -> Pubkey;

    fn include_sol(&self) -> bool {
        let sol = TokenMint::SOL.to_pubkey();
        in_list!(sol, self.get_base_mint(), self.get_token_mint())
    }

    fn must_include_sol(&self, pool_address: Option<&Pubkey>) -> anyhow::Result<bool> {
        if !self.include_sol() {
            let address = match pool_address {
                Some(pool_address) => pool_address.to_string(),
                None => { "Unknown" }.parse()?,
            };

            return Err(anyhow::anyhow!(
                "This pool {} doesn't include sol!",
                address
            ));
        }
        Ok(true)
    }

    fn get_sol_mint(&self) -> anyhow::Result<Pubkey> {
        self.must_include_sol(None)?;
        if self.get_base_mint() == TokenMint::SOL.to_pubkey() {
            Ok(self.get_base_mint())
        } else {
            Ok(self.get_token_mint())
        }
    }

    fn get_sol_vault(&self) -> anyhow::Result<Pubkey> {
        self.must_include_sol(None)?;
        if self.get_base_mint() == TokenMint::SOL.to_pubkey() {
            Ok(self.get_base_vault())
        } else {
            Ok(self.get_token_vault())
        }
    }

    fn get_not_sol_mint(&self) -> anyhow::Result<Pubkey> {
        self.must_include_sol(None)?;
        if self.get_base_mint() == TokenMint::SOL.to_pubkey() {
            Ok(self.get_token_mint())
        } else {
            Ok(self.get_base_mint())
        }
    }
}
