use anyhow::Result;
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