use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
#[path = "cache_type_test.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheType {
    MintRecord,
    PoolConfig,
    Custom(String),
}

impl CacheType {
    pub fn as_str(&self) -> &str {
        match self {
            CacheType::MintRecord => "mint_record",
            CacheType::PoolConfig => "pool_config",
            CacheType::Custom(name) => name.as_str(),
        }
    }
}

impl fmt::Display for CacheType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<CacheType> for String {
    fn from(cache_type: CacheType) -> Self {
        cache_type.as_str().to_string()
    }
}