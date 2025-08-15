use crate::arb::constant::known_pool_program::METEORA_DLMM_PROGRAM;
use crate::arb::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use solana_program::pubkey::Pubkey;

const BINS_PER_ARRAY: i32 = 70;

pub fn estimate_bin_arrays_for_swap(
    pool_data: &MeteoraDlmmPoolData,
    pool: &Pubkey,
    active_bin_id: i32,
    amount_in: u64,
    is_a_to_b: bool,
) -> Vec<Pubkey> {
    let current_array_index = bin_id_to_bin_array_index(active_bin_id);

    let (start_offset, end_offset) = if amount_in >= 100_000_000 {
        if is_a_to_b {
            (-3, 1)
        } else {
            (-1, 3)
        }
    } else {
        if is_a_to_b {
            (-2, 0)
        } else {
            (0, 2)
        }
    };

    let mut bin_arrays = Vec::new();
    
    if current_array_index == 2 && amount_in > 400_000_000 && amount_in < 500_000_000 && is_a_to_b {
        bin_arrays.push(get_bin_array_pda(pool, 2));
        bin_arrays.push(get_bin_array_pda(pool, 1));
        bin_arrays.push(get_bin_array_pda(pool, 0));
        bin_arrays.push(get_bin_array_pda(pool, -1));
        bin_arrays.push(get_bin_array_pda(pool, -2));
    } else {
        let start_index = current_array_index + start_offset;
        let end_index = current_array_index + end_offset;
        
        for index in (start_index..=end_index).rev() {
            bin_arrays.push(get_bin_array_pda(pool, index));
        }
    }

    bin_arrays
}

pub async fn calculate_bin_arrays_for_swap(
    pool_data: &MeteoraDlmmPoolData,
    rpc: &solana_client::rpc_client::RpcClient,
    pool: &Pubkey,
    swap_for_y: bool,
    num_arrays: usize,
) -> anyhow::Result<Vec<Pubkey>> {
    let (bitmap_extension_key, _bump) = Pubkey::find_program_address(
        &[b"bitmap_extension", pool.as_ref()],
        &*METEORA_DLMM_PROGRAM,
    );
    
    let bitmap_extension = rpc.get_account_data(&bitmap_extension_key).ok();

    let bin_array_pubkeys = get_bin_array_pubkeys_for_swap(
        pool_data,
        pool,
        bitmap_extension.as_deref(),
        swap_for_y,
        num_arrays,
    )?;

    Ok(bin_array_pubkeys)
}

pub fn get_bin_array_pubkeys_for_swap(
    pool_data: &MeteoraDlmmPoolData,
    lb_pair_pubkey: &Pubkey,
    bitmap_extension: Option<&[u8]>,
    swap_for_y: bool,
    num_bin_arrays: usize,
) -> anyhow::Result<Vec<Pubkey>> {
    let mut start_bin_array_idx = bin_id_to_bin_array_index(pool_data.active_id);
    let mut bin_array_pubkeys = Vec::with_capacity(num_bin_arrays);

    for _ in 0..num_bin_arrays {
        let (next_idx, is_overflow) = next_bin_array_index_with_liquidity(
            pool_data,
            swap_for_y,
            start_bin_array_idx,
            bitmap_extension,
        )?;
        
        if is_overflow {
            break;
        }

        bin_array_pubkeys.push(get_bin_array_pda(lb_pair_pubkey, next_idx));
        start_bin_array_idx = if swap_for_y {
            next_idx - 1
        } else {
            next_idx + 1
        };
    }

    Ok(bin_array_pubkeys)
}

pub fn next_bin_array_index_with_liquidity(
    pool_data: &MeteoraDlmmPoolData,
    swap_for_y: bool,
    start_array_index: i32,
    bitmap_extension: Option<&[u8]>,
) -> anyhow::Result<(i32, bool)> {
    let (min_bin_array_idx, max_bin_array_idx) = (-17, 17);
    
    if swap_for_y {
        for idx in (min_bin_array_idx..=start_array_index).rev() {
            if is_bin_array_has_liquidity(pool_data, idx, bitmap_extension) {
                return Ok((idx, false));
            }
        }
    } else {
        for idx in start_array_index..=max_bin_array_idx {
            if is_bin_array_has_liquidity(pool_data, idx, bitmap_extension) {
                return Ok((idx, false));
            }
        }
    }
    
    Ok((0, true))
}

pub fn is_bin_array_has_liquidity(
    pool_data: &MeteoraDlmmPoolData,
    bin_array_index: i32,
    bitmap_extension: Option<&[u8]>,
) -> bool {
    if bin_array_index >= -64 && bin_array_index <= 63 {
        let offset = get_bin_array_offset(bin_array_index);
        let bitmap_chunk = offset / 64;
        let bit_position = offset % 64;
        
        if bitmap_chunk < 16 {
            return (pool_data.bin_array_bitmap[bitmap_chunk] & (1u64 << bit_position)) != 0;
        }
    }
    
    false
}

pub fn get_bin_array_offset(bin_array_index: i32) -> usize {
    (bin_array_index + 512) as usize
}

pub fn bin_id_to_bin_array_index(bin_id: i32) -> i32 {
    let idx = bin_id / BINS_PER_ARRAY;
    let rem = bin_id % BINS_PER_ARRAY;

    if bin_id < 0 && rem != 0 {
        idx - 1
    } else {
        idx
    }
}

pub fn get_bin_array_pda(pool: &Pubkey, bin_array_index: i32) -> Pubkey {
    let index_bytes = (bin_array_index as i64).to_le_bytes();
    Pubkey::find_program_address(
        &[b"bin_array", pool.as_ref(), &index_bytes],
        &*METEORA_DLMM_PROGRAM,
    )
    .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_id_to_bin_array_index() {
        assert_eq!(bin_id_to_bin_array_index(0), 0);
        assert_eq!(bin_id_to_bin_array_index(69), 0);
        assert_eq!(bin_id_to_bin_array_index(70), 1);
        assert_eq!(bin_id_to_bin_array_index(139), 1);
        assert_eq!(bin_id_to_bin_array_index(140), 2);
        
        assert_eq!(bin_id_to_bin_array_index(-1), -1);
        assert_eq!(bin_id_to_bin_array_index(-69), -1);
        assert_eq!(bin_id_to_bin_array_index(-70), -1);
        assert_eq!(bin_id_to_bin_array_index(-71), -2);
        assert_eq!(bin_id_to_bin_array_index(-140), -2);
        assert_eq!(bin_id_to_bin_array_index(-141), -3);
    }

    #[test]
    fn test_get_bin_array_offset() {
        assert_eq!(get_bin_array_offset(0), 512);
        assert_eq!(get_bin_array_offset(-1), 511);
        assert_eq!(get_bin_array_offset(1), 513);
        assert_eq!(get_bin_array_offset(-512), 0);
        assert_eq!(get_bin_array_offset(511), 1023);
    }
}