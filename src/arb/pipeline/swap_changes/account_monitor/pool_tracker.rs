use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::util::alias::{MintAddress, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;

pub async fn get_minor_mint_for_pool(pool: &PoolAddress) -> Option<MintAddress> {
    let pool = PoolRecordRepository::get_pool_by_address(pool).await?;
    MintPair(pool.base_mint.into(), pool.quote_mint.into())
        .minor_mint()
        .ok()
}
