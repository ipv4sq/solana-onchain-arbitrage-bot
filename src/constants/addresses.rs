//! Solana addresses constants
//! All strings in this file are valid Solana addresses (base58 encoded public keys)

#![allow(clippy::all)]
// Suppress IDE typo warnings for base58 addresses
//noinspection SpellCheckingInspection

// Token mints
pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

// Address lookup tables
pub const DEFAULT_LOOKUP_TABLE: &str = "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC";

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_all_addresses_are_valid() {
        // All addresses for validation
        let all_addresses = &[
            ("SOL_MINT", SOL_MINT),
            ("DEFAULT_LOOKUP_TABLE", DEFAULT_LOOKUP_TABLE),
        ];

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

            // Verify the address length is valid for base58
            assert!(
                address.len() >= 32 && address.len() <= 44,
                "{} should have valid base58 length (32-44 chars), got {}",
                name,
                address.len()
            );

            // Verify it only contains valid base58 characters (no 0, O, I, l)
            let valid_chars = address.chars().all(|c| {
                matches!(c, 
                    '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'
                )
            });
            assert!(
                valid_chars,
                "{} should only contain valid base58 characters",
                name
            );

            // Verify it's not the default (all zeros) pubkey
            assert_ne!(
                pubkey,
                Pubkey::default(),
                "{} should not be the default pubkey",
                name
            );
        }
    }
}