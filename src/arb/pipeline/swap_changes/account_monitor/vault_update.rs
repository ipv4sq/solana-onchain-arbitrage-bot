use crate::arb::convention::chain::AccountState;
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct VaultUpdate {
    pub previous: AccountState,
    pub current: AccountState,
}

impl VaultUpdate {
    pub fn vault(&self) -> &Pubkey {
        &self.current.pubkey
    }

    pub fn lamport_change(&self) -> i64 {
        self.current.calculate_lamport_change(&self.previous)
    }

    pub fn data_changed(&self) -> bool {
        self.current.data_changed(&self.previous)
    }

    pub fn owner_changed(&self) -> bool {
        self.current.owner_changed(&self.previous)
    }

    pub fn slot_delta(&self) -> u64 {
        self.current.slot - self.previous.slot
    }
}
