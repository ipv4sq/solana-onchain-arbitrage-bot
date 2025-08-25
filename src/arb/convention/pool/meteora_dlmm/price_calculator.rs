use crate::arb::convention::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::arb::util::alias::MintAddress;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

pub enum Direction {
    XtoY,
    YtoX,
}

pub struct DlmmQuote {
    pub mid_price: Decimal, // 0滑点价: 1 unit_in -> ? unit_out
                            // pub fee_bps_total: u32, // 交易费(池费 + 协议费)，单位bps
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

    pub fn mid_price_for_quick_estimate(&self, from: &MintAddress, to: &MintAddress) -> DlmmQuote {
        let base = Decimal::ONE + Decimal::from(self.bin_step) / Decimal::from(10_000u32);
        let px_x_per_y = base.powi(self.active_id as i64); // 1 X -> ? Y
        let px_y_per_x = Decimal::ONE / px_x_per_y; // 1 Y -> ? X
        let dir = self.dir(from, to);
        let mid_price = match dir {
            Direction::XtoY => px_x_per_y,
            Direction::YtoX => px_y_per_x,
        };

        DlmmQuote { mid_price }
    }
}
