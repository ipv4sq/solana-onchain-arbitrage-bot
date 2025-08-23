use anyhow::Result;
use crate::arb::convention::chain::Transaction;

pub trait ToUnified {
    fn to_unified(&self) -> Result<Transaction>;
}
