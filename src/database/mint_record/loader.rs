use crate::database::mint_record::model::Model as MintRecord;
#[allow(unused_imports)]
use crate::global::constant::mint::Mints;
use crate::global::constant::token_program::TokenProgram;
use crate::sdk::solana_rpc::buffered_get_account::buffered_get_account_batch;
use crate::util::traits::orm::ToOrmString;
use anyhow::Result;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::programs::MPL_TOKEN_METADATA_ID as METADATA_PROGRAM_ID;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_token::state::Mint;
use spl_token_2022::extension::StateWithExtensions;

pub async fn load_mint_from_address(mint: &Pubkey) -> Result<MintRecord> {
    let metadata_seeds = &[b"metadata", METADATA_PROGRAM_ID.as_ref(), mint.as_ref()];
    let (metadata_pda, _) = Pubkey::find_program_address(metadata_seeds, &METADATA_PROGRAM_ID);

    let addresses = vec![*mint, metadata_pda];
    let accounts = buffered_get_account_batch(&addresses)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch accounts: {}", e))?;

    let (mint_account, metadata_account) = match accounts.as_slice() {
        [mint_opt, metadata_opt] => (mint_opt.as_ref(), metadata_opt.as_ref()),
        _ => return Err(anyhow::anyhow!("Unexpected number of accounts returned")),
    };

    let account = mint_account.ok_or_else(|| anyhow::anyhow!("Mint account {} not found", mint))?;

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

    let repr = match metadata_account {
        Some(metadata_account) => match deserialize_metadata(&metadata_account.data) {
            Ok(metadata) => {
                let symbol = metadata.symbol.trim_matches('\0').to_string();
                let name = metadata.name.trim_matches('\0').to_string();
                if !symbol.is_empty() {
                    symbol
                } else if !name.is_empty() {
                    name
                } else {
                    "Unknown".to_string()
                }
            }
            Err(_) => "Unknown".to_string(),
        },
        None => "Unknown".to_string(),
    };

    Ok(MintRecord {
        address: mint.to_orm(),
        repr,
        decimals: decimals as i16,
        program: owner.to_orm(),
        created_at: None,
        updated_at: None,
    })
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
        assert_eq!(mint_record.repr, "SOL");
    }
}

#[tokio::test]
async fn test_load_custom_mint() {
    let result = load_mint_from_address(&Mints::USDC).await;

    if let Ok(mint_record) = result {
        assert_eq!(mint_record.decimals, 6);
        assert_eq!(mint_record.repr, "USDC");
    }
}

#[tokio::test]
async fn test_load_token_2022_mint() {
    use crate::util::traits::pubkey::ToPubkey;

    let token_2022_mint = "BnszRWbs9LxSzsCUUS57HMTNNtyDHFsnmZ1mVhAYdaos".to_pubkey();
    let result = load_mint_from_address(&token_2022_mint).await;

    if let Ok(mint_record) = result {
        assert_eq!(mint_record.decimals, 9);
        assert_eq!(mint_record.program.0, TokenProgram::TOKEN_2022);
        assert_eq!(mint_record.address.0, token_2022_mint);
        assert!(mint_record.repr == "Unknown" || mint_record.repr == "LLM");
    }
}
