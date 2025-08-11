use solana_program::pubkey::Pubkey;
use std::str::FromStr;

use crate::constants::addresses::{DEFAULT_LOOKUP_TABLE, SOL_MINT};

pub fn sol_mint() -> Pubkey {
    Pubkey::from_str(SOL_MINT).unwrap()
}

pub fn default_lookup_table() -> Pubkey {
    Pubkey::from_str(DEFAULT_LOOKUP_TABLE).unwrap()
}