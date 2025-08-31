use crate::arb::pipeline::event_processor::structs::pool_update::AccountComparison;
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub enum Trigger {
    AccountCompare(AccountComparison),
    PoolAddress(Pubkey),
}

impl Trigger {
    pub fn pool(&self) -> &Pubkey {
        match self {
            Trigger::AccountCompare(update) => update.pool(),
            Trigger::PoolAddress(addr) => addr,
        }
    }

    pub fn as_pool_update(&self) -> Option<&AccountComparison> {
        match self {
            Trigger::AccountCompare(update) => Some(update),
            Trigger::PoolAddress(_) => None,
        }
    }
}
