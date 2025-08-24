use anyhow::{Error, Result};
use solana_program::pubkey::Pubkey;

// type alias
pub type VaultAddress = Pubkey;
pub type MintAddress = Pubkey;
pub type TokenProgramAddress = Pubkey;
pub type PoolAddress = Pubkey;
pub type AResult<T, E = Error> = Result<T, E>;
