use crate::arb::chain::message::Message;
use crate::arb::chain::meta::TransactionMeta;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub signature: String,
    pub slot: u64,
    pub message: Message,
    pub meta: Option<TransactionMeta>,
}
