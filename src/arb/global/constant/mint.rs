use crate::constants::helpers::ToPubkey;
use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;
pub struct Mints;

impl Mints {
    pub const WSOL: &'static str = "So11111111111111111111111111111111111111112";
    pub const USDC: &'static str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
}

lazy_static! {
    pub static ref WSOL_KEY: Pubkey = Mints::WSOL.to_pubkey();
    pub static ref USDC_KEY: Pubkey = Mints::USDC.to_pubkey();
}

