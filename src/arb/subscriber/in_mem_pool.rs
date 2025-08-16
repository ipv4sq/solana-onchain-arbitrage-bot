use crate::arb::pool::interface::PoolConfigInit;
use crate::arb::pool::meteora_damm_v2::pool_config::MeteoraDammV2Config;
use crate::arb::pool::meteora_dlmm::pool_config::MeteoraDlmmPoolConfig;
use crate::arb::tx::constants::DexType;
use crate::arb::tx::types::LitePool;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub enum AnyPoolConfig {
    MeteoraDlmm(MeteoraDlmmPoolConfig),
    MeteoraDammV2(MeteoraDammV2Config),
    Unsupported,
}

static MEM_POOL: Lazy<Arc<MemPool>> = Lazy::new(|| Arc::new(MemPool::new()));

pub fn mem_pool() -> Arc<MemPool> {
    MEM_POOL.clone()
}

pub struct MemPool {
    pub watching: RwLock<HashMap<String, AnyPoolConfig>>,
    pub queued: RwLock<HashMap<String, LitePool>>,
}

impl MemPool {
    fn new() -> MemPool {
        MemPool {
            watching: Default::default(),
            queued: Default::default(),
        }
    }

    pub fn add_if_not_exists(&self, pool: LitePool) -> Result<()> {
        let read_guard = self
            .queued
            .read()
            .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?;
        match read_guard.get(&pool.pool_address.to_string()) {
            None => {
                drop(read_guard);
                self.queued
                    .write()
                    .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?
                    .insert(pool.pool_address.to_string(), pool.clone());

                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    let _ = MemPool::add_to_register(pool_clone).await;
                });
            }
            Some(_) => {}
        }

        Ok(())
    }

    async fn add_to_register(pool: LitePool) -> Result<()> {
        let mem = mem_pool();

        // Load the appropriate config based on dex_type
        let config = match pool.dex_type {
            DexType::MeteoraDlmm => {
                let config = MeteoraDlmmPoolConfig::from_address(&pool.pool_address).await?;
                AnyPoolConfig::MeteoraDlmm(config)
            }
            DexType::MeteoraDammV2 => {
                let config = MeteoraDammV2Config::from_address(&pool.pool_address).await?;
                AnyPoolConfig::MeteoraDammV2(config)
            }
            _ => AnyPoolConfig::Unsupported,
        };

        mem.watching
            .write()
            .map_err(|e| anyhow::anyhow!("Write lock poisoned: {}", e))?
            .insert(pool.pool_address.to_string(), config);
        Ok(())
    }
}
