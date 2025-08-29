use crate::arb::convention::pool::interface::PoolDataLoader;
use crate::arb::database::entity::pool_do::{
    Model as PoolRecord, PoolRecordDescriptor,
};
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::pipeline::pool_indexer::token_recorder;
use crate::arb::util::traits::orm::ToOrm;
use anyhow::Result;
use sea_orm::EntityTrait;
use serde::Serialize;
use solana_program::pubkey::Pubkey;

pub async fn build_model<T: PoolDataLoader + Serialize>(
    pool: &Pubkey,
    data: &T,
    dex_type: DexType,
) -> Result<PoolRecord> {
    let base = token_recorder::ensure_mint_record_exist(&data.base_mint()).await?;
    let quote = token_recorder::ensure_mint_record_exist(&data.quote_mint()).await?;
    let name = format!("{} - {}", base.symbol, quote.symbol);
    Ok(PoolRecord {
        address: pool.to_orm(),
        name,
        dex_type,
        base_mint: data.base_mint().to_orm(),
        quote_mint: data.quote_mint().to_orm(),
        base_vault: data.base_vault().to_orm(),
        quote_vault: data.quote_vault().to_orm(),
        description: PoolRecordDescriptor {
            base_symbol: base.symbol,
            quote_symbol: quote.symbol,
            base: data.base_mint().to_string(),
            quote: data.quote_mint().to_string(),
            pool_address: pool.to_string(),
        },
        data_snapshot: serde_json::json!(data),
        created_at: None,
        updated_at: None,
    })
}
