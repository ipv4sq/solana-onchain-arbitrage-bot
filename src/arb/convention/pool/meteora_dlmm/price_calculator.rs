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
        let px_x_per_y = base.powi(self.active_id as i64); // 1 X -> ? Y
        let px_y_per_x = Decimal::ONE / px_x_per_y; // 1 Y -> ? X
        let dir = self.dir(from, to);
        let mid_price_token = match dir {
            Direction::XtoY => px_x_per_y,
            Direction::YtoX => px_y_per_x,
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
