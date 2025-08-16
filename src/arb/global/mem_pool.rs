use crate::arb::constant::pool_owner::AnyPoolConfig;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::arb::chain::types::LitePool;

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
        let config = AnyPoolConfig::from_address(&pool.pool_address, pool.dex_type).await?;

        mem_pool()
            .watching
            .write()
            .map_err(|e| anyhow::anyhow!("Write lock poisoned: {}", e))?
            .insert(pool.pool_address.to_string(), config);
        Ok(())
    }
}
