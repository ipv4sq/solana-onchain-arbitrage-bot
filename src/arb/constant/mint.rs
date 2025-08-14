use crate::constants::helpers::ToPubkey;
use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;

pub struct Mints;

impl Mints {
    pub const WSOL: &'static str = "So11111111111111111111111111111111111111112";
}

lazy_static! {
    pub static ref WSOL_KEY: Pubkey = Mints::WSOL.to_pubkey();
}
