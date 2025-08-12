use crate::constants::{
    addresses::TokenMint,
    helpers::ToPubkey,
    utils::expect_owner,
};
use crate::dex::pump::{pump_fee_wallet, pump_program_id, PumpAmmInfo};
use crate::pools::MintPoolData;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account;
use tracing::info;

pub fn initialize_pump_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let pump_pool_pubkey = pool_address.to_pubkey();

        let account = rpc_client.get_account(&pump_pool_pubkey).map_err(|e| {
            anyhow::anyhow!("Error fetching Pump pool account {pump_pool_pubkey}: {e:?}")
        })?;

        expect_owner(&pump_pool_pubkey, &account, &pump_program_id())?;

        let amm_info = PumpAmmInfo::load_checked(&account.data).map_err(|e| {
            anyhow::anyhow!(
                "Error parsing AmmInfo from Pump pool {}: {:?}",
                pump_pool_pubkey,
                e
            )
        })?;

        let sol_mint = TokenMint::SOL.to_pubkey();
        let (sol_vault, token_vault) = if sol_mint == amm_info.base_mint {
            (amm_info.pool_base_token_account, amm_info.pool_quote_token_account)
        } else if sol_mint == amm_info.quote_mint {
            (amm_info.pool_quote_token_account, amm_info.pool_base_token_account)
        } else {
            return Err(anyhow::anyhow!(
                "Pump pool {} does not contain SOL. Base: {}, Quote: {}",
                pump_pool_pubkey, amm_info.base_mint, amm_info.quote_mint
            ));
        };

        let fee_token_wallet = spl_associated_token_account::get_associated_token_address(
            &pump_fee_wallet(),
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

        pool_data.add_pump_pool(
            pool_address,
            &token_vault.to_string(),
            &sol_vault.to_string(),
            &fee_token_wallet.to_string(),
            &coin_creator_vault_ata.to_string(),
            &amm_info.coin_creator_vault_authority.to_string(),
            &token_mint.to_string(),
            &base_mint.to_string(),
        )?;

        info!("Pump pool added: {}", pool_address);
        info!("    Base mint: {}", amm_info.base_mint.to_string());
        info!("    Quote mint: {}", amm_info.quote_mint.to_string());
        info!("    Token vault: {}", token_vault.to_string());
        info!("    Sol vault: {}", sol_vault.to_string());
        info!("    Fee token wallet: {}", fee_token_wallet.to_string());
        info!(
            "    Coin creator vault ata: {}",
            coin_creator_vault_ata.to_string()
        );
        info!(
            "    Coin creator vault authority: {}",
            amm_info.coin_creator_vault_authority.to_string()
        );
        info!("    Initialized Pump pool: {}\n", pump_pool_pubkey);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pump_pool_initialization() {
        // TODO: Add tests for pump pool initialization
        // Test cases:
        // 1. Valid pool with SOL as base
        // 2. Valid pool with SOL as quote  
        // 3. Invalid pool without SOL
        // 4. Invalid pool with wrong owner
        // 5. Invalid pool data format
    }
}