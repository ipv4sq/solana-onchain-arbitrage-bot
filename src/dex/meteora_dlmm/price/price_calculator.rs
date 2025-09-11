use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::global::enums::direction::Direction;
use crate::util::alias::MintAddress;
use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

#[derive(Debug, Clone, Copy)]
pub struct DlmmQuote {
    pub mid_price: Decimal, // 0滑点价: 1 unit_in -> ? unit_out
}

impl MeteoraDlmmPoolData {
    pub fn dir(&self, from: &MintAddress, to: &MintAddress) -> Direction {
        if *from == self.token_x_mint && *to == self.token_y_mint {
            return Direction::XtoY;
        } else if *from == self.token_y_mint && *to == self.token_x_mint {
            return Direction::YtoX;
        }
        panic!(
            "Pool has mints: {} <> {}, but not {} <> {}",
            self.token_x_mint, self.token_y_mint, from, to
        );
    }

    pub async fn mid_price_for_quick_estimate(
        &self,
        from: &MintAddress,
        to: &MintAddress,
    ) -> Result<DlmmQuote> {
        let base = Decimal::ONE + Decimal::from(self.bin_step) / Decimal::from(10_000u32);
        let px = base.powi(self.active_id as i64);

        let x_dec: u8 = MintRecordRepository::get_decimal(&self.token_x_mint)
            .await
            .ok_or_else(|| anyhow!("mint decimals not found in cache for token_x"))?;

        let y_dec: u8 = MintRecordRepository::get_decimal(&self.token_y_mint)
            .await
            .ok_or_else(|| anyhow!("mint decimals not found in cache for token_y"))?;

        let dir = self.dir(from, to);
        let mid_price = match dir {
            Direction::XtoY => {
                let scale =
                    Decimal::from(10u64.pow(x_dec as u32)) / Decimal::from(10u64.pow(y_dec as u32));
                px * scale
            }
            Direction::YtoX => {
                let scale =
                    Decimal::from(10u64.pow(y_dec as u32)) / Decimal::from(10u64.pow(x_dec as u32));
                (Decimal::ONE / px) * scale
            }
        };

        Ok(DlmmQuote { mid_price })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::mint_record::repository::MintRecordRepository;
    use crate::dex::interface::PoolDataLoader;
    use crate::global::constant::mint::Mints;
    use crate::sdk::solana_rpc::buffered_get_account::buffered_get_account;
    use crate::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_pool_sol_usdc_price() {
        let pool_address = "5rCf1DM8LjKTw4YqhnoLcngyZYeNnQqztScTogYHAS6".to_pubkey();

        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();
        let sol_mint = Mints::WSOL;

        let account = buffered_get_account(&pool_address).await.unwrap();
        let account_data = account.data;

        let pool_data = MeteoraDlmmPoolData::load_data(&account_data).unwrap();

        let quote_sol_to_usdc = pool_data
            .mid_price_for_quick_estimate(&sol_mint, &usdc_mint)
            .await
            .unwrap();

        let quote_usdc_to_sol = pool_data
            .mid_price_for_quick_estimate(&usdc_mint, &sol_mint)
            .await
            .unwrap();

        println!("1 SOL = {} USDC", quote_sol_to_usdc.mid_price);
        println!("1 USDC = {} SOL", quote_usdc_to_sol.mid_price);

        let reciprocal_check = quote_sol_to_usdc.mid_price * quote_usdc_to_sol.mid_price;
        let diff = (reciprocal_check - Decimal::ONE).abs();
        assert!(
            diff < Decimal::new(1, 6),
            "Reciprocal check failed: {}",
            reciprocal_check
        );

        assert!(
            quote_sol_to_usdc.mid_price >= Decimal::from(150)
                && quote_sol_to_usdc.mid_price <= Decimal::from(250),
            "SOL price outside expected range: {} USDC (expected 150-250 USDC)",
            quote_sol_to_usdc.mid_price
        );

        assert!(
            quote_usdc_to_sol.mid_price >= Decimal::new(4, 3)
                && quote_usdc_to_sol.mid_price <= Decimal::new(67, 4),
            "USDC->SOL price outside expected range: {} SOL (expected 0.004-0.0067 SOL per USDC)",
            quote_usdc_to_sol.mid_price
        );
    }

    #[tokio::test]
    async fn test_pool_trump_usdc_price() {
        let pool_address = "9d9mb8kooFfaD3SctgZtkxQypkshx6ezhbKio89ixyy2".to_pubkey();

        // TRUMP token mint (from pool data)
        let trump_mint = "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN".to_pubkey();
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();

        let account = buffered_get_account(&pool_address).await.unwrap();
        let account_data = account.data;

        let pool_data = MeteoraDlmmPoolData::load_data(&account_data).unwrap();

        println!("\n=== Pool 9d9mb8kooFfaD3SctgZtkxQypkshx6ezhbKio89ixyy2 ===");
        println!("Token X: {}", pool_data.token_x_mint);
        println!("Token Y: {}", pool_data.token_y_mint);
        println!("Active ID: {}", pool_data.active_id);
        println!("Bin Step: {}", pool_data.bin_step);

        // Determine which token is which
        let is_trump_x = pool_data.token_x_mint == trump_mint;

        println!("Token X is: {}", if is_trump_x { "TRUMP" } else { "USDC" });
        println!("Token Y is: {}", if is_trump_x { "USDC" } else { "TRUMP" });

        let quote_trump_to_usdc = pool_data
            .mid_price_for_quick_estimate(&trump_mint, &usdc_mint)
            .await
            .unwrap();

        let quote_usdc_to_trump = pool_data
            .mid_price_for_quick_estimate(&usdc_mint, &trump_mint)
            .await
            .unwrap();

        println!("\n=== Price Results ===");
        println!("1 TRUMP = {} USDC", quote_trump_to_usdc.mid_price);
        println!("1 USDC = {} TRUMP", quote_usdc_to_trump.mid_price);

        // Verify reciprocal relationship
        let reciprocal_check = quote_trump_to_usdc.mid_price * quote_usdc_to_trump.mid_price;
        let diff = (reciprocal_check - Decimal::ONE).abs();
        assert!(
            diff < Decimal::new(1, 6),
            "Reciprocal check failed: {}",
            reciprocal_check
        );

        // TRUMP price should be in a reasonable range (5-50 USDC based on current market)
        assert!(
            quote_trump_to_usdc.mid_price >= Decimal::from(5)
                && quote_trump_to_usdc.mid_price <= Decimal::from(50),
            "TRUMP price outside expected range: {} USDC (expected 5-50 USDC)",
            quote_trump_to_usdc.mid_price
        );
    }
}
