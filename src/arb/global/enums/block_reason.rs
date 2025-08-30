use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlocklistReason {
    AccountNotFound,
    InvalidDataSize { size: usize },
    NotInDatabase,
    NoWsolInvolved,
}
