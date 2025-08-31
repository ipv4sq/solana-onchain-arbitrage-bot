use crate::arb::database::mint_record::repository::MintRecordRepository;
use crate::arb::dex::meteora_dlmm::misc::bin_array::{
    bin_id_to_bin_array_index, get_bin_array_pda,
};
use crate::arb::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::arb::global::client::rpc::rpc_client;
use crate::arb::global::state::account_data_holder::AccountDataHolder;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;

const BIN_STEP_SCALE: u64 = 10_000;
const SCALE: u128 = 1_000_000_000_000_000_000u128;
const MAX_FEE_RATE: u64 = 100_000_000;
const FEE_PRECISION: u128 = 1_000_000_000;

#[derive(Debug, Clone, BorshDeserialize)]
struct BinArrayState {
    pub index: i64,
    pub version: u8,
    pub padding: [u8; 7],
    pub lb_pair: Pubkey,
    pub bins: [Bin; 70],
}

#[derive(Debug, Clone, Copy, BorshDeserialize)]
struct Bin {
    pub amount_x: u64,
    pub amount_y: u64,
    pub price: u128,
    pub liquidity_supply: u128,
    pub reward_per_token_stored: [u128; 2],
    pub fee_amount_x_per_token_stored: u128,
    pub fee_amount_y_per_token_stored: u128,
    pub amount_x_in: u128,
    pub amount_y_in: u128,
}

