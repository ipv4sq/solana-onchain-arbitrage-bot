use crate::arb::database::entity::mint_record::{Entity as MintEntity, Model as MintRecord};
use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::db::get_db;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::util::traits::orm::ToOrm;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::return_ok_if_some;
use anyhow::Result;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::programs::MPL_TOKEN_METADATA_ID as METADATA_PROGRAM_ID;
use sea_orm::EntityTrait;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_token::state::Mint;

pub async fn ensure_mint_record_exist(mint: &Pubkey) -> Result<MintRecord> {
    let existed = MintEntity::find_by_id(mint.to_orm()).one(get_db()).await?;
    return_ok_if_some!(existed);

    let record = load_mint_from_address(mint).await?;
    Ok(MintRecordRepository::upsert_mint(record).await?)
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
        Err(_) => "Unknown".to_string(),
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

#[tokio::test]
async fn test_load_mint_from_address() {
    let result = load_mint_from_address(&Mints::WSOL).await;

    match result {
        Ok(mint_record) => {
            assert_eq!(mint_record.decimals, 9);
            assert_eq!(mint_record.symbol, "SOL");
        }
        Err(e) => {
            println!("Failed to load mint (expected if not on mainnet): {}", e);
        }
    }
}

#[tokio::test]
async fn test_load_custom_mint() {
    let result = load_mint_from_address(&Mints::USDC).await;

    match result {
        Ok(mint_record) => {
            assert_eq!(mint_record.decimals, 6);
            assert_eq!(mint_record.symbol, "USDC");
        }
        Err(e) => {
            println!("Failed to load mint: {}", e);
        }
    }
}
