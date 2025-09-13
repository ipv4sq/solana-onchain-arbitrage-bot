use crate::dex::whirlpool::pool_data::WhirlpoolPoolData;
use crate::util::alias::{AResult, MintAddress};

impl WhirlpoolPoolData {
    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        todo!()
    }
}
