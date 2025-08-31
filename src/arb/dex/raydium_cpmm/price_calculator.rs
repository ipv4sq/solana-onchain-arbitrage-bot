use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
use crate::arb::util::alias::{AResult, MintAddress};

impl RaydiumCpmmPoolData {
    pub async fn mid_price_for_quick_estimate(
        &self,
        from: &MintAddress,
        to: &MintAddress,
    ) -> AResult<DlmmQuote> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_price() {}
}
