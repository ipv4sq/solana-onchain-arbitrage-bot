use super::message::Message;
use super::meta::TransactionMeta;

#[derive(Debug, Clone)]
pub struct UnifiedTransaction {
    pub signature: String,
    pub slot: u64,
    pub message: Message,
    pub meta: Option<TransactionMeta>,
}
