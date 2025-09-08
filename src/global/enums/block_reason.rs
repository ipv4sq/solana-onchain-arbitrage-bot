use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlocklistReason {
    SaveFailed,
    QueryFailed,
    AccountNotFound,
    InvalidDataSize { size: usize },
    NotInDatabase,
    NoWsolInvolved,
}
