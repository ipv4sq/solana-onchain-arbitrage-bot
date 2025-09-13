use crate::dex::whirlpool::pool_data::WhirlpoolPoolData;
use crate::lined_err;
use crate::util::alias::{AResult, MintAddress};

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

        let sqrt_price_limit = if a_to_b {
            MIN_SQRT_PRICE_X64
        } else {
            MAX_SQRT_PRICE_X64
        };

        let swap_result = self.compute_swap_step(
            input_amount,
            self.fee_rate as u32,
            self.liquidity,
            self.sqrt_price,
            sqrt_price_limit,
            true,
            a_to_b,
        )?;

        Ok(swap_result.1)
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
        let amount_after_fee = if amount_specified_is_input {
            let fee_amount = Self::calculate_fee_amount(amount_remaining, fee_rate)?;
            amount_remaining
                .checked_sub(fee_amount)
                .ok_or_else(|| lined_err!("Fee exceeds input amount"))?
        } else {
            amount_remaining
        };

        let (amount_in, amount_out) = if a_to_b {
            let amount_out = Self::get_amount_delta_b(
                sqrt_price_current,
                sqrt_price_target,
                liquidity,
                false,
            )?;

            let capped_amount_out = amount_out.min(amount_after_fee);

            let amount_in = if amount_specified_is_input {
                amount_after_fee.min(Self::get_amount_delta_a(
                    sqrt_price_current,
                    sqrt_price_target,
                    liquidity,
                    true,
                )?)
            } else {
                Self::get_amount_delta_a(
                    sqrt_price_current,
                    sqrt_price_target,
                    liquidity,
                    true,
                )?
            };

            (amount_in, capped_amount_out)
        } else {
            let amount_out = Self::get_amount_delta_a(
                sqrt_price_current,
                sqrt_price_target,
                liquidity,
                false,
            )?;

            let capped_amount_out = amount_out.min(amount_after_fee);

            let amount_in = if amount_specified_is_input {
                amount_after_fee.min(Self::get_amount_delta_b(
                    sqrt_price_current,
                    sqrt_price_target,
                    liquidity,
                    true,
                )?)
            } else {
                Self::get_amount_delta_b(
                    sqrt_price_current,
                    sqrt_price_target,
                    liquidity,
                    true,
                )?
            };

            (amount_in, capped_amount_out)
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

        let numerator = (liquidity as u128)
            .checked_mul(sqrt_price_diff)
            .ok_or_else(|| lined_err!("Multiplication overflow in delta_a"))?
            .checked_shl(64)
            .ok_or_else(|| lined_err!("Shift overflow in delta_a"))?;

        let denominator = (sqrt_price_upper as u128)
            .checked_mul(sqrt_price_lower as u128)
            .ok_or_else(|| lined_err!("Multiplication overflow in denominator"))?;

        let mut result = numerator
            .checked_div(denominator)
            .ok_or_else(|| lined_err!("Division by zero in delta_a"))?;

        if round_up && numerator % denominator != 0 {
            result = result
                .checked_add(1)
                .ok_or_else(|| lined_err!("Addition overflow in rounding"))?;
        }

        if result > u64::MAX as u128 {
            return Err(lined_err!("Amount delta_a exceeds u64 max"));
        }

        Ok(result as u64)
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

        let product = (liquidity as u128)
            .checked_mul(sqrt_price_diff)
            .ok_or_else(|| lined_err!("Multiplication overflow in delta_b"))?;

        let result = product >> Q64_RESOLUTION;

        let final_result = if round_up && (product & ((1u128 << Q64_RESOLUTION) - 1)) > 0 {
            result
                .checked_add(1)
                .ok_or_else(|| lined_err!("Addition overflow in rounding"))?
        } else {
            result
        };

        if final_result > u64::MAX as u128 {
            return Err(lined_err!("Amount delta_b exceeds u64 max"));
        }

        Ok(final_result as u64)
    }
}
