use crate::arb::util::serde_helpers;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
#[repr(C)]
pub struct BaseFeeStruct {
    pub cliff_fee_numerator: u64,
    pub fee_scheduler_mode: u8,
    pub padding_0: [u8; 7], // Changed from 5 to 7 to align to 8 bytes
    pub number_of_period: u64,
    pub period_frequency: u64,
    pub reduction_factor: u64,
    // Removed padding_1 as it's not in the actual data
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
#[repr(C)]
pub struct DynamicFeeStruct {
    pub initialized: u8,
    pub padding: [u8; 7],
    pub max_volatility_accumulator: u64,
    pub variable_fee_control: u32,
    pub bin_step: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub last_update_timestamp: u64,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub bin_step_u128: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub sqrt_price_reference: u128,
    pub volatility_accumulator: u64,
    pub volatility_reference: u64,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
#[repr(C)]
pub struct PoolFeesStruct {
    pub base_fee: BaseFeeStruct,
    pub protocol_fee_percent: u8,
    pub partner_fee_percent: u8,
    pub referral_fee_percent: u8,
    pub padding_0: [u8; 5],
    pub dynamic_fee: DynamicFeeStruct,
    pub padding_1: [u64; 2],
    pub _extra_padding: [u8; 12], // 12 bytes to reach exactly 160 bytes total
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
#[repr(C)]
pub struct PoolMetrics {
    #[serde(with = "serde_helpers::u128_as_string")]
    pub total_lp_a_fee: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub total_lp_b_fee: u128,
    pub total_protocol_a_fee: u64,
    pub total_protocol_b_fee: u64,
    pub total_partner_a_fee: u64,
    pub total_partner_b_fee: u64,
    pub total_position: u64,
    pub padding: u64,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize)]
#[repr(C)]
pub struct RewardInfo {
    pub initialized: u8,
    pub reward_token_flag: u8,
    pub _padding_0: [u8; 6],
    pub _padding_1: [u8; 8],
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub reward_duration: u64,
    pub reward_duration_end: u64,
    pub reward_rate: u64,
    pub reward_per_token_stored: [u8; 32],
    pub last_update_time: u64,
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}
