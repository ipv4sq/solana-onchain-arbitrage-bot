use crate::util::traits::pubkey::ToPubkey;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

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
