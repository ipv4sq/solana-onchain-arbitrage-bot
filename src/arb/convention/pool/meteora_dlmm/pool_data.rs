use crate::arb::convention::pool::interface::PoolDataLoader;
use crate::arb::convention::pool::meteora_dlmm::pool_data_type::{
    ProtocolFee, RewardInfo, StaticParameters, VariableParameters,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct MeteoraDlmmPoolData {
    pub parameters: StaticParameters,
    pub v_parameters: VariableParameters,
    pub bump_seed: [u8; 1],
    pub bin_step_seed: [u8; 2],
    pub pair_type: u8,
    pub active_id: i32,
    pub bin_step: u16,
    pub status: u8,
    pub require_base_factor_seed: u8,
    pub base_factor_seed: [u8; 2],
    pub activation_type: u8,
    pub creator_pool_on_off_control: u8,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub protocol_fee: ProtocolFee,
    pub _padding_1: [u8; 32],
    pub reward_infos: [RewardInfo; 2],
    pub oracle: Pubkey,
    pub bin_array_bitmap: [u64; 16],
    pub last_updated_at: i64,
    pub _padding_2: [u8; 32],
    pub pre_activation_swap_address: Pubkey,
    pub base_key: Pubkey,
    pub activation_point: u64,
    pub pre_activation_duration: u64,
    pub _padding_3: [u8; 8],
    pub _padding_4: u64,
    pub creator: Pubkey,
    pub token_mint_x_program_flag: u8,
    pub token_mint_y_program_flag: u8,
    pub _reserved: [u8; 22],
}

impl PoolDataLoader for MeteoraDlmmPoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        MeteoraDlmmPoolData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn base_mint(&self) -> Pubkey {
        self.token_x_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_y_mint
    }

    fn base_vault(&self) -> Pubkey {
        self.reserve_x
    }

    fn quote_vault(&self) -> Pubkey {
        self.reserve_y
    }
}
