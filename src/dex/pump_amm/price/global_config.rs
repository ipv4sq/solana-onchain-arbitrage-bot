use crate::dex::pump_amm::PUMP_GLOBAL_CONFIG;
use crate::f;
use crate::sdk::solana_rpc::buffered_get_account::buffered_get_account;
use crate::sdk::solana_rpc::proxy;
use crate::util::alias::AResult;
use crate::util::cache::persistent_cache::PersistentCache;
use crate::util::serde_helpers;
use crate::util::structs::cache_type::CacheType;
use crate::util::traits::option::OptionExt;
use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub admin: Pubkey,                            // 32 bytes
    pub lp_fee_basis_points: u64,                 // 8 bytes
    pub protocol_fee_basis_points: u64,           // 8 bytes
    pub disable_flags: u8,                        // 1 byte
    pub protocol_fee_recipients: [Pubkey; 8],     // 256 bytes (32 * 8)
    pub coin_creator_fee_basis_points: u64,       // 8 bytes
    pub admin_set_coin_creator_authority: Pubkey, // 32 bytes
    // Total so far: 32 + 8 + 8 + 1 + 256 + 8 + 32 = 345 bytes
    // Account has 512 bytes total (minus 8 byte discriminator = 504 bytes)
    // Padding needed: 504 - 345 = 159 bytes
    #[serde(with = "serde_helpers::byte_array_159")]
    pub _padding: [u8; 159],
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct FeeConfig {
    pub bump: u8,
    pub admin: Pubkey,
    pub flat_fees: Fees,
    pub fee_tiers: Vec<FeeTier>,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct FeeTier {
    pub market_cap_lamports_threshold: u128,
    pub fees: Fees,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Fees {
    pub lp_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub creator_fee_bps: u64,
}

impl GlobalConfig {
    async fn fetch(address: &Pubkey) -> AResult<Self> {
        let account = proxy::get_account(address).await?;

        if account.data.len() < 8 {
            return Err(anyhow::anyhow!(
                "GlobalConfig account data too short, expected at least 8 bytes"
            ));
        }

        GlobalConfig::try_from_slice(&account.data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to deserialize GlobalConfig: {}", e))
    }

    pub async fn get() -> AResult<GlobalConfig> {
        GLOBAL_CONFIG_CACHE
            .get(&PUMP_GLOBAL_CONFIG)
            .await
            .or_err(f!("Failed to fetch GlobalConfig from cache"))
    }
}

impl FeeConfig {
    pub async fn fetch(address: &Pubkey) -> AResult<Self> {
        let account = buffered_get_account(address).await?;

        if account.data.len() < 8 {
            return Err(anyhow::anyhow!(
                "FeeConfig account data too short, expected at least 8 bytes"
            ));
        }

        FeeConfig::try_from_slice(&account.data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to deserialize FeeConfig: {}", e))
    }
}

static GLOBAL_CONFIG_CACHE: Lazy<PersistentCache<Pubkey, GlobalConfig>> = Lazy::new(|| {
    PersistentCache::new(
        CacheType::Custom("PumpGlobalConfig".to_string()),
        100,
        60 * 60 * 24 * 7, // 7 days TTL in seconds
        |address: Pubkey| async move { GlobalConfig::fetch(&address).await.ok() },
    )
});

pub fn compute_fees_bps(
    global_config: &GlobalConfig,
    fee_config: Option<&FeeConfig>,
    is_pump_pool: bool,
    market_cap: u128,
) -> Fees {
    if let Some(fee_config) = fee_config {
        if is_pump_pool {
            calculate_fee_tier(&fee_config.fee_tiers, market_cap)
        } else {
            fee_config.flat_fees.clone()
        }
    } else {
        Fees {
            lp_fee_bps: global_config.lp_fee_basis_points,
            protocol_fee_bps: global_config.protocol_fee_basis_points,
            creator_fee_bps: global_config.coin_creator_fee_basis_points,
        }
    }
}

fn calculate_fee_tier(fee_tiers: &[FeeTier], market_cap: u128) -> Fees {
    if fee_tiers.is_empty() {
        return Fees {
            lp_fee_bps: 0,
            protocol_fee_bps: 0,
            creator_fee_bps: 0,
        };
    }

    let first_tier = &fee_tiers[0];
    if market_cap < first_tier.market_cap_lamports_threshold {
        return first_tier.fees.clone();
    }

    for tier in fee_tiers.iter().rev() {
        if market_cap >= tier.market_cap_lamports_threshold {
            return tier.fees.clone();
        }
    }

    first_tier.fees.clone()
}
