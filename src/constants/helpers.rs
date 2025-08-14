use solana_program::instruction::AccountMeta;
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

/// Extension trait for Pubkey to easily create AccountMeta
pub trait ToAccountMeta {
    fn to_signer(&self) -> AccountMeta;
    fn to_readonly(&self) -> AccountMeta;
    fn to_writable(&self) -> AccountMeta;
    fn to_program(&self) -> AccountMeta;
}

impl ToAccountMeta for String {
    fn to_signer(&self) -> AccountMeta {
        AccountMeta::new(self.to_pubkey(), true)
    }

    fn to_readonly(&self) -> AccountMeta {
        AccountMeta::new_readonly(self.to_pubkey(), false)
    }

    fn to_writable(&self) -> AccountMeta {
        AccountMeta::new(self.to_pubkey(), false)
    }

    fn to_program(&self) -> AccountMeta {
        AccountMeta::new_readonly(self.to_pubkey(), false)
    }
}
impl ToAccountMeta for &str {
    fn to_signer(&self) -> AccountMeta {
        AccountMeta::new(self.to_pubkey(), true)
    }

    fn to_readonly(&self) -> AccountMeta {
        AccountMeta::new_readonly(self.to_pubkey(), false)
    }

    fn to_writable(&self) -> AccountMeta {
        AccountMeta::new(self.to_pubkey(), false)
    }

    fn to_program(&self) -> AccountMeta {
        AccountMeta::new_readonly(self.to_pubkey(), false)
    }
}

impl ToAccountMeta for Pubkey {
    fn to_signer(&self) -> AccountMeta {
        AccountMeta::new(*self, true)
    }

    fn to_readonly(&self) -> AccountMeta {
        AccountMeta::new_readonly(*self, false)
    }

    fn to_writable(&self) -> AccountMeta {
        AccountMeta::new(*self, false)
    }

    fn to_program(&self) -> AccountMeta {
        AccountMeta::new_readonly(*self, false)
    }
}
