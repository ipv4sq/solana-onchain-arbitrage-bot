use super::message::Message;
use super::meta::TransactionMeta;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub signature: String,
    pub slot: u64,
    pub message: Message,
    pub meta: Option<TransactionMeta>,
}
