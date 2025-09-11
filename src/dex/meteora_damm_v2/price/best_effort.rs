use crate::dex::meteora_damm_v2::misc::curve::{
    get_delta_amount_a_unsigned, get_delta_amount_b_unsigned, get_next_sqrt_price_from_input,
    Rounding,
};
use crate::dex::meteora_damm_v2::misc::fee::{FeeMode, FeeOnAmountResult};
use crate::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::util::alias::{AResult, MintAddress};
use anyhow::anyhow;
use chrono::Utc;

#[derive(Debug)]
pub enum TradeDirection {
    AtoB,
    BtoA,
}

#[derive(Debug)]
pub struct SwapResult {
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

impl MeteoraDammV2PoolData {
    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        if input_amount == 0 {
            return Ok(0);
        }

        if self.pool_status != 0 {
            return Err(anyhow!("Pool is disabled"));
        }

        let trade_direction = if *from_mint == self.token_a_mint && *to_mint == self.token_b_mint {
            TradeDirection::AtoB
        } else if *from_mint == self.token_b_mint && *to_mint == self.token_a_mint {
            TradeDirection::BtoA
        } else {
            return Err(anyhow!("Invalid mint pair for this pool"));
        };

        let current_timestamp = Utc::now().timestamp() as u64;
        let current_point = current_timestamp;

        let fee_mode = FeeMode {
            fees_on_input: true,
            fees_on_token_a: matches!(trade_direction, TradeDirection::AtoB),
            has_referral: false,
        };

        let swap_result =
            self.get_swap_result(input_amount, &fee_mode, trade_direction, current_point)?;

        Ok(swap_result.output_amount)
    }

    fn get_swap_result(
        &self,
        amount_in: u64,
        fee_mode: &FeeMode,
        trade_direction: TradeDirection,
        current_point: u64,
    ) -> AResult<SwapResult> {
        let mut actual_protocol_fee = 0;
        let mut actual_lp_fee = 0;
        let mut actual_referral_fee = 0;
        let mut actual_partner_fee = 0;

        let actual_amount_in = if fee_mode.fees_on_input {
            let FeeOnAmountResult {
                amount,
                lp_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                amount_in,
                fee_mode.has_referral,
                current_point,
                self.activation_point,
                self.has_partner(),
            )?;

            actual_protocol_fee = protocol_fee;
            actual_lp_fee = lp_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;

            amount
        } else {
            amount_in
        };

        let (output_amount, next_sqrt_price) = match trade_direction {
            TradeDirection::AtoB => self.get_swap_result_from_a_to_b(actual_amount_in)?,
            TradeDirection::BtoA => self.get_swap_result_from_b_to_a(actual_amount_in)?,
        };

        let actual_amount_out = if fee_mode.fees_on_input {
            output_amount
        } else {
            let FeeOnAmountResult {
                amount,
                lp_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                output_amount,
                fee_mode.has_referral,
                current_point,
                self.activation_point,
                self.has_partner(),
            )?;

            actual_protocol_fee = protocol_fee;
            actual_lp_fee = lp_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;
            amount
        };

        Ok(SwapResult {
            output_amount: actual_amount_out,
            next_sqrt_price,
            lp_fee: actual_lp_fee,
            protocol_fee: actual_protocol_fee,
            partner_fee: actual_partner_fee,
            referral_fee: actual_referral_fee,
        })
    }

    fn get_swap_result_from_a_to_b(&self, amount_in: u64) -> AResult<(u64, u128)> {
        let next_sqrt_price =
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, true)?;

        if next_sqrt_price < self.sqrt_min_price {
            return Err(anyhow!("Price range violation: price too low"));
        }

        let output_amount = get_delta_amount_b_unsigned(
            next_sqrt_price.min(self.sqrt_price),
            next_sqrt_price.max(self.sqrt_price),
            self.liquidity,
            Rounding::Down,
        )?;

