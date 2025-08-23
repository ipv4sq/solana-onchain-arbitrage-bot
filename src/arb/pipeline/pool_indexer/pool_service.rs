use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::pool_record::{Model, PoolRecordDescriptor};
use crate::arb::global::enums::dex_type::DexType;
use anyhow::Result;
use chrono::Utc;
use solana_program::pubkey::Pubkey;

pub async fn upsert_pool(pool: &Pubkey, dex_type: DexType, name: Option<String>) -> Result<()> {
    let anyConfig = AnyPoolConfig::from_address(pool, dex_type).await?;
    let data = match anyConfig {
        AnyPoolConfig::MeteoraDlmm(c) => {
            c.data
        }
        AnyPoolConfig::MeteoraDammV2(c) => {
            c.data
        }
        AnyPoolConfig::Unsupported => {
            todo!()
        }
    }
    let dto = Model {
        address: (*pool).into(),
        name: name.unwrap_or("Unknown Pool".into()),
        dex_type: DexType::RaydiumV4,             // Or whatever DEX type
        base_mint:, // Replace with actual mint
        quote_mint: PubkeyType(Pubkey::default()), // Replace with actual mint
        base_vault: PubkeyType(Pubkey::default()), // Replace with actual vault
        quote_vault: PubkeyType(Pubkey::default()), // Replace with actual vault
        description: PoolRecordDescriptor {
            base_symbol: "BASE".to_string(),
            quote_symbol: "QUOTE".to_string(),
            base: Pubkey::default(),
            quote: Pubkey::default(),
        },
        data_snapshot: serde_json::json!({}), // Your JSON data here
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    todo!()
}
