use solana_program::pubkey::Pubkey;
use solana_sdk::pubkey;

// Token program addresses
pub struct TokenProgram;

impl TokenProgram {
    // SPL Token program (included for completeness, use spl_token::ID when possible)
    pub const SPL_TOKEN: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    // Token 2022 program
    pub const TOKEN_2022: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
}
