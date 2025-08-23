use crate::arb::convention::pool::interface::PoolDataLoader;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::pool_record::{Model as PoolRecord, PoolRecordDescriptor};
use crate::arb::database::repositories::pool_record_repo::PoolRecordRepository;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::pipeline::pool_indexer::token_recorder;
use crate::arb::util::traits::orm::ToOrm;
use anyhow::Result;
use serde::Serialize;
use solana_program::pubkey::Pubkey;

pub async fn upsert_pool(pool: &Pubkey, dex_type: DexType) -> Result<()> {
    let any_config = AnyPoolConfig::from_address(pool, dex_type).await?;

    async fn build_model<T: PoolDataLoader + Serialize>(
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
                base: data.base_mint(),
                quote: data.quote_mint(),
            },
            data_snapshot: serde_json::json!(data),
            created_at: None,
            updated_at: None,
        })
    }

    let dto = match any_config {
        AnyPoolConfig::MeteoraDlmm(c) => build_model(pool, &c.data, dex_type).await?,
        AnyPoolConfig::MeteoraDammV2(c) => build_model(pool, &c.data, dex_type).await?,
        AnyPoolConfig::Unsupported => todo!(),
    };

    PoolRecordRepository::upsert_pool(dto).await?;
    Ok(())
}
