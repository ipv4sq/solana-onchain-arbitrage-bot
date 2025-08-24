use solana_sdk::signature::Signature;
use std::str::FromStr;

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
