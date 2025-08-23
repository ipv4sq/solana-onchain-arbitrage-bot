use anyhow::Result;
use solana_program::pubkey::Pubkey;
use crate::arb::global::constant::mint::{USDC_KEY, WSOL_KEY};

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

    pub fn consists_of(&self, mint1: &Pubkey, mint2: &Pubkey) -> Result<()> {
        if self.0 == *mint1 && self.1 == *mint2 {
            Ok(())
        } else if self.0 == *mint2 && self.1 == *mint1 {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Pair doesn't contain {} and {}, instead, it's {} and {}",
                mint1,
                mint2,
                self.0,
                self.1
            ))
        }
    }

    pub fn shall_contain(&self, mint: &Pubkey) -> Result<()> {
        match self.0 == *mint || self.1 == *mint {
            true => Ok(()),
            false => Err(anyhow::anyhow!(
                "This pool doesn't contain {} or {}",
                self.0,
                self.1
            )),
        }
    }

    pub fn minor_mint(&self) -> Result<Pubkey> {
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