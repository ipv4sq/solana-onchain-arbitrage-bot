use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Signature;
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

/// Extension trait for &str to easily convert to Signature
pub trait ToSignature {
    fn to_sig(&self) -> Signature;
}

impl ToSignature for &str {
    fn to_sig(&self) -> Signature {
        Signature::from_str(self).unwrap()
    }
}

impl ToSignature for String {
    fn to_sig(&self) -> Signature {
        Signature::from_str(self).unwrap()
    }
}
