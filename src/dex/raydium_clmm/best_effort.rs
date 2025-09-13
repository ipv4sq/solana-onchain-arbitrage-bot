use crate::dex::raydium_clmm::pool_data::RaydiumClmmPoolData;
use crate::util::alias::{AResult, MintAddress};

const FEE_RATE_DENOMINATOR_VALUE: u32 = 1_000_000;
const Q64: u128 = 1 << 64;

impl RaydiumClmmPoolData {
    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        let zero_for_one = from_mint == &self.token_mint_0;

        if zero_for_one && to_mint != &self.token_mint_1 {
            return Err(anyhow::anyhow!("Invalid mint pair for swap"));
        }
        if !zero_for_one && from_mint != &self.token_mint_1 {
            return Err(anyhow::anyhow!("Invalid mint pair for swap"));
        }

        if self.liquidity == 0 {
            return Ok(0);
        }

        let fee_rate = 600;

        let amount_in_less_fee = (input_amount as u128)
            .checked_mul((FEE_RATE_DENOMINATOR_VALUE - fee_rate) as u128)
            .ok_or_else(|| anyhow::anyhow!("Overflow in fee calculation"))?
            .checked_div(FEE_RATE_DENOMINATOR_VALUE as u128)
            .ok_or_else(|| anyhow::anyhow!("Division by zero"))?;

        if amount_in_less_fee == 0 {
            return Ok(0);
        }

        let sqrt_price_next = self.calculate_next_sqrt_price(
            amount_in_less_fee as u64,
            zero_for_one
        )?;

        let amount_out = if zero_for_one {
            self.calculate_amount_1_delta(
                self.sqrt_price_x64,
                sqrt_price_next,
                self.liquidity,
                false
            )?
        } else {
            self.calculate_amount_0_delta(
                sqrt_price_next,
                self.sqrt_price_x64,
                self.liquidity,
                false
            )?
        };

        Ok(amount_out)
    }

    fn calculate_next_sqrt_price(
        &self,
        amount_in: u64,
        zero_for_one: bool,
    ) -> AResult<u128> {
        if zero_for_one {
            // Calculate: sqrt_price_next = (L * sqrt_price) / (L + amount_in * sqrt_price / Q64)
            // Rearranged to avoid overflow: sqrt_price_next = sqrt_price / (1 + amount_in / L * sqrt_price / Q64)

            // First calculate amount_in * sqrt_price / Q64 safely
            let amount_scaled = ((amount_in as u128) * (self.sqrt_price_x64 / 1_000_000)) / (Q64 / 1_000_000);

            let denominator = (self.liquidity as u128)
                .checked_add(amount_scaled)
                .ok_or_else(|| anyhow::anyhow!("Overflow in denominator"))?;

            if denominator == 0 {
                return Err(anyhow::anyhow!("Division by zero"));
            }

            // Calculate (L * sqrt_price) / denominator
            // Split calculation to avoid overflow
            let sqrt_price_scaled = self.sqrt_price_x64 / 1_000;
            let liquidity_scaled = self.liquidity / 1_000;

            let result = ((liquidity_scaled as u128) * sqrt_price_scaled * 1_000_000)
                .checked_div(denominator)
                .ok_or_else(|| anyhow::anyhow!("Division error"))?;

            Ok(result)
        } else {
            let delta = (amount_in as u128)
                .checked_mul(Q64)
                .ok_or_else(|| anyhow::anyhow!("Overflow in amount calculation"))?
                .checked_div(self.liquidity)
                .ok_or_else(|| anyhow::anyhow!("Division by zero"))?;

            Ok(self.sqrt_price_x64
                .checked_add(delta)
                .ok_or_else(|| anyhow::anyhow!("Overflow in price addition"))?)
        }
    }

    fn calculate_amount_0_delta(
        &self,
        sqrt_price_a: u128,
        sqrt_price_b: u128,
        liquidity: u128,
        round_up: bool,
    ) -> AResult<u64> {
        let (lower, upper) = if sqrt_price_a < sqrt_price_b {
            (sqrt_price_a, sqrt_price_b)
        } else {
            (sqrt_price_b, sqrt_price_a)
        };

        if lower == 0 {
            return Err(anyhow::anyhow!("Invalid sqrt price"));
        }

        let price_delta = upper.saturating_sub(lower);

        // Calculate: amount = liquidity * price_delta * Q64 / (upper * lower)
        // To avoid overflow, we rearrange: amount = (liquidity * price_delta / lower) * Q64 / upper

        // First calculate liquidity * price_delta / lower
        let intermediate = (liquidity as u128)
            .checked_mul(price_delta)
            .and_then(|v| v.checked_div(lower))
            .ok_or_else(|| anyhow::anyhow!("Overflow in intermediate calculation"))?;

        // Then multiply by Q64 and divide by upper
        let mut amount = intermediate
            .checked_mul(Q64)
            .and_then(|v| v.checked_div(upper))
            .ok_or_else(|| anyhow::anyhow!("Overflow in final calculation"))?;

        // Handle rounding
        if round_up {
            let remainder_check = intermediate
                .checked_mul(Q64)
                .and_then(|v| Some(v % upper))
                .unwrap_or(0);
            if remainder_check != 0 {
                amount = amount.saturating_add(1);
            }
        }

        if amount > u64::MAX as u128 {
            return Err(anyhow::anyhow!("Amount exceeds u64 max"));
        }

        Ok(amount as u64)
    }

    fn calculate_amount_1_delta(
        &self,
        sqrt_price_a: u128,
        sqrt_price_b: u128,
        liquidity: u128,
        round_up: bool,
    ) -> AResult<u64> {
        let price_delta = if sqrt_price_a > sqrt_price_b {
            sqrt_price_a.saturating_sub(sqrt_price_b)
        } else {
            sqrt_price_b.saturating_sub(sqrt_price_a)
        };

        let mut amount = (liquidity as u128)
            .checked_mul(price_delta)
            .ok_or_else(|| anyhow::anyhow!("Overflow in amount calculation"))?
            .checked_div(Q64)
            .ok_or_else(|| anyhow::anyhow!("Division error"))?;

        if round_up && ((liquidity as u128).checked_mul(price_delta).unwrap_or(0) % Q64) != 0 {
            amount = amount.saturating_add(1);
        }

        if amount > u64::MAX as u128 {
            return Err(anyhow::anyhow!("Amount exceeds u64 max"));
        }

        Ok(amount as u64)
    }
}
