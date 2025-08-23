use crate::arb::convention::chain::Transaction;
use anyhow::Result;

pub trait ToUnified {
    fn to_unified(&self) -> Result<Transaction>;
}
