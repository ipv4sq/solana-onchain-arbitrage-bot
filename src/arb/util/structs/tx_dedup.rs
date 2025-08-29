use parking_lot::RwLock;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TxKey {
    minor_mint: Pubkey,
    pools_hash: u64,
}

impl TxKey {
    pub fn new(minor_mint: &Pubkey, pools: &[Pubkey]) -> Self {
        let mut sorted_pools: Vec<Pubkey> = pools.to_vec();
        sorted_pools.sort();
        
        let pools_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            for pool in &sorted_pools {
                pool.hash(&mut hasher);
            }
            hasher.finish()
        };
        
        Self {
            minor_mint: *minor_mint,
            pools_hash,
        }
    }
}

pub struct TxDeduplicator {
    entries: Arc<RwLock<HashMap<TxKey, Instant>>>,
    backoff_duration: Duration,
}

impl TxDeduplicator {
    pub fn new(backoff_duration: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            backoff_duration,
        }
    }
    
    pub fn can_send(&self, minor_mint: &Pubkey, pools: &[Pubkey]) -> bool {
        let key = TxKey::new(minor_mint, pools);
        let now = Instant::now();
        
        let mut entries = self.entries.write();
        
        entries.retain(|_, &mut last_sent| {
            now.duration_since(last_sent) < self.backoff_duration * 2
        });
        
        if let Some(&last_sent) = entries.get(&key) {
            if now.duration_since(last_sent) < self.backoff_duration {
                return false;
            }
        }
        
        entries.insert(key, now);
        true
    }
    
    pub fn check_without_marking(&self, minor_mint: &Pubkey, pools: &[Pubkey]) -> bool {
        let key = TxKey::new(minor_mint, pools);
        let now = Instant::now();
        
        let entries = self.entries.read();
        
        if let Some(&last_sent) = entries.get(&key) {
            now.duration_since(last_sent) >= self.backoff_duration
        } else {
            true
        }
    }
    
    pub fn mark_sent(&self, minor_mint: &Pubkey, pools: &[Pubkey]) {
        let key = TxKey::new(minor_mint, pools);
        let now = Instant::now();
        
        let mut entries = self.entries.write();
        entries.insert(key, now);
    }
    
    pub fn size(&self) -> usize {
        self.entries.read().len()
    }
    
    pub fn clear(&self) {
        self.entries.write().clear();
    }
}

unsafe impl Send for TxDeduplicator {}
unsafe impl Sync for TxDeduplicator {}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    
    fn new_unique_pubkey() -> Pubkey {
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        Pubkey::new_from_array(bytes)
    }
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_basic_dedup() {
        let dedup = TxDeduplicator::new(Duration::from_millis(100));
        
        let mint = new_unique_pubkey();
        let pools = vec![new_unique_pubkey(), new_unique_pubkey()];
        
        assert!(dedup.can_send(&mint, &pools));
        assert!(!dedup.can_send(&mint, &pools));
        
        std::thread::sleep(Duration::from_millis(150));
        assert!(dedup.can_send(&mint, &pools));
    }
    
    #[test]
    fn test_pool_order_independence() {
        let dedup = TxDeduplicator::new(Duration::from_millis(100));
        
        let mint = new_unique_pubkey();
        let pool1 = new_unique_pubkey();
        let pool2 = new_unique_pubkey();
        
        let pools_forward = vec![pool1, pool2];
        let pools_reverse = vec![pool2, pool1];
        
        assert!(dedup.can_send(&mint, &pools_forward));
        assert!(!dedup.can_send(&mint, &pools_reverse));
    }
    
    #[test]
    fn test_different_mints() {
        let dedup = TxDeduplicator::new(Duration::from_millis(100));
        
        let mint1 = new_unique_pubkey();
        let mint2 = new_unique_pubkey();
        let pools = vec![new_unique_pubkey(), new_unique_pubkey()];
        
        assert!(dedup.can_send(&mint1, &pools));
        assert!(dedup.can_send(&mint2, &pools));
    }
    
    #[test]
    fn test_different_pools() {
        let dedup = TxDeduplicator::new(Duration::from_millis(100));
        
        let mint = new_unique_pubkey();
        let pools1 = vec![new_unique_pubkey(), new_unique_pubkey()];
        let pools2 = vec![new_unique_pubkey(), new_unique_pubkey()];
        
        assert!(dedup.can_send(&mint, &pools1));
        assert!(dedup.can_send(&mint, &pools2));
    }
    
    #[test]
    fn test_check_without_marking() {
        let dedup = TxDeduplicator::new(Duration::from_millis(100));
        
        let mint = new_unique_pubkey();
        let pools = vec![new_unique_pubkey()];
        
        assert!(dedup.check_without_marking(&mint, &pools));
        assert!(dedup.check_without_marking(&mint, &pools));
        
        dedup.mark_sent(&mint, &pools);
        assert!(!dedup.check_without_marking(&mint, &pools));
    }
}