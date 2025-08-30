use crate::arb::database::mint_record::repository::MintRecordRepository;
use crate::arb::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::global::enums::direction::Direction;
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

        let from_dec: u8 = MintRecordRepository::get_decimal(from)
            .await?
            .ok_or_else(|| anyhow!("mint decimals not found in cache for {}", from))?;

        let to_dec: u8 = MintRecordRepository::get_decimal(to)
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
