use solana_program::pubkey::Pubkey;

// Token mint addresses
pub struct TokenMint;
impl TokenMint {
    // Native SOL mint address
    pub const SOL: &'static str = "So11111111111111111111111111111111111111112";
    
    pub const USDC: &'static str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
}

// Token program addresses
pub struct TokenProgram;
impl TokenProgram {
    // SPL Token program (included for completeness, use spl_token::ID when possible)
    pub const SPL_TOKEN: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    // Token 2022 program
    pub const TOKEN_2022: &'static str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
}

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
        let all_addresses = &[
            ("TokenMint::SOL", TokenMint::SOL),
            ("TokenProgram::SPL_TOKEN", TokenProgram::SPL_TOKEN),
            ("TokenProgram::TOKEN_2022", TokenProgram::TOKEN_2022),
            ("LookupTable::DEFAULT", LookupTable::DEFAULT),
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
        }
    }
}