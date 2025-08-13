use crate::constants::{addresses::TokenMint, helpers::ToPubkey, utils::expect_owner};
use crate::dex::pump::{PumpAmmInfo, PumpPool, PUMP_FEE_WALLET, PUMP_PROGRAM_ID};
use crate::pools::MintPoolData;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account;
use tracing::info;

/// Fetches and parses a single Pump pool
pub fn fetch_pump_pool(
    pool_address: &str,
    mint_pubkey: &Pubkey,
    rpc_client: &RpcClient,
) -> anyhow::Result<PumpPool> {
    let pump_pool = pool_address.to_pubkey();
    let pump_program_id = PUMP_PROGRAM_ID.to_pubkey();

    let account = rpc_client
        .get_account(&pump_pool)
        .map_err(|e| anyhow::anyhow!("Error fetching Pump pool account {pump_pool}: {e:?}"))?;

    expect_owner(&pump_pool, &account, &pump_program_id)?;

    let amm_info = PumpAmmInfo::load_checked(&account.data).map_err(|e| {
        anyhow::anyhow!(
            "Error parsing AmmInfo from Pump pool {}: {:?}",
            pump_pool,
            e
        )
    })?;

    let sol_mint = TokenMint::SOL.to_pubkey();
    let (sol_vault, token_vault) = amm_info.get_vaults_for_sol(&sol_mint).ok_or_else(|| {
        anyhow::anyhow!(
            "Pump pool {} does not contain SOL. Base: {}, Quote: {}",
            pump_pool,
            amm_info.base_mint,
            amm_info.quote_mint
        )
    })?;

    let pump_fee_wallet = PUMP_FEE_WALLET.to_pubkey();
    let fee_token_wallet = spl_associated_token_account::get_associated_token_address(
        &pump_fee_wallet,
        &amm_info.quote_mint,
    );

    let coin_creator_vault_ata = spl_associated_token_account::get_associated_token_address(
        &amm_info.coin_creator_vault_authority,
        &amm_info.quote_mint,
    );

    let (token_mint, base_mint) = if mint_pubkey == &amm_info.base_mint {
        (amm_info.base_mint, amm_info.quote_mint)
    } else {
        (amm_info.quote_mint, amm_info.base_mint)
    };

    Ok(PumpPool {
        pool: pump_pool,
        token_vault,
        sol_vault,
        fee_token_wallet,
        coin_creator_vault_ata,
        coin_creator_vault_authority: amm_info.coin_creator_vault_authority,
        token_mint,
        base_mint,
    })
}

/// Initializes multiple Pump pools and adds them to pool_data
pub fn initialize_pump_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let pump_pool = fetch_pump_pool(pool_address, mint_pubkey, rpc_client)?;

        info!("Pump pool fetched: {{\n  pool_address: {},\n  token_vault: {},\n  sol_vault: {},\n  fee_token_wallet: {},\n  coin_creator_vault_ata: {},\n  coin_creator_vault_authority: {},\n  token_mint: {},\n  base_mint: {}\n}}", 
            pool_address,
            pump_pool.token_vault,
            pump_pool.sol_vault,
            pump_pool.fee_token_wallet,
            pump_pool.coin_creator_vault_ata,
            pump_pool.coin_creator_vault_authority,
            pump_pool.token_mint,
            pump_pool.base_mint
        );

        pool_data.add_pump_pool(
            pool_address,
            &pump_pool.token_vault.to_string(),
            &pump_pool.sol_vault.to_string(),
            &pump_pool.fee_token_wallet.to_string(),
            &pump_pool.coin_creator_vault_ata.to_string(),
            &pump_pool.coin_creator_vault_authority.to_string(),
            &pump_pool.token_mint.to_string(),
            &pump_pool.base_mint.to_string(),
        )?;

        info!("Pump pool added: {}", pool_address);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::test_utils::{get_test_rpc_client, pool_addresses};

    #[test]
    fn test_fetch_pump_pool() {
        let rpc_client = get_test_rpc_client();

        // 这个是池子的地址
        let pool_address = pool_addresses::PUMP_TEST_POOL;
        // 这个是土狗币的地址
        let mint_pubkey = pool_addresses::PUMP_TEST_TOKEN_MINT.to_pubkey();

        // Now you can test the fetch_pump_pool function directly
        let result = fetch_pump_pool(pool_address, &mint_pubkey, &rpc_client).unwrap();

        assert_eq!(result.pool.to_string(), pool_address);
        assert_eq!(result.base_mint, TokenMint::SOL.to_pubkey());
    }
}
