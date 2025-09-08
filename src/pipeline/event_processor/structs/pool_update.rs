use crate::convention::chain::AccountState;
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct AccountComparison {
    pub previous: Option<AccountState>,
    pub current: AccountState,
}

impl AccountComparison {
    pub fn pool(&self) -> &Pubkey {
        &self.current.pubkey
    }

    pub fn lamport_change(&self) -> i64 {
        self.previous
            .as_ref()
            .map(|prev| self.current.calculate_lamport_change(prev))
            .unwrap_or_else(|| self.current.lamports.try_into().unwrap_or(i64::MAX))
    }

    pub fn data_changed(&self) -> bool {
        self.previous
            .as_ref()
            .map(|prev| self.current.data_changed(prev))
            .unwrap_or(true)
    }

    pub fn owner_changed(&self) -> bool {
        self.previous
            .as_ref()
            .map(|prev| self.current.owner_changed(prev))
            .unwrap_or(false)
    }

    pub fn slot_delta(&self) -> u64 {
        self.previous
            .as_ref()
            .map(|prev| self.current.slot.saturating_sub(prev.slot))
            .unwrap_or(0)
    }

    pub fn is_initial(&self) -> bool {
        self.previous.is_none()
    }
}
