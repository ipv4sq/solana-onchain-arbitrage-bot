use crate::arb::pool::meteora_dlmm::pool_config::MeteoraDlmmPoolConfig;
use crate::arb::tx::types::{LitePool, SwapInstruction};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

static MEM_POOL: Lazy<Arc<RwLock<MemPool>>> = Lazy::new(|| Arc::new(RwLock::new(MemPool::new())));

pub fn mem_pool() -> Arc<RwLock<MemPool>> {
    MEM_POOL.clone()
}

pub struct MemPool {
    pub watching: RwLock<HashMap<String, LitePool>>,
    pub registered: RwLock<HashMap<String, LitePool>>,
}

impl MemPool {
    fn new() -> MemPool {
        todo!()
    }

    pub fn add_if_not_exists(&self, pool: LitePool) -> Result<()> {
        let read_guard = self
            .registered
            .read()
            .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?;
        match read_guard.get(&pool.pool_address.to_string()) {
            None => {
                drop(read_guard);
                self.registered
                    .write()
                    .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?
                    .insert(pool.pool_address.to_string(), pool.clone());

                tokio::spawn(async move {
                    let mem = mem_pool();
                    mem.read().unwrap().add_to_registered(pool);
                });
            }
            Some(_) => {}
        }

        Ok(())
    }

    pub async fn add_to_registered(&self, pool: LitePool) -> Result<()> {
        let config = MeteoraDlmmPoolConfig::load_from_address(&pool.pool_address).await?;
        let mut write_guard = self
            .registered
            .write()
            .map_err(|e| anyhow::anyhow!("Write pock poisoned: {}", e))?;
        write_guard.insert(pool.pool_address.to_string(), pool);
        Ok(())
    }
}
