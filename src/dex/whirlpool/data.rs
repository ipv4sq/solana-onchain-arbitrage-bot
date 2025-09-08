use crate::dex::interface::PoolDataLoader;
use crate::global::constant::pool_program::PoolProgram;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct WhirlpoolPoolData {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub fee_tier_index_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    pub fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_growth_global_b: u128,
    pub reward_last_updated_timestamp: u64,
    pub reward_infos: [WhirlpoolRewardInfo; 3],
}

impl PoolDataLoader for WhirlpoolPoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        // Whirlpool accounts always have an 8-byte discriminator at the beginning
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        WhirlpoolPoolData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn base_mint(&self) -> Pubkey {
        self.token_mint_a
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_mint_b
    }

    fn base_vault(&self) -> Pubkey {
        self.token_vault_a
    }

    fn quote_vault(&self) -> Pubkey {
        self.token_vault_b
    }
}

impl WhirlpoolPoolData {
    pub(crate) fn get_oracle(pool: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"oracle", pool.as_ref()], &PoolProgram::WHIRLPOOL).0
    }

    pub(crate) fn get_tick_arrays(&self, pool: &Pubkey) -> Vec<Pubkey> {
        const TICK_ARRAY_SIZE: i32 = 88;

        let tick_spacing = self.tick_spacing as i32;
        let current_tick = self.tick_current_index;
        let num_ticks_in_array = TICK_ARRAY_SIZE * tick_spacing;

        // Calculate start index for current tick array
        let current_start = if current_tick < 0 && current_tick % num_ticks_in_array != 0 {
            current_tick - (current_tick % num_ticks_in_array) - num_ticks_in_array
        } else {
            current_tick - (current_tick % num_ticks_in_array)
        };

        // Get tick arrays for both directions (previous, current, next)
        let prev_start = current_start - num_ticks_in_array;
        let next_start = current_start + num_ticks_in_array;

        vec![
            Self::get_tick_array_pda(pool, prev_start),
            Self::get_tick_array_pda(pool, current_start),
            Self::get_tick_array_pda(pool, next_start),
        ]
    }

    fn get_tick_array_pda(pool: &Pubkey, start_tick_index: i32) -> Pubkey {
        let start_tick_str = start_tick_index.to_string();
        Pubkey::find_program_address(
            &[b"tick_array", pool.as_ref(), start_tick_str.as_bytes()],
            &PoolProgram::WHIRLPOOL,
        )
        .0
    }
}

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub emissions_per_second_x64: u128,
    pub growth_global_x64: u128,
}
