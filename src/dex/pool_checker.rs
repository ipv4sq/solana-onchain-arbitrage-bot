use crate::arb::global::constant::mint::Mints;
use crate::constants::helpers::ToPubkey;
use crate::in_list;
use solana_program::pubkey::Pubkey;

pub trait PoolChecker: Sized {
    fn get_base_mint(&self) -> Pubkey;
    fn get_token_mint(&self) -> Pubkey;

    fn get_base_vault(&self) -> Pubkey;
    fn get_token_vault(&self) -> Pubkey;

    fn include_sol(&self) -> bool {
        let sol = Mints::WSOL;
        in_list!(sol, self.get_base_mint(), self.get_token_mint())
    }

    // orders doesn't matter here
    fn consists_of(
        &self,
        mint1: &Pubkey,
        mint2: &Pubkey,
        pool: Option<&Pubkey>,
    ) -> anyhow::Result<()> {
        if self.get_base_mint() == *mint1 && self.get_token_mint() == *mint2 {
            return Ok(());
        }
        if self.get_token_mint() == *mint1 && self.get_base_mint() == *mint2 {
            return Ok(());
        }
        Err(anyhow::anyhow!(
            "Pool {} doesn't contain {} and {}, instead, it's {} and {}",
            pool.map_or_else(|| "unknown".to_string(), |t| t.to_string()),
            mint1,
            mint2,
            self.get_base_mint(),
            self.get_token_mint()
        ))
    }

    fn shall_include_sol(&self, pool: Option<&Pubkey>) -> anyhow::Result<()> {
        if !self.include_sol() {
            return Err(anyhow::anyhow!(
                "This pool {} doesn't include sol!",
                pool.map_or_else(|| "unknown".to_string(), |t| t.to_string()),
            ));
        }
        Ok(())
    }

    fn get_sol_mint(&self) -> anyhow::Result<Pubkey> {
        self.shall_include_sol(None)?;
        if self.get_base_mint() == Mints::WSOL {
            Ok(self.get_base_mint())
        } else {
            Ok(self.get_token_mint())
        }
    }

    fn get_sol_vault(&self) -> anyhow::Result<Pubkey> {
        self.shall_include_sol(None)?;
        if self.get_base_mint() == Mints::WSOL {
            Ok(self.get_base_vault())
        } else {
            Ok(self.get_token_vault())
        }
    }

    fn get_not_sol_mint(&self) -> anyhow::Result<Pubkey> {
        self.shall_include_sol(None)?;
        if self.get_base_mint() == Mints::WSOL {
            Ok(self.get_token_mint())
        } else {
            Ok(self.get_base_mint())
        }
    }
}
