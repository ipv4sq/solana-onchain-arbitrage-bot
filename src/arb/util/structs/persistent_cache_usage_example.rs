use crate::arb::util::structs::cache_type::CacheType;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use solana_program::pubkey::Pubkey;
use std::time::Duration;
use rand::Rng;

pub struct CacheManager {
    pool_cache: PersistentCache<Pubkey, serde_json::Value>,
    mint_cache: PersistentCache<Pubkey, serde_json::Value>,
    price_cache: PersistentCache<String, f64>,
}

impl CacheManager {
    pub fn new() -> Self {
        let pool_cache = PersistentCache::new(
            CacheType::Custom("pool_metadata".to_string()),
            1000,
            Duration::from_secs(300),
            |pool_address: &Pubkey| {
                let pool_address = *pool_address;
                async move {
                    println!("Fetching pool metadata for: {}", pool_address);
                    Some(serde_json::json!({
                        "address": pool_address.to_string(),
                        "liquidity": 1000000000u64,
                    }))
                }
            },
        );
        
        let mint_cache = PersistentCache::new(
            CacheType::MintRecord,
            500,
            Duration::from_secs(3600),
            |mint: &Pubkey| {
                let mint = *mint;
                async move {
                    println!("Fetching mint info for: {}", mint);
                    Some(serde_json::json!({
                        "mint": mint.to_string(),
                        "decimals": 9,
                        "symbol": "TOKEN"
                    }))
                }
            },
        );
        
        let price_cache = PersistentCache::new(
            CacheType::Custom("price_data".to_string()),
            100,
            Duration::from_secs(30),
            |pair: &String| {
                let pair = pair.clone();
                async move {
                    println!("Fetching price for: {}", pair);
                    Some(150.25)
                }
            },
        );
        
        Self {
            pool_cache,
            mint_cache,
            price_cache,
        }
    }
    
    pub async fn get_pool_data(&self, pool: &Pubkey) -> Option<serde_json::Value> {
        self.pool_cache.get(pool).await
    }
    
    pub async fn get_mint_info(&self, mint: &Pubkey) -> Option<serde_json::Value> {
        self.mint_cache.get(mint).await
    }
    
    pub async fn get_price(&self, pair: &str) -> Option<f64> {
        self.price_cache.get(&pair.to_string()).await
    }
    
    pub async fn update_price(&self, pair: &str, price: f64) {
        self.price_cache.put_with_ttl(
            pair.to_string(),
            price,
            Duration::from_secs(60),
        ).await;
    }
    
    pub async fn invalidate_pool(&self, pool: &Pubkey) {
        self.pool_cache.evict(pool).await;
    }
}

pub async fn example_usage() {
    let cache_manager = CacheManager::new();
    
    let mut rng = rand::rng();
    let bytes: [u8; 32] = rng.gen();
    let pool = Pubkey::new_from_array(bytes);
    if let Some(pool_data) = cache_manager.get_pool_data(&pool).await {
        println!("Pool data: {:?}", pool_data);
    }
    
    let price = cache_manager.get_price("SOL/USDC").await;
    println!("SOL/USDC price: {:?}", price);
    
    cache_manager.update_price("SOL/USDC", 155.50).await;
    
    cache_manager.invalidate_pool(&pool).await;
    
    let custom_cache: PersistentCache<String, String> = PersistentCache::new(
        CacheType::Custom("my_custom_cache".to_string()),
        50,
        Duration::from_secs(120),
        |key: &String| {
            let key = key.clone();
            async move {
                Some(format!("Custom value for {}", key))
            }
        },
    );
    
    let custom_value = custom_cache.get(&"test_key".to_string()).await;
    println!("Custom cache value: {:?}", custom_value);
}