use crate::constants::addresses::TokenMint;
use crate::constants::helpers::ToPubkey;
use crate::in_list;
use solana_program::pubkey::Pubkey;

pub trait PoolChecker: Sized {
    fn get_base_mint(&self) -> Pubkey;
    fn get_base_account(&self) -> Pubkey;

    fn get_token_mint(&self) -> Pubkey;
    fn get_token_account(&self) -> Pubkey;

    fn include_sol(&self) -> bool {
        let sol = TokenMint::SOL.to_pubkey();
        in_list!(sol, self.get_base_mint(), self.get_token_mint())
    }

    fn must_include_sol(&self) -> anyhow::Result<bool> {
        if !self.include_sol() {
            return Err(anyhow::anyhow!("This pool doesn't include sol!",));
        }
        Ok(true)
    }

    fn get_sol_mint(&self) -> anyhow::Result<Pubkey> {
        self.must_include_sol()?;
        if self.get_base_mint() == TokenMint::SOL.to_pubkey() {
            Ok(self.get_base_mint())
        } else {
            Ok(self.get_token_mint())
        }
    }
}
