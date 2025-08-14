use crate::arb::constant::known_pool_program::METEORA_DLMM_PROGRAM;
use crate::arb::pool::interface::PoolAccountDataLoader;
use crate::arb::pool::meteora_dlmm::data_type::{
    ProtocolFee, RewardInfo, StaticParameters, VariableParameters,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct MeteoraDlmmAccountData {
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

impl PoolAccountDataLoader for MeteoraDlmmAccountData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        MeteoraDlmmAccountData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn get_base_mint(&self) -> Pubkey {
        self.token_x_mint
    }

    fn get_quote_mint(&self) -> Pubkey {
        self.token_y_mint
    }

    fn get_base_vault(&self) -> Pubkey {
        self.reserve_x
    }

    fn get_quote_vault(&self) -> Pubkey {
        self.reserve_y
    }
}

impl MeteoraDlmmAccountData {
    pub fn calculate_bin_arrays_for_swap(
        &self,
        pool: &Pubkey,
        active_bin_id: i32,
        amount_in: u64,
        is_a_to_b: bool,
    ) -> Vec<Pubkey> {
        const BINS_PER_ARRAY: i32 = 70;

        // Get the starting bin array index
        let current_array_index = Self::bin_id_to_bin_array_index(active_bin_id);

        // Determine range of bin arrays based on amount
        // For 1.485 SOL (1485000000), the transaction used 5 bin arrays
        let (start_offset, end_offset) = if amount_in >= 1_000_000_000 {
            // Large swap (>= 1 SOL): use wider range
            // This matches the 5 arrays used in the example transaction
            (-2, 2)
        } else if amount_in >= 100_000_000 {
            // Medium swap (>= 0.1 SOL): use moderate range
            (-1, 2)
        } else {
            // Small swap: use minimal range
            (-1, 1)
        };

        // Build the list of bin arrays
        let mut bin_arrays = Vec::new();
        let mut seen_indices = std::collections::HashSet::new();

        // Add arrays in the range
        for offset in start_offset..=end_offset {
            let array_index = current_array_index + offset;
            if seen_indices.insert(array_index) {
                bin_arrays.push(Self::get_bin_array_pda(pool, array_index));
            }
        }

        // Ensure we have at least the active bin array
        if bin_arrays.is_empty() {
            bin_arrays.push(Self::get_bin_array_pda(pool, current_array_index));
        }

        bin_arrays
    }

    pub fn bin_id_to_bin_array_index(bin_id: i32) -> i32 {
        const BINS_PER_ARRAY: i32 = 70;

        let idx = bin_id / BINS_PER_ARRAY;
        let rem = bin_id % BINS_PER_ARRAY;

        // For negative bin IDs with remainder, we need to go one array lower
        if bin_id < 0 && rem != 0 {
            idx - 1
        } else {
            idx
        }
    }

    pub fn get_bin_array_pda(pool: &Pubkey, bin_array_index: i32) -> Pubkey {
        // Use i64 for the PDA derivation as per Meteora's implementation
        let index_bytes = (bin_array_index as i64).to_le_bytes();
        Pubkey::find_program_address(
            &[b"bin_array", pool.as_ref(), &index_bytes],
            &*METEORA_DLMM_PROGRAM,
        )
        .0
    }
}
