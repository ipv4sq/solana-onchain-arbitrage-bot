use crate::arb::convention::pool::interface::PoolDataLoader;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::mint_record::Model as MintRecord;
use crate::arb::database::entity::pool_record::{Model as PoolRecord, PoolRecordDescriptor};
use crate::arb::global::enums::dex_type::DexType;
use anyhow::Result;
use solana_program::pubkey::Pubkey;

pub async fn ensure_mint_record_exist(mint: &Pubkey) -> Result<()> {
    let record = load_mint_from_address(mint)?;

    todo!()
}

pub async fn load_mint_from_address(mint: &Pubkey) -> Result<MintRecord> {

    todo!()
}

pub async fn upsert_pool(pool: &Pubkey, dex_type: DexType, name: Option<String>) -> Result<()> {
    let any_config = AnyPoolConfig::from_address(pool, dex_type).await?;

    fn build_model<T: PoolDataLoader>(
        pool: &Pubkey,
        data: &T,
        dex_type: DexType,
        name: Option<String>,
    ) -> PoolRecord {
        PoolRecord {
            address: (*pool).into(),
            name: name.unwrap_or("Unknown Pool".into()),
            dex_type,
            base_mint: PubkeyType(data.base_mint()),
            quote_mint: PubkeyType(data.quote_mint()),
            base_vault: PubkeyType(data.base_vault()),
            quote_vault: PubkeyType(data.quote_vault()),
            description: PoolRecordDescriptor {
                base_symbol: "BASE".to_string(),
                quote_symbol: "QUOTE".to_string(),
                base: data.base_mint(),
                quote: data.quote_mint(),
            },
            data_snapshot: serde_json::json!({}), // Can't serialize generic data
            created_at: None,
            updated_at: None,
        }
    }

    let dto = match any_config {
        AnyPoolConfig::MeteoraDlmm(c) => build_model(pool, &c.data, dex_type, name.clone()),
        AnyPoolConfig::MeteoraDammV2(c) => build_model(pool, &c.data, dex_type, name),
        AnyPoolConfig::Unsupported => todo!(),
    };

    todo!()
}
