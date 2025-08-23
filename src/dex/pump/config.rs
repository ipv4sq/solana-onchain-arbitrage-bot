use crate::arb::global::constant::mint::Mints;
use crate::constants::helpers::ToPubkey;
use crate::constants::utils::expect_owner;
use crate::dex::pool_fetch::PoolFetch;
use crate::dex::pump::{PumpAmmInfo, PUMP_FEE_WALLET, PUMP_PROGRAM_ID};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct PumpPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub fee_token_wallet: Pubkey,
    pub coin_creator_vault_ata: Pubkey,
    pub coin_creator_vault_authority: Pubkey,
    // no idea why it is not being used.
    pub token_mint: Pubkey,
    // Strange here, 这个地方, 在pool里, base是土狗, quote是sol, 但是在这里base是sol, 可能是想要
    // 按照sol来套利
    pub base_mint: Pubkey,
}

impl PoolFetch for PumpPool {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> anyhow::Result<Self> {
        let pump_pool = pool;
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

        let sol_mint = Mints::WSOL;
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

        let (token_mint, base_mint) = if mint == &amm_info.base_mint {
            (amm_info.base_mint, amm_info.quote_mint)
        } else {
            (amm_info.quote_mint, amm_info.base_mint)
        };

        Ok(PumpPool {
            pool: *pump_pool,
            token_vault,
            sol_vault,
            fee_token_wallet,
            coin_creator_vault_ata,
            coin_creator_vault_authority: amm_info.coin_creator_vault_authority,
            token_mint,
            base_mint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::test_utils::{get_test_rpc_client, pool_addresses};

    #[test]
    fn test_fetch_pump_pool() {
        let rpc_client = get_test_rpc_client();

        // 这个是池子的地址
        let pool_address = pool_addresses::PUMP_TEST_POOL.to_pubkey();
        // 这个是土狗币的地址
        let mint_pubkey = pool_addresses::PUMP_TEST_TOKEN_MINT.to_pubkey();

        // Now you can test the fetch_pump_pool function directly

        let result = PumpPool::fetch(&pool_address, &mint_pubkey, &rpc_client).unwrap();
        assert_eq!(result.pool.to_string(), pool_address.to_string());
        assert_eq!(result.base_mint, Mints::WSOL);
    }
}
