use crate::arb::convention::pool::interface::Direction;
use crate::arb::convention::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::arb::database::repositories::mint_repo::MintRecordRepository;
use crate::arb::util::alias::MintAddress;
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
        panic!();
    }

    pub async fn mid_price_for_quick_estimate(
        &self,
        from: &MintAddress,
        to: &MintAddress,
    ) -> Result<DlmmQuote> {
        let base = Decimal::ONE + Decimal::from(self.bin_step) / Decimal::from(10_000u32);
        let px = base.powi(self.active_id as i64);
        
        let x_dec: u8 = MintRecordRepository::get_decimal_from_cache(&self.token_x_mint)
            .await?
            .ok_or_else(|| anyhow!("mint decimals not found in cache for token_x"))?;

        let y_dec: u8 = MintRecordRepository::get_decimal_from_cache(&self.token_y_mint)
            .await?
            .ok_or_else(|| anyhow!("mint decimals not found in cache for token_y"))?;

        let dir = self.dir(from, to);
        let mid_price = match dir {
            Direction::XtoY => {
                let scale = Decimal::from(10u64.pow(x_dec as u32)) / Decimal::from(10u64.pow(y_dec as u32));
                px * scale
            }
            Direction::YtoX => {
                let scale = Decimal::from(10u64.pow(y_dec as u32)) / Decimal::from(10u64.pow(x_dec as u32));
                (Decimal::ONE / px) * scale
            }
        };

        Ok(DlmmQuote {
            mid_price,
        })
    }
}