        Ok((output_amount, next_sqrt_price))
    }

    fn get_swap_result_from_b_to_a(&self, amount_in: u64) -> AResult<(u64, u128)> {
        let next_sqrt_price =
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, false)?;

        if next_sqrt_price > self.sqrt_max_price {
            return Err(anyhow!("Price range violation: price too high"));
        }

        let output_amount = get_delta_amount_a_unsigned(
            self.sqrt_price.min(next_sqrt_price),
            self.sqrt_price.max(next_sqrt_price),
            self.liquidity,
            Rounding::Down,
        )?;

        Ok((output_amount, next_sqrt_price))
    }

    fn has_partner(&self) -> bool {
        self.partner != solana_program::pubkey::Pubkey::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dex::interface::PoolDataLoader;
    use crate::dex::meteora_damm_v2::pool_data::test::load_pool_data;
    use crate::global::constant::mint::Mints;
    use crate::sdk::solana_rpc::proxy;
    use crate::util::traits::pubkey::ToPubkey;
    use solana_program::pubkey::Pubkey;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_amount_out_a_to_b() {
        let pool_data = load_pool_data();

        let token_a_mint = pool_data.token_a_mint;
        let token_b_mint = pool_data.token_b_mint;

        let input_amount = 1_000_000_000;

        let result = pool_data
            .get_amount_out(input_amount, &token_a_mint, &token_b_mint)
            .await;

        assert!(result.is_ok());
        let output_amount = result.unwrap();
        assert!(output_amount > 0);
        assert!(output_amount < input_amount * 100);
    }

    #[tokio::test]
    async fn test_get_amount_out_b_to_a() {
        let pool_data = load_pool_data();

        let token_a_mint = pool_data.token_a_mint;
        let token_b_mint = pool_data.token_b_mint;

        let input_amount = 1_000_000;

        let result = pool_data
            .get_amount_out(input_amount, &token_b_mint, &token_a_mint)
            .await;

        assert!(result.is_ok());
        let output_amount = result.unwrap();
        assert!(output_amount > 0);
    }

    #[tokio::test]
    async fn test_get_amount_out_zero_input() {
        let pool_data = load_pool_data();

        let token_a_mint = pool_data.token_a_mint;
        let token_b_mint = pool_data.token_b_mint;

        let input_amount = 0;

        let result = pool_data
            .get_amount_out(input_amount, &token_a_mint, &token_b_mint)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_amount_out_invalid_mints() {
        let pool_data = load_pool_data();

        let invalid_mint = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let token_b_mint = pool_data.token_b_mint;

        let input_amount = 1_000_000;

        let result = pool_data
            .get_amount_out(input_amount, &invalid_mint, &token_b_mint)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test test_meteora_damm_v2_real_pool -- --ignored
    async fn test_meteora_damm_v2_real_pool() {
        let pool_address: Pubkey = "8Pm2kZpnxD3hoMmt4bjStX2Pw2Z9abpbHzZxMPqxPmie".to_pubkey();
        let wsol: Pubkey = Mints::WSOL;
        let usdc: Pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();

        let account = proxy
            .get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account");

        let pool_data =
            MeteoraDammV2PoolData::load_data(&account.data).expect("Failed to load pool data");

        // Determine which token is which
        let (sol_mint, usdc_mint) = if pool_data.token_a_mint == wsol {
            (pool_data.token_a_mint, pool_data.token_b_mint)
        } else if pool_data.token_b_mint == wsol {
            (pool_data.token_b_mint, pool_data.token_a_mint)
        } else {
            panic!("This pool doesn't contain SOL");
        };

        // Test 1 SOL -> USDC
        let one_sol = 1_000_000_000u64;
        let result = pool_data
            .get_amount_out(one_sol, &sol_mint, &usdc_mint)
            .await;
        assert!(result.is_ok());

        let usdc_out = result.unwrap();
        let usdc_human = usdc_out as f64 / 1_000_000.0;

        // Exchange rate should be reasonable (between $50 and $500 per SOL)
        assert!(
            usdc_human > 50.0 && usdc_human < 500.0,
            "Exchange rate out of expected range: 1 SOL = {:.2} USDC",
            usdc_human
        );

        // Test reverse swap
        let hundred_usdc = 100_000_000u64; // 100 USDC
        let reverse_result = pool_data
            .get_amount_out(hundred_usdc, &usdc_mint, &sol_mint)
            .await;
        assert!(reverse_result.is_ok());

        let sol_out = reverse_result.unwrap();
        let sol_human = sol_out as f64 / 1_000_000_000.0;

        // Should get reasonable amount of SOL for 100 USDC
        assert!(
            sol_human > 0.2 && sol_human < 2.0,
            "Reverse swap out of expected range: 100 USDC = {:.6} SOL",
            sol_human
        );
    }
}
