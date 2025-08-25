use crate::arb::convention::pool::interface::Direction;
use crate::arb::convention::pool::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::arb::convention::pool::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::util::alias::MintAddress;
use anyhow::anyhow;
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
    ) -> anyhow::Result<DlmmQuote> {
        todo!()
    }
}
