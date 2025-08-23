use crate::arb::convention::pool::interface::PoolDataLoader;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::mint_record::Entity as MintEntity;
use crate::arb::database::entity::mint_record::Model as MintRecord;
use crate::arb::database::entity::pool_record::{Model as PoolRecord, PoolRecordDescriptor};
use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::global::db::get_db;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::global::state::rpc::{ensure_mint_account_exists, rpc_client};
use crate::arb::util::traits::orm::ToOrm;
use crate::return_ok_if_some;
use anyhow::{ensure, Result};
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::ID as METADATA_PROGRAM_ID;
use sea_orm::EntityTrait;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_token::state::Mint;

pub async fn ensure_mint_record_exist(mint: &Pubkey) -> Result<MintRecord> {
    let existed = MintEntity::find_by_id(mint.to_orm()).one(get_db()).await?;
    return_ok_if_some!(existed);

    let record = load_mint_from_address(mint).await?;
    let repo = MintRecordRepository::new();
    Ok(repo.upsert_mint(record).await?)
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
        address: mint.to_orm(),
        symbol,
        decimals: mint_state.decimals as i16,
        program: account.owner.to_orm(),
        created_at: None,
        updated_at: None,
    })
}

async fn fetch_token_metadata(mint: &Pubkey) -> Result<(String, String)> {
    let metadata_seeds = &[b"metadata", METADATA_PROGRAM_ID.as_ref(), mint.as_ref()];
    let (metadata_pda, _) = Pubkey::find_program_address(metadata_seeds, &METADATA_PROGRAM_ID);
    let client = rpc_client();

    match client.get_account(&metadata_pda).await {
        Ok(account) => match deserialize_metadata(&account.data) {
            Ok(metadata) => {
                let symbol = metadata.symbol.trim_matches('\0').to_string();
                let name = metadata.name.trim_matches('\0').to_string();
                Ok((symbol, name))
            }
            Err(e) => Err(anyhow::anyhow!("Failed to deserialize metadata: {}", e)),
        },
        Err(_) => Err(anyhow::anyhow!("No metadata account found")),
    }
}

fn deserialize_metadata(data: &[u8]) -> Result<Metadata> {
    Metadata::safe_deserialize(data)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize metadata: {:?}", e))
}

pub async fn upsert_pool(pool: &Pubkey, dex_type: DexType) -> Result<()> {
    let any_config = AnyPoolConfig::from_address(pool, dex_type).await?;

    async fn build_model<T: PoolDataLoader>(
        pool: &Pubkey,
        data: &T,
        dex_type: DexType,
    ) -> Result<PoolRecord> {
        let base = ensure_mint_record_exist(&data.base_mint()).await?;
        let quote = ensure_mint_record_exist(&data.quote_mint()).await?;
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
            data_snapshot: serde_json::json!({}), // Can't serialize generic data
            created_at: None,
            updated_at: None,
        })
    }

    let _dto = match any_config {
        AnyPoolConfig::MeteoraDlmm(c) => build_model(pool, &c.data, dex_type).await?,
        AnyPoolConfig::MeteoraDammV2(c) => build_model(pool, &c.data, dex_type).await?,
        AnyPoolConfig::Unsupported => todo!(),
    };

    todo!()
}
