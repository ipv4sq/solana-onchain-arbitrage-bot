use solana_program::pubkey::Pubkey;
use std::str::FromStr;

/// Extension trait for &str to easily convert to Pubkey
pub trait ToPubkey {
    fn to_pubkey(&self) -> Pubkey;
}

impl ToPubkey for &str {
    fn to_pubkey(&self) -> Pubkey {
        Pubkey::from_str(self).unwrap()
    }
}

impl ToPubkey for String {
    fn to_pubkey(&self) -> Pubkey {
        Pubkey::from_str(self).unwrap()
    }
}