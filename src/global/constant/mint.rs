use solana_program::pubkey::Pubkey;
use solana_sdk::pubkey;

pub struct Mints;

impl Mints {
    pub const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
    pub const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    pub const USDT: Pubkey = pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
}
