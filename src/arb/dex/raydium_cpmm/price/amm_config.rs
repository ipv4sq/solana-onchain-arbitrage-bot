use crate::arb::global::client::rpc::rpc_client;
use crate::arb::util::alias::AResult;
use crate::arb::util::structs::cache_type::CacheType;
use crate::arb::util::structs::persistent_cache::PersistentCache;
use crate::return_error;
use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::time::Duration;

#[allow(non_upper_case_globals)]
static AmmConfigCache: Lazy<PersistentCache<Pubkey, CpmmAmmConfig>> = Lazy::new(|| {
    PersistentCache::new(
        CacheType::Custom("RaydiumAMMConfig".to_string()),
        200,
        Duration::from_secs(60 * 24 * 30),
        |x: &Pubkey| {
            let x = x.clone();
            async move { fetch_amm_config(&x).await }
        },
    )
});

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct CpmmAmmConfig {
    pub bump: u8,
    pub disable_create_pool: bool,
    pub index: u16,
    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub create_pool_fee: u64,
    pub protocol_owner: Pubkey,
    pub fund_owner: Pubkey,
    pub creator_fee_rate: u64,
    pub padding: [u64; 15],
}

impl CpmmAmmConfig {
    pub async fn get(address: &Pubkey) -> Option<Self> {
        AmmConfigCache.get(address).await
    }

    pub async fn load_data(data: &[u8]) -> AResult<Self> {
        if data.len() < 8 {
            return_error!("data length is {} bytes, not amm config", data.len());
        }
        let config: CpmmAmmConfig = BorshDeserialize::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse AMM config data: {}", e))?;
        Ok(config)
    }
}

async fn fetch_amm_config(config_address: &Pubkey) -> Option<CpmmAmmConfig> {
    let account = rpc_client()
        .get_account(config_address)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to fetch AMM config account {}: {}",
                config_address,
                e
            )
        })
        .ok()?;
    let config = CpmmAmmConfig::load_data(&account.data).await.ok()?;
    Some(config)
}
