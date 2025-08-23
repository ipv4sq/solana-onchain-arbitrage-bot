use crate::arb::convention::pool::interface::PoolDataLoader;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::mint_record::Model as MintRecord;
use crate::arb::database::entity::pool_record::{Model as PoolRecord, PoolRecordDescriptor};
use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::global::state::rpc::rpc_client;
use anyhow::Result;
use chrono::Utc;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::ID as METADATA_PROGRAM_ID;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_token::state::Mint;
use std::cmp::min;

pub async fn ensure_mint_record_exist(mint: &Pubkey) -> Result<()> {
    let existed = MintRecordRepository::find_by_address(mint);
    let _record = load_mint_from_address(mint).await?;

    todo!()
}

pub async fn load_mint_from_address(mint: &Pubkey) -> Result<MintRecord> {
    let client = rpc_client();
    let account = client
        .get_account(mint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch mint account {}: {}", mint, e))?;

    let mint_state = Mint::unpack(&account.data)
        .map_err(|e| anyhow::anyhow!("Failed to unpack mint data for {}: {}", mint, e))?;

    // Try to fetch metadata from Metaplex Token Metadata program
    let symbol = match fetch_token_metadata(mint).await {
        Ok((symbol, _name)) => symbol,
        Err(_) => "UNKNOWN".to_string(),
    };

    Ok(MintRecord {
        address: PubkeyType(*mint),
        symbol,
        decimals: mint_state.decimals as i16,
        program: PubkeyType(account.owner),
        created_at: None,
        updated_at: None,
    })
}

async fn fetch_token_metadata(mint: &Pubkey) -> Result<(String, String)> {
    // Derive the metadata PDA
    let metadata_seeds = &[b"metadata", METADATA_PROGRAM_ID.as_ref(), mint.as_ref()];

    let (metadata_pda, _) = Pubkey::find_program_address(metadata_seeds, &METADATA_PROGRAM_ID);

    let client = rpc_client();

    // Try to fetch the metadata account
    match client.get_account(&metadata_pda).await {
        Ok(account) => {
            // Try to deserialize the metadata
            match deserialize_metadata(&account.data) {
                Ok(metadata) => {
                    let symbol = metadata.symbol.trim_matches('\0').to_string();
                    let name = metadata.name.trim_matches('\0').to_string();
                    Ok((symbol, name))
                }
                Err(e) => {
                    // Metadata exists but couldn't deserialize
                    Err(anyhow::anyhow!("Failed to deserialize metadata: {}", e))
                }
            }
        }
        Err(_) => {
            // No metadata account found
            Err(anyhow::anyhow!("No metadata account found"))
        }
    }
}

fn deserialize_metadata(data: &[u8]) -> Result<Metadata> {
    Metadata::safe_deserialize(data)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize metadata: {:?}", e))
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

    let _dto = match any_config {
        AnyPoolConfig::MeteoraDlmm(c) => build_model(pool, &c.data, dex_type, name.clone()),
        AnyPoolConfig::MeteoraDammV2(c) => build_model(pool, &c.data, dex_type, name),
        AnyPoolConfig::Unsupported => todo!(),
    };

    todo!()
}
