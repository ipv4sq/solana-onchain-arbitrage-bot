use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;

#[macro_export]
macro_rules! impl_swap_accounts_to_list {
    ($struct_name:ident { $($field:ident),+ $(,)? }) => {
        impl $crate::arb::convention::pool::interface::SwapAccountsToList for $struct_name {
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

pub fn ata(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, token_program)
}

pub fn ata_sol_token(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, &spl_token::id())
}