impl MeteoraDlmmPoolData {
    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
        pool_address: &PoolAddress,
    ) -> AResult<u64> {
        let swap_for_y = *from_mint == self.token_x_mint;

        let from_decimals = MintRecordRepository::get_decimal(from_mint)
            .await?
            .ok_or_else(|| anyhow::anyhow!("from mint decimals not found"))?;
        let to_decimals = MintRecordRepository::get_decimal(to_mint)
            .await?
            .ok_or_else(|| anyhow::anyhow!("to mint decimals not found"))?;

        let mut amount_in_left = input_amount;
        let mut amount_out = 0u64;
        let mut current_active_id = self.active_id;

        let base_fee_rate = self.parameters.base_factor as u128 * 10_000;
        let variable_fee_rate = self.v_parameters.volatility_accumulator as u128 * 10_000;
        let total_fee_rate = (base_fee_rate + variable_fee_rate).min(MAX_FEE_RATE as u128);

        let max_bins_to_traverse = 10;
        let mut bins_traversed = 0;

        while amount_in_left > 0 && bins_traversed < max_bins_to_traverse {
            let bin_array_index = bin_id_to_bin_array_index(current_active_id);
            let bin_array_pubkey = get_bin_array_pda(&pool_address, bin_array_index);
            let bin_array_data = AccountDataHolder::get_account_data(&bin_array_pubkey).await;

            if bin_array_data.is_none() {
                break;
            }
            let bin_array_data = bin_array_data.unwrap();

            if bin_array_data.len() < 8 {
                break;
            }

            let bin_array = match BinArrayState::try_from_slice(&bin_array_data[8..]) {
                Ok(state) => state,
                Err(_) => break,
            };

            let bin_id_within_array = current_active_id.rem_euclid(70);
            let bin = &bin_array.bins[bin_id_within_array as usize];

            let (reserve_in, reserve_out) = if swap_for_y {
                (bin.amount_x as u128, bin.amount_y as u128)
            } else {
                (bin.amount_y as u128, bin.amount_x as u128)
            };

            // For swapping, we only need reserve_out to be available
            // We're providing reserve_in from our wallet
            if reserve_out == 0 {
                if swap_for_y {
                    current_active_id -= 1;
                } else {
                    current_active_id += 1;
                }
                bins_traversed += 1;
                continue;
            }

            // Always calculate price from bin ID, don't trust the stored price
            let calculated_price = self.calculate_price_from_id(current_active_id)?;
            let stored_price = bin.price;

            let price = calculated_price; // Use calculated price

            // Calculate how much input would consume all output liquidity
            let max_amount_in = if swap_for_y {
                // X -> Y: How much X is needed to get all the Y in the bin
                // Need to account for decimal differences
                let decimal_adj = if from_decimals > to_decimals {
                    10u128.pow((from_decimals - to_decimals) as u32)
                } else {
                    1
                };
                reserve_out
                    .saturating_mul(SCALE)
                    .saturating_mul(decimal_adj)
                    .checked_div(price)
                    .unwrap_or(u64::MAX as u128)
            } else {
                // Y -> X: How much Y is needed to get all the X in the bin
                let decimal_adj = if to_decimals > from_decimals {
                    10u128.pow((to_decimals - from_decimals) as u32)
                } else {
                    1
                };
                reserve_out
                    .saturating_mul(price)
                    .checked_div(SCALE)
                    .unwrap_or(u64::MAX as u128)
                    .checked_div(decimal_adj)
                    .unwrap_or(u64::MAX as u128)
            }
            .min(u64::MAX as u128) as u64;

            let amount_to_swap = amount_in_left.min(max_amount_in);

            let amount_in_with_fee = (amount_to_swap as u128 * FEE_PRECISION)
                .checked_div(FEE_PRECISION - total_fee_rate)
                .unwrap_or(amount_to_swap as u128) as u64;

            let actual_amount_in = amount_in_with_fee.min(amount_in_left);

            let swap_amount_out = if swap_for_y {
                let decimal_adj = if from_decimals > to_decimals {
                    10u128.pow((from_decimals - to_decimals) as u32)
                } else {
                    1
                };
                let amount_out_128 = (actual_amount_in as u128)
                    .saturating_mul(price)
                    .checked_div(SCALE)
                    .unwrap_or(0)
                    .checked_div(decimal_adj)
                    .unwrap_or(0);
                amount_out_128.min(reserve_out) as u64
            } else {
                // For Y -> X swap: amount_out = amount_in / price
                let amount_out_base = (actual_amount_in as u128)
                    .saturating_mul(SCALE)
                    .checked_div(price)
                    .unwrap_or(0);

                let amount_out_128 = if to_decimals > from_decimals {
                    let decimal_adj = 10u128.pow((to_decimals - from_decimals) as u32);
                    amount_out_base.saturating_mul(decimal_adj)
                } else if from_decimals > to_decimals {
                    let decimal_adj = 10u128.pow((from_decimals - to_decimals) as u32);
                    amount_out_base.checked_div(decimal_adj).unwrap_or(0)
                } else {
                    amount_out_base
                };

                amount_out_128.min(reserve_out) as u64
            };

            amount_out = amount_out.saturating_add(swap_amount_out);
            amount_in_left = amount_in_left.saturating_sub(actual_amount_in);

            if swap_for_y {
                current_active_id -= 1; // Move to lower bins for X->Y swap (bins with Y liquidity)
            } else {
                current_active_id += 1; // Move to higher bins for Y->X swap (bins with X liquidity)
            }
            bins_traversed += 1;
        }

        // The price in DLMM is stored with 18 decimals of precision
        // We don't need to adjust for decimals here as the calculation already handles it

        Ok(amount_out)
    }

    fn calculate_price_from_id(&self, bin_id: i32) -> AResult<u128> {
        // Price = (1 + bin_step / 10000) ^ bin_id
        // Where bin_step is in basis points
        let bin_step = self.bin_step as f64 / 10_000.0;
        let price_float = (1.0 + bin_step).powi(bin_id);

        // Convert to u128 with 18 decimals of precision
        let price_u128 = (price_float * SCALE as f64) as u128;

        Ok(price_u128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::dex::interface::PoolDataLoader;
    use crate::arb::global::client::db::must_init_db;
    use crate::arb::global::client::rpc::rpc_client;
    use crate::arb::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_trump_usdc_swap() {
        must_init_db().await;

        let pool_address = "9d9mb8kooFfaD3SctgZtkxQypkshx6ezhbKio89ixyy2".to_pubkey();
        let trump_mint = "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN".to_pubkey();
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();
        let one_trump = 1_000_000;
        let account_data = rpc_client().get_account_data(&pool_address).await.unwrap();
        let pool_data = MeteoraDlmmPoolData::load_data(&account_data).unwrap();

        println!("\n=== TRUMP/USDC Swap Test ===");
        println!("Pool: {}", pool_address);
        println!("Active Bin ID: {}", pool_data.active_id);
        println!("Bin Step: {} bps", pool_data.bin_step);

        let usdc_out = pool_data
            .get_amount_out(one_trump, &trump_mint, &usdc_mint, &pool_address)
            .await
            .unwrap();

        let trump_decimals = 6;
        let usdc_decimals = 6;
        let trump_amount = one_trump as f64 / 10_f64.powi(trump_decimals);
        let usdc_amount = usdc_out as f64 / 10_f64.powi(usdc_decimals);

        println!("\n=== Swap Result ===");
        println!("Input: {} TRUMP ({} lamports)", trump_amount, one_trump);
        println!("Output: {} USDC ({} lamports)", usdc_amount, usdc_out);
        println!("\n1 TRUMP = {} USDC", usdc_amount / trump_amount);
    }
}
