use crate::constants::helpers::ToPubkey;
use anyhow::Result;
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
#[derive(Debug, Clone)]
pub struct MintPair(pub Pubkey, pub Pubkey);

impl MintPair {
    pub fn sol_mint(&self) -> Result<&Pubkey> {
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

    pub fn desired_mint(&self) -> Result<Pubkey> {
        if self.0 == *WSOL_KEY || self.1 == *WSOL_KEY {
            Ok(*WSOL_KEY)
        } else if self.0 == *USDC_KEY || self.1 == *USDC_KEY {
            Ok(*USDC_KEY)
        } else {
            Err(anyhow::anyhow!(""))
        }
    }

    pub fn the_other_mint(&self) -> Result<Pubkey> {
        let desired_mint = self.desired_mint()?;
        if self.0 == desired_mint {
            Ok(self.1)
        } else if self.1 == desired_mint {
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
