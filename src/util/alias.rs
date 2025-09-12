use anyhow::{Error, Result};
use solana_program::pubkey::Pubkey;
use solana_sdk::native_token::LAMPORTS_PER_SOL;

// type alias
pub type VaultAddress = Pubkey;
pub type MintAddress = Pubkey;
pub type TokenProgramAddress = Pubkey;
pub type PoolAddress = Pubkey;
pub type AResult<T, E = Error> = Result<T, E>;

pub type Literal = f64;
pub type Lamport = u64;

pub trait SOLUnitConvert {
    fn to_lamport(&self) -> Lamport;
}
impl SOLUnitConvert for Literal {
    fn to_lamport(&self) -> Lamport {
        (self * LAMPORTS_PER_SOL as f64) as Lamport
    }
}
