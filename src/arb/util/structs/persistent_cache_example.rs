use crate::arb::util::structs::cache_type::CacheType;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use std::time::Duration;

pub async fn example_usage() {
    let cache: PersistentCache<String, serde_json::Value> = PersistentCache::new(
        CacheType::PoolMetadata,
        1000,
        Duration::from_secs(3600),
        |pool_address: &String| async move {
            println!("Fetching pool metadata for: {}", pool_address);
            
            let metadata = serde_json::json!({
                "address": pool_address,
                "base_mint": "So11111111111111111111111111111111111111112",
                "quote_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "liquidity": 1000000000u64,
                "timestamp": chrono::Utc::now().timestamp(),
            });
            
            Some(metadata)
        },
    );
    
    let pool_address = "PoolAddressExample123".to_string();
    
    if let Some(metadata) = cache.get(&pool_address).await {
        println!("Retrieved pool metadata: {:?}", metadata);
    }
    
    let custom_data = serde_json::json!({
        "custom": "data",
        "value": 42,
    });
    cache.put("custom_key".to_string(), custom_data).await;
    
    cache.put_with_ttl(
        "short_lived".to_string(),
        serde_json::json!({"temporary": true}),
        Duration::from_secs(60),
    ).await;
    
    cache.evict(&"pool_address".to_string()).await;
}