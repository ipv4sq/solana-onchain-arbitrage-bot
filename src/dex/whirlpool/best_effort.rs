use crate::dex::whirlpool::pool_data::WhirlpoolPoolData;
use crate::lined_err;
use crate::util::alias::{AResult, MintAddress};
use ethnum::U256;

const Q64_RESOLUTION: u32 = 64;
const FEE_RATE_MUL_VALUE: u128 = 1_000_000;
const MIN_SQRT_PRICE_X64: u128 = 4295048016;
const MAX_SQRT_PRICE_X64: u128 = 79226673515401279992447579055;

impl WhirlpoolPoolData {
    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        if input_amount == 0 {
            return Ok(0);
        }

        let a_to_b = if from_mint == &self.token_mint_a {
            if to_mint != &self.token_mint_b {
                return Err(lined_err!("Invalid mint pair for swap"));
            }
            true
        } else if from_mint == &self.token_mint_b {
            if to_mint != &self.token_mint_a {
                return Err(lined_err!("Invalid mint pair for swap"));
            }
            false
        } else {
            return Err(lined_err!("From mint not found in pool"));
        };

        // Use a more reasonable price limit for single-step calculation
        // This approximates swapping within current tick range
        let sqrt_price_limit = if a_to_b {
            // For A->B, price decreases. Use 1% slippage
            self.sqrt_price.saturating_mul(99) / 100
        } else {
            // For B->A, price increases. Use 1% slippage
            self.sqrt_price.saturating_mul(101) / 100
        };

        // Use compute_swap_step for accurate calculation
        let swap_result = self.compute_swap_step(
            input_amount,
            self.fee_rate as u32,
            self.liquidity,
            self.sqrt_price,
            sqrt_price_limit,
            true, // amount_specified_is_input
            a_to_b,
        )?;

