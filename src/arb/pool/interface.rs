use crate::constants::addresses::TokenProgram;
use anyhow::Result;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;

#[macro_export]
macro_rules! impl_swap_accounts_to_list {
    ($struct_name:ident { $($field:ident),+ $(,)? }) => {
        impl $crate::arb::pool::interface::SwapAccountsToList for $struct_name {
            fn to_list(&self) -> Vec<&AccountMeta> {
                vec![
                    $(&self.$field),+
                ]
            }
        }
    };
    // Alternate syntax without braces for backwards compatibility
    ($struct_name:ident, $($field:ident),+ $(,)?) => {
        impl_swap_accounts_to_list!($struct_name { $($field),+ });
    };
}

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
    pub data: T,
    pub desired_mint: Pubkey,
    pub minor_mint: Pubkey,
}

pub trait PoolConfigInit<T, P>: Sized {
    fn init(pool: &Pubkey, account_data: T, desired_mint: Pubkey) -> Result<Self>;
    fn build_accounts(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<P>;

    fn ata(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
        get_associated_token_address_with_program_id(owner, mint, token_program)
    }
}

pub trait SwapAccountsToList: Sized {
    fn to_list(&self) -> Vec<&AccountMeta>;
}
