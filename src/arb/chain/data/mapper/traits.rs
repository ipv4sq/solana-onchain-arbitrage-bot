use crate::arb::chain::data::Transaction;
use anyhow::Result;

pub trait ToUnified {
    fn to_unified(&self) -> Result<Transaction>;
}
