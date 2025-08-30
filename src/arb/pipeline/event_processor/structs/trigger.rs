use crate::arb::pipeline::event_processor::structs::pool_update::PoolUpdate;
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub enum Trigger {
    PoolUpdate(PoolUpdate),
    PoolAddress(Pubkey),
}

impl Trigger {
    pub fn pool(&self) -> &Pubkey {
        match self {
            Trigger::PoolUpdate(update) => update.pool(),
            Trigger::PoolAddress(addr) => addr,
        }
    }

    pub fn as_pool_update(&self) -> Option<&PoolUpdate> {
        match self {
            Trigger::PoolUpdate(update) => Some(update),
            Trigger::PoolAddress(_) => None,
        }
    }
}
