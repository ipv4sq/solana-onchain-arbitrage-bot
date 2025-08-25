use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
#[path = "cache_type_test.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheType {
    PoolMetadata,
    MintInfo,
    PriceData,
    AccountData,
    TransactionData,
    MarketData,
    RoutingData,
    Custom(String),
}

impl CacheType {
    pub fn as_str(&self) -> &str {
        match self {
            CacheType::PoolMetadata => "pool_metadata",
            CacheType::MintInfo => "mint_info",
            CacheType::PriceData => "price_data",
            CacheType::AccountData => "account_data",
            CacheType::TransactionData => "transaction_data",
            CacheType::MarketData => "market_data",
            CacheType::RoutingData => "routing_data",
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