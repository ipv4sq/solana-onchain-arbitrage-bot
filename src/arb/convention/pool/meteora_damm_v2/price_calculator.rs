use crate::arb::convention::pool::interface::Direction;
use crate::arb::convention::pool::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::arb::convention::pool::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::database::repositories::mint_repo::MintRecordRepository;
use crate::arb::util::alias::{AResult, MintAddress};
use anyhow::anyhow;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

impl MeteoraDammV2PoolData {
    pub fn dir(&self, from: &MintAddress, to: &MintAddress) -> Direction {
        if *from == self.token_a_mint && *to == self.token_b_mint {
            return Direction::XtoY;
        } else if *from == self.token_b_mint && *to == self.token_a_mint {
            return Direction::YtoX;
        }
        panic!();
    }

    pub async fn mid_price_for_quick_estimate(
        &self,
        from: &MintAddress,
        to: &MintAddress,
    ) -> AResult<DlmmQuote> {
        const Q64: u128 = 1 << 64;

        let sqrt_price_q64 = Decimal::from_u128(self.sqrt_price)
            .ok_or_else(|| anyhow!("Failed to convert sqrt_price to Decimal"))?;
        let q64_decimal =
            Decimal::from_u128(Q64).ok_or_else(|| anyhow!("Failed to convert Q64 to Decimal"))?;

        let sqrt_price_decimal = sqrt_price_q64 / q64_decimal;
        let price_a_per_b = sqrt_price_decimal * sqrt_price_decimal;

        let price_b_per_a = Decimal::ONE / price_a_per_b;

        let dir = self.dir(from, to);
        let mid_price_token = match dir {
            Direction::XtoY => price_a_per_b,
            Direction::YtoX => price_b_per_a,
        };

        let from_dec: u8 = MintRecordRepository::get_decimal_from_cache(from)
            .await?
            .ok_or_else(|| anyhow!("mint decimals not found in cache for {}", from))?;

        let to_dec: u8 = MintRecordRepository::get_decimal_from_cache(to)
            .await?
            .ok_or_else(|| anyhow!("mint decimals not found in cache for {}", to))?;

        let exp = to_dec as i32 - from_dec as i32;
        let scale = if exp >= 0 {
            Decimal::from(10u64.pow(exp as u32))
        } else {
            Decimal::new(1, (-exp) as u32)
        };

        Ok(DlmmQuote {
            mid_price: mid_price_token * scale,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::util::traits::pubkey::ToPubkey;
    use crate::arb::global::state::rpc::rpc_client;
    use crate::arb::convention::pool::interface::PoolDataLoader;
    
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_pool_8Pm2kZpnxD3hoMmt4bjStX2Pw2Z9abpbHzZxMPqxPmie_price() {
        let pool_address = "8Pm2kZpnxD3hoMmt4bjStX2Pw2Z9abpbHzZxMPqxPmie".to_pubkey();
        let client = rpc_client();
        
        // Fetch pool account data
        let account = client
            .get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account");
        
        let pool_data = MeteoraDammV2PoolData::load_data(&account.data)
            .expect("Failed to parse pool data");
        
        println!("Pool: {}", pool_address);
        println!("Token A mint: {}", pool_data.token_a_mint);
        println!("Token B mint: {}", pool_data.token_b_mint);
        println!("sqrt_price: {}", pool_data.sqrt_price);
        
        // Calculate price directly
        const Q64: u128 = 1 << 64;
        let sqrt_price_decimal = pool_data.sqrt_price as f64 / Q64 as f64;
        let price_a_per_b = sqrt_price_decimal * sqrt_price_decimal;
        let price_b_per_a = 1.0 / price_a_per_b;
        
        // Assuming SOL is token A and USDC is token B
        let sol_mint = "So11111111111111111111111111111111111111112".to_pubkey();
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();
        
        if pool_data.token_a_mint == sol_mint && pool_data.token_b_mint == usdc_mint {
            // 1 SOL = ? USDC
            println!("\n=== Price Calculation ===");
            println!("sqrt_price (decimal): {}", sqrt_price_decimal);
            println!("Raw price A per B: {}", price_a_per_b);
            println!("Raw price B per A: {}", price_b_per_a);
            
            // Adjust for decimals: SOL has 9 decimals, USDC has 6 decimals
            // The price needs to be multiplied by 10^(6-9) = 10^-3 = 0.001
            let decimal_adjustment = 0.001; // 10^(6-9)
            let adjusted_price = price_a_per_b / decimal_adjustment;
            
            println!("\n=== Decimal Adjustment ===");
            println!("SOL decimals: 9, USDC decimals: 6");
            println!("Adjustment factor: {}", decimal_adjustment);
            println!("Adjusted price (1 SOL = {} USDC): {}", adjusted_price, adjusted_price);
            
            let usdc_to_sol = 1.0 / adjusted_price;
            
            println!("\n=== RESULT ===");
            println!("1 SOL = {} USDC", adjusted_price);
            println!("1 USDC = {} SOL", usdc_to_sol);
        } else if pool_data.token_b_mint == sol_mint && pool_data.token_a_mint == usdc_mint {
            // USDC is token A, SOL is token B
            println!("\n=== Price Calculation ===");
            println!("sqrt_price (decimal): {}", sqrt_price_decimal);
            println!("Raw price A per B: {}", price_a_per_b);
            println!("Raw price B per A: {}", price_b_per_a);
            
            // Adjust for decimals
            let decimal_adjustment = 1000.0; // 10^(9-6)
            let adjusted_price = price_b_per_a / decimal_adjustment;
            
            println!("\n=== Decimal Adjustment ===");
            println!("USDC decimals: 6, SOL decimals: 9");
            println!("Adjustment factor: {}", decimal_adjustment);
            println!("Adjusted price (1 SOL = {} USDC): {}", adjusted_price, adjusted_price);
            
            println!("\n=== RESULT: 1 SOL can buy approximately {} USDC ===", adjusted_price);
        } else {
            println!("Unexpected token configuration!");
            println!("Token A: {}", pool_data.token_a_mint);
            println!("Token B: {}", pool_data.token_b_mint);
        }
    }
}