        Ok(swap_result.1) // Return amount_out
    }

    fn compute_swap_step(
        &self,
        amount_remaining: u64,
        fee_rate: u32,
        liquidity: u128,
        sqrt_price_current: u128,
        sqrt_price_target: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> AResult<(u64, u64, u64)> {
        // Apply fee for input amounts
        let amount_calculated = if amount_specified_is_input {
            let fee_amount = Self::calculate_fee_amount(amount_remaining, fee_rate)?;
            amount_remaining
                .checked_sub(fee_amount)
                .ok_or_else(|| lined_err!("Fee exceeds input amount"))?
        } else {
            amount_remaining
        };

        // Calculate the maximum amount that can be swapped at the target price
        let amount_fixed_delta = if a_to_b == amount_specified_is_input {
            Self::get_amount_delta_a(sqrt_price_current, sqrt_price_target, liquidity, a_to_b)?
        } else {
            Self::get_amount_delta_b(sqrt_price_current, sqrt_price_target, liquidity, !a_to_b)?
        };

        // Check if we can reach the target price with the amount we have
        let is_max_swap = amount_calculated >= amount_fixed_delta;

        let (amount_in, amount_out) = if is_max_swap {
            // We can reach the target price
            let amount_unfixed = if a_to_b == amount_specified_is_input {
                Self::get_amount_delta_b(sqrt_price_current, sqrt_price_target, liquidity, false)?
            } else {
                Self::get_amount_delta_a(sqrt_price_current, sqrt_price_target, liquidity, false)?
            };

            if amount_specified_is_input {
                (amount_fixed_delta, amount_unfixed)
            } else {
                (amount_unfixed, amount_fixed_delta)
            }
        } else {
            // We can't reach the target price, calculate partial swap
            if amount_specified_is_input {
                // Calculate output for given input
                if a_to_b {
                    // Calculate output B for input A
                    let output = Self::calculate_partial_swap_output(
                        amount_calculated,
                        sqrt_price_current,
                        liquidity,
                        true,
                    )?;
                    (amount_calculated, output)
                } else {
                    // Calculate output A for input B
                    let output = Self::calculate_partial_swap_output(
                        amount_calculated,
                        sqrt_price_current,
                        liquidity,
                        false,
                    )?;
                    (amount_calculated, output)
                }
            } else {
                // Calculate input for given output (exact out)
                if a_to_b {
                    let input = Self::calculate_partial_swap_input(
                        amount_calculated,
                        sqrt_price_current,
                        liquidity,
                        true,
                    )?;
                    (input, amount_calculated)
                } else {
                    let input = Self::calculate_partial_swap_input(
                        amount_calculated,
                        sqrt_price_current,
                        liquidity,
                        false,
                    )?;
                    (input, amount_calculated)
                }
            }
        };

        let fee_amount = if amount_specified_is_input {
            Self::calculate_fee_amount(amount_in, fee_rate)?
        } else {
            0
        };

        Ok((amount_in, amount_out, fee_amount))
    }

    fn calculate_fee_amount(amount: u64, fee_rate: u32) -> AResult<u64> {
        let fee = (amount as u128)
            .checked_mul(fee_rate as u128)
            .ok_or_else(|| lined_err!("Fee calculation overflow"))?
            .checked_div(FEE_RATE_MUL_VALUE)
            .ok_or_else(|| lined_err!("Fee division error"))?;

        if fee > u64::MAX as u128 {
            return Err(lined_err!("Fee exceeds u64 max"));
        }

        Ok(fee as u64)
    }

    fn get_amount_delta_a(
        sqrt_price_0: u128,
        sqrt_price_1: u128,
        liquidity: u128,
        round_up: bool,
    ) -> AResult<u64> {
        let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_0 > sqrt_price_1 {
            (sqrt_price_1, sqrt_price_0)
        } else {
            (sqrt_price_0, sqrt_price_1)
        };

        if sqrt_price_lower == sqrt_price_upper {
            return Ok(0);
        }

        let sqrt_price_diff = sqrt_price_upper - sqrt_price_lower;

        // Use U256 for overflow-safe calculation
        let numerator: U256 = U256::from(liquidity)
            .checked_mul(U256::from(sqrt_price_diff))
            .ok_or_else(|| lined_err!("Multiplication overflow in delta_a"))?
            .checked_shl(64)
            .ok_or_else(|| lined_err!("Shift overflow in delta_a"))?;

        let denominator: U256 = U256::from(sqrt_price_lower)
            .checked_mul(U256::from(sqrt_price_upper))
            .ok_or_else(|| lined_err!("Multiplication overflow in denominator"))?;

        let quotient = numerator / denominator;
        let remainder = numerator % denominator;

        let result = if round_up && remainder != U256::ZERO {
            quotient + U256::ONE
        } else {
            quotient
        };

        result.try_into().map_err(|_| lined_err!("Amount delta_a exceeds u64 max"))
    }

    fn calculate_partial_swap_output(
        amount_in: u64,
        sqrt_price_current: u128,
        liquidity: u128,
        a_to_b: bool,
    ) -> AResult<u64> {
        // For partial swaps, we need to calculate the new sqrt price after consuming amount_in
        // Then calculate the output from the price change

        if liquidity == 0 {
            return Ok(0);
        }

        // Calculate how much the price moves for the given input
        // This is a simplified approximation
        let price_impact = U256::from(amount_in) * U256::from(sqrt_price_current) / U256::from(liquidity);

        // For a_to_b, price decreases; for b_to_a, price increases
        let new_sqrt_price = if a_to_b {
            U256::from(sqrt_price_current).saturating_sub(price_impact >> 64)
        } else {
            U256::from(sqrt_price_current).saturating_add(price_impact >> 64)
        };

        // Calculate output from price change
        if a_to_b {
            Self::get_amount_delta_b(
                sqrt_price_current,
                new_sqrt_price.as_u128(),
                liquidity,
                false,
            )
        } else {
            Self::get_amount_delta_a(
                sqrt_price_current,
                new_sqrt_price.as_u128(),
                liquidity,
                false,
            )
        }
    }

    fn calculate_partial_swap_input(
        amount_out: u64,
        sqrt_price_current: u128,
        liquidity: u128,
        a_to_b: bool,
    ) -> AResult<u64> {
        // For exact output, calculate required input
        // This is the inverse of calculate_partial_swap_output

        if liquidity == 0 {
            return Ok(u64::MAX);
        }

        // Simplified approximation for exact output
        let price_impact = U256::from(amount_out) * U256::from(sqrt_price_current) / U256::from(liquidity);

        let new_sqrt_price = if a_to_b {
            U256::from(sqrt_price_current).saturating_sub(price_impact >> 64)
        } else {
            U256::from(sqrt_price_current).saturating_add(price_impact >> 64)
        };

        // Calculate input required for this price change
        if a_to_b {
            Self::get_amount_delta_a(
                sqrt_price_current,
                new_sqrt_price.as_u128(),
                liquidity,
                true,
            )
        } else {
            Self::get_amount_delta_b(
                sqrt_price_current,
                new_sqrt_price.as_u128(),
                liquidity,
                true,
            )
        }
    }

    fn get_amount_delta_b(
        sqrt_price_0: u128,
        sqrt_price_1: u128,
        liquidity: u128,
        round_up: bool,
    ) -> AResult<u64> {
        let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_0 > sqrt_price_1 {
            (sqrt_price_1, sqrt_price_0)
        } else {
            (sqrt_price_0, sqrt_price_1)
        };

        let sqrt_price_diff = sqrt_price_upper - sqrt_price_lower;

        // Use U256 for overflow-safe calculation
        let product: U256 = U256::from(liquidity)
            .checked_mul(U256::from(sqrt_price_diff))
            .ok_or_else(|| lined_err!("Multiplication overflow in delta_b"))?;

        let quotient = product >> Q64_RESOLUTION;
        let remainder_mask = U256::from(u64::MAX);

        let should_round = round_up && (product & remainder_mask) > U256::ZERO;

        let result = if should_round {
            quotient + U256::ONE
        } else {
            quotient
        };

        result.try_into().map_err(|_| lined_err!("Amount delta_b exceeds u64 max"))
    }
}
