use crate::database::mint_record::repository::MintRecordRepository;
use crate::database::pool_record::model::{Model as PoolRecord, PoolRecordDescriptor};
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::util::traits::orm::ToOrm;
use crate::f;

pub async fn build_model(config: AnyPoolConfig) -> anyhow::Result<PoolRecord> {
    let base = MintRecordRepository::get_mint_err(&config.base_mint()).await?;
    let quote = MintRecordRepository::get_mint_err(&config.quote_mint()).await?;

    let name = f!("{} - {}", base.symbol, quote.symbol);
    Ok(PoolRecord {
        address: config.pool().to_orm(),
        name,
        dex_type: config.dex_type(),
        base_mint: config.base_mint().to_orm(),
        quote_mint: config.quote_mint().to_orm(),
        description: PoolRecordDescriptor {
            base_symbol: base.symbol,
            quote_symbol: quote.symbol,
            base: config.base_mint().to_string(),
            quote: config.quote_mint().to_string(),
            pool_address: config.pool().to_string(),
        },
        data_snapshot: config.pool_data_json(),
        created_at: None,
        updated_at: None,
    })
}
