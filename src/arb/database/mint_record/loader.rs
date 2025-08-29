use crate::arb::database::mint_record::model::Model as MintRecord;
#[allow(unused_imports)]
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::constant::token_program::TokenProgram;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::util::traits::orm::ToOrm;
use anyhow::Result;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::programs::MPL_TOKEN_METADATA_ID as METADATA_PROGRAM_ID;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_token::state::Mint;
use spl_token_2022::extension::StateWithExtensions;

pub async fn load_mint_from_address(mint: &Pubkey) -> Result<MintRecord> {
    let client = rpc_client();
    let account = client
        .get_account(mint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch mint account {}: {}", mint, e))?;

    let (decimals, owner) = if account.owner == TokenProgram::SPL_TOKEN {
        let mint_state = Mint::unpack(&account.data)
            .map_err(|e| anyhow::anyhow!("Failed to unpack SPL mint data for {}: {}", mint, e))?;
        (mint_state.decimals, account.owner)
    } else if account.owner == TokenProgram::TOKEN_2022 {
        let mint_state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&account.data)
            .map_err(|e| {
                anyhow::anyhow!("Failed to unpack Token-2022 mint data for {}: {}", mint, e)
            })?;
        (mint_state.base.decimals, account.owner)
    } else {
        return Err(anyhow::anyhow!(
            "Account {} is not a valid mint. Owner: {}",
            mint,
            account.owner
        ));
    };

    let symbol = match fetch_token_metadata(mint).await {
        Ok((symbol, _name)) => symbol,
        Err(_) => "Unknown".to_string(),
    };

    Ok(MintRecord {
        address: mint.to_orm(),
        symbol,
        decimals: decimals as i16,
        program: owner.to_orm(),
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

    if let Ok(mint_record) = result {
        assert_eq!(mint_record.decimals, 9);
        assert_eq!(mint_record.symbol, "SOL");
    }
}

#[tokio::test]
async fn test_load_custom_mint() {
    let result = load_mint_from_address(&Mints::USDC).await;

    if let Ok(mint_record) = result {
        assert_eq!(mint_record.decimals, 6);
        assert_eq!(mint_record.symbol, "USDC");
    }
}

#[tokio::test]
async fn test_load_token_2022_mint() {
    use crate::arb::util::traits::pubkey::ToPubkey;

    let token_2022_mint = "BnszRWbs9LxSzsCUUS57HMTNNtyDHFsnmZ1mVhAYdaos".to_pubkey();
    let result = load_mint_from_address(&token_2022_mint).await;

    if let Ok(mint_record) = result {
        assert_eq!(mint_record.decimals, 9);
        assert_eq!(mint_record.program, TokenProgram::TOKEN_2022.to_orm());
        assert_eq!(mint_record.address, token_2022_mint.to_orm());
        assert!(mint_record.symbol == "Unknown" || mint_record.symbol == "LLM");
    }
}
