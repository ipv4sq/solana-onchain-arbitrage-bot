#[cfg(test)]
mod tests {
    use crate::util::structs::cache_type::CacheType;
    use crate::util::structs::persistent_cache::PersistentCache;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_type_enum_usage() {
        let mint_cache: PersistentCache<String, serde_json::Value> = PersistentCache::new(
            CacheType::MintRecord,
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

        let custom_cache: PersistentCache<String, f64> = PersistentCache::new(
            CacheType::Custom("price_data".to_string()),
            50,
            Duration::from_secs(60),
            |pair: &String| async move { Some(150.25) },
        );

        let mint_info = mint_cache
            .get(&"So11111111111111111111111111111111111111112".to_string())
            .await;
        assert!(mint_info.is_some());

        let price = custom_cache.get(&"SOL/USDC".to_string()).await;
        assert_eq!(price, Some(150.25));
    }

    #[test]
    fn test_cache_type_display() {
        assert_eq!(CacheType::MintRecord.as_str(), "mint_record");
        assert_eq!(
            CacheType::Custom("my_custom".to_string()).as_str(),
            "my_custom"
        );
    }
}
