use crate::arb::database::entity::pool_do;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

pub async fn on_swap_occurred(mint: &Pubkey, pool: pool_do::Model) {}
