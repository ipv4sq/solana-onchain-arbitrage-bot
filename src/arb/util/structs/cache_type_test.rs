#[cfg(test)]
mod tests {
    use crate::arb::util::structs::cache_type::CacheType;
    use crate::arb::util::structs::persistent_cache::PersistentCache;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_type_enum_usage() {
        let pool_cache: PersistentCache<String, serde_json::Value> = PersistentCache::new(
            CacheType::PoolMetadata,
            100,
            Duration::from_secs(300),
            |pool_id: &String| {
                let pool_id = pool_id.clone();
                async move {
                    Some(serde_json::json!({
                        "pool_id": pool_id,
                        "liquidity": 1000000
                    }))
                }
            },
        );
        
        let mint_cache: PersistentCache<String, serde_json::Value> = PersistentCache::new(
            CacheType::MintInfo,
            100,
            Duration::from_secs(3600),
            |mint: &String| {
                let mint = mint.clone();
                async move {
                    Some(serde_json::json!({
                        "mint": mint,
                        "decimals": 9,
                        "symbol": "SOL"
                    }))
                }
            },
        );
        
        let price_cache: PersistentCache<String, f64> = PersistentCache::new(
            CacheType::PriceData,
            50,
            Duration::from_secs(60),
            |pair: &String| async move {
                Some(150.25)
            },
        );
        
        let pool_data = pool_cache.get(&"pool123".to_string()).await;
        assert!(pool_data.is_some());
        
        let mint_info = mint_cache.get(&"So11111111111111111111111111111111111111112".to_string()).await;
        assert!(mint_info.is_some());
        
        let price = price_cache.get(&"SOL/USDC".to_string()).await;
        assert_eq!(price, Some(150.25));
    }
    
    #[test]
    fn test_cache_type_display() {
        assert_eq!(CacheType::PoolMetadata.as_str(), "pool_metadata");
        assert_eq!(CacheType::MintInfo.as_str(), "mint_info");
        assert_eq!(CacheType::PriceData.as_str(), "price_data");
        assert_eq!(CacheType::AccountData.as_str(), "account_data");
        assert_eq!(CacheType::TransactionData.as_str(), "transaction_data");
        assert_eq!(CacheType::MarketData.as_str(), "market_data");
        assert_eq!(CacheType::RoutingData.as_str(), "routing_data");
        assert_eq!(CacheType::Custom("my_custom".to_string()).as_str(), "my_custom");
    }
}