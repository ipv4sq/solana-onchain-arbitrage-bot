use crate::arb::sdk::yellowstone::GrpcAccountUpdate;
use solana_program::pubkey::Pubkey;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct AccountState {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub slot: u64,
    pub last_update: Instant,
}

impl AccountState {
    pub fn from_grpc_update(update: &GrpcAccountUpdate) -> Self {
        Self {
            pubkey: update.account,
            lamports: update.lamports,
            data: update.data.clone(),
            owner: update.owner,
            slot: update.slot,
            last_update: Instant::now(),
        }
    }

    pub fn calculate_lamport_change(&self, previous: &AccountState) -> i64 {
        self.lamports as i64 - previous.lamports as i64
    }

    pub fn data_changed(&self, previous: &AccountState) -> bool {
        self.data != previous.data
    }

    pub fn owner_changed(&self, previous: &AccountState) -> bool {
        self.owner != previous.owner
    }
}