use crate::arb::util::traits::pubkey::ToPubkey;
use solana_program::pubkey::Pubkey;

// Address lookup table addresses
pub struct LookupTable;
impl LookupTable {
    // Default lookup table for this bot
    pub const DEFAULT: &'static str = "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC";
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_all_addresses_are_valid() {
        // All addresses for validation
        let all_addresses = &[("LookupTable::DEFAULT", LookupTable::DEFAULT)];

        for (name, address) in all_addresses {
            // Test that it's a valid Pubkey
            let result = Pubkey::from_str(address);
            assert!(
                result.is_ok(),
                "{} with value '{}' should be a valid Solana address",
                name,
                address
            );

            // Verify round-trip conversion
            let pubkey = result.unwrap();
            assert_eq!(
                pubkey.to_string(),
                *address,
                "{} should round-trip correctly",
                name
            );
        }
    }
}
