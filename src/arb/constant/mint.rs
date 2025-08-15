use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;
pub struct Mints;

impl Mints {
    pub const WSOL: &'static str = "So11111111111111111111111111111111111111112";
}

lazy_static! {
    pub static ref WSOL_KEY: Pubkey = Mints::WSOL.to_pubkey();
}
#[derive(Debug, Clone)]
pub struct MintPair(pub Pubkey, pub Pubkey);

impl MintPair {
    fn sol_mint(&self) -> Result<&Pubkey> {
        if self.0 == *WSOL_KEY {
            Ok(&self.0)
        } else if self.1 == *WSOL_KEY {
            Ok(&self.1)
        } else {
            Err(anyhow::anyhow!(
                "Pair {} <-> {} doesn't include wsol",
                self.0,
                self.1
            ))
        }
    }

    fn the_other_mint(&self) -> Result<Pubkey> {
        if self.0 == *WSOL_KEY {
            Ok(self.1)
        } else if self.1 == *WSOL_KEY {
            Ok(self.0)
        } else {
            Err(anyhow::anyhow!(
                "Pair {} <-> {} doesn't include wsol",
                self.0,
                self.1
            ))
        }
    }
}
