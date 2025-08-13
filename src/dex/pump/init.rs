use crate::constants::{addresses::TokenMint, helpers::ToPubkey, utils::expect_owner};
use crate::dex::pump::{config, PumpAmmInfo, PumpPool, PUMP_FEE_WALLET, PUMP_PROGRAM_ID};
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
        let pump_pool = config::fetch_pump_pool(pool_address, mint_pubkey, rpc_client)?;

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

        pool_data.pump_pools.push(pump_pool);

        info!("Pump pool added: {}", pool_address);
    }
    Ok(())
}


