use crate::constants::{
    helpers::ToPubkey,
};
use crate::dex::meteora::constants::{damm_program_id, damm_v2_program_id};
use crate::dex::meteora::pool_damm_v2_info::MeteoraDAmmV2Info;
use crate::dex::meteora::{constants::dlmm_program_id, pool_dlmm_info::MeteoraDlmmInfo};
use crate::dex::pool_fetch::fetch_pool;
use crate::dex::raydium::{
    _get_tick_array_pubkeys, raydium_clmm_program_id, raydium_cp_program_id, raydium_program_id,
    RaydiumAmmInfo, RaydiumClmmPoolInfo, RaydiumCpAmmInfo,
};
use crate::dex::solfi::constants::solfi_program_id;
use crate::dex::solfi::pool_info::SolfiInfo;
use crate::dex::vertigo::{derive_vault_address, vertigo_program_id, VertigoInfo};
use crate::dex::whirlpool::{
    constants::whirlpool_program_id, get_tick_arrays, pool_clmm::WhirlpoolInfo,
};
use crate::pools::*;
use futures::StreamExt;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{error, info};
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::constant::token_program::TokenProgram;

pub async fn initialize_pool_data(
    mint: &str,
    wallet_account: &str,
    raydium_pools_config: Option<&Vec<String>>,
    raydium_cp_pools_config: Option<&Vec<String>>,
    pump_pools_config: Option<&Vec<String>>,
    meteora_dlmm_pools_config: Option<&Vec<String>>,
    whirlpool_pools_config: Option<&Vec<String>>,
    raydium_clmm_pools_config: Option<&Vec<String>>,
    meteora_damm_pools: Option<&Vec<String>>,
    solfi_pools: Option<&Vec<String>>,
    meteora_damm_v2_pools: Option<&Vec<String>>,
    vertigo_pools: Option<&Vec<String>>,
    rpc_client: Arc<RpcClient>,
) -> anyhow::Result<MintPoolData> {
    info!("Initializing pool data for mint: {}", mint);

    // Fetch mint account to determine token program
    let mint_pubkey = mint.to_pubkey();
    let mint_account = rpc_client.get_account(&mint_pubkey)?;

    // Determine token program based on mint account owner
    let token_2022_program_id = TokenProgram::TOKEN_2022;
    let token_program = if mint_account.owner == spl_token::ID {
        spl_token::ID
    } else if mint_account.owner == token_2022_program_id {
        token_2022_program_id
    } else {
        return Err(anyhow::anyhow!("Unknown token program for mint: {}", mint));
    };

    info!("Detected token program: {}", token_program);
    let mut global_pool_config = MintPoolData::new(mint, wallet_account, token_program)?;
    info!("Pool data initialized for mint: {}", mint);

    pump_pools_config
        .into_iter()
        .flatten()
        .map(|it| fetch_pool(&it.to_pubkey(), &mint_pubkey, &rpc_client))
        .for_each(|r| match r {
            Ok(pool) => global_pool_config.pump_pools.push(pool),
            Err(e) => error!("Failed to fetch pump pool: {}", e),
        });

    raydium_pools_config
        .into_iter()
        .flatten()
        .map(|it| fetch_pool(&it.to_pubkey(), &mint_pubkey, &rpc_client))
        .for_each(|r| match r {
            Ok(pool) => global_pool_config.raydium_pools.push(pool),
            Err(e) => error!("Failed to fetch raydium pool: {}", e),
        });

    raydium_cp_pools_config
        .into_iter()
        .flatten()
        .map(|it| fetch_pool(&it.to_pubkey(), &mint_pubkey, &rpc_client))
        .for_each(|r| match r {
            Ok(pool) => global_pool_config.raydium_cp_pools.push(pool),
            Err(e) => error!("Failed to fetch raydium pool: {}", e),
        });

    raydium_clmm_pools_config
        .into_iter()
        .flatten()
        .map(|it| fetch_pool(&it.to_pubkey(), &mint_pubkey, &rpc_client))
        .for_each(|r| match r {
            Ok(pool) => global_pool_config.raydium_clmm_pools.push(pool),
            Err(e) => error!("Failed to fetch raydium clmm pool: {}", e),
        });

    
    meteora_dlmm_pools_config
        .into_iter()
        .flatten()
        .map(|it| fetch_pool(&it.to_pubkey(), &mint_pubkey, &rpc_client))
        .for_each(|r| match r {
            Ok(pool) => global_pool_config.meteora_dlmm_pools.push(pool),
            Err(e) => error!("Failed to fetch meteor pool: {}", e),
        });

    whirlpool_pools_config
        .into_iter()
        .flatten()
        .map(|it| fetch_pool(&it.to_pubkey(), &mint_pubkey, &rpc_client))
        .for_each(|r| match r {
            Ok(pool) => global_pool_config.whirlpool_pools.push(pool),
            Err(e) => error!("Failed to fetch whirlpool pool: {}", e),
        });


    meteora_damm_pools
        .map(|pools| {
            initialize_meteora_damm_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

    meteora_damm_v2_pools
        .map(|pools| {
            initialize_meteora_damm_v2_pools(
                pools,
                &mint_pubkey,
                &mut global_pool_config,
                &rpc_client,
            )
        })
        .transpose()?;

    solfi_pools
        .map(|pools| {
            initialize_solfi_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

    vertigo_pools
        .map(|pools| {
            initialize_vertigo_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

    Ok(global_pool_config)
}


fn initialize_meteora_damm_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let meteora_damm_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&meteora_damm_pool_pubkey) {
            Ok(account) => {
                if account.owner != damm_program_id() {
                    error!(
                        "Error: Meteora DAMM pool account is not owned by the Meteora DAMM program. Expected: {}, Actual: {}",
                        damm_program_id(), account.owner
                    );
                    return Err(anyhow::anyhow!(
                        "Meteora DAMM pool account is not owned by the Meteora DAMM program"
                    ));
                }

                match meteora_damm_cpi::Pool::deserialize_unchecked(&account.data) {
                    Ok(pool) => {
                        if pool.token_a_mint != pool_data.mint
                            && pool.token_b_mint != pool_data.mint
                        {
                            error!(
                                "Mint {} is not present in Meteora DAMM pool {}, skipping",
                                pool_data.mint, meteora_damm_pool_pubkey
                            );
                            return Err(anyhow::anyhow!(
                                "Invalid Meteora DAMM pool: {}",
                                meteora_damm_pool_pubkey
                            ));
                        }

                        let sol_mint = Mints::WSOL;
                        if pool.token_a_mint != sol_mint && pool.token_b_mint != sol_mint {
                            error!(
                                "SOL is not present in Meteora DAMM pool {}",
                                meteora_damm_pool_pubkey
                            );
                            return Err(anyhow::anyhow!(
                                "SOL is not present in Meteora DAMM pool: {}",
                                meteora_damm_pool_pubkey
                            ));
                        }

                        let (x_vault, sol_vault) = if sol_mint == pool.token_a_mint {
                            (pool.b_vault, pool.a_vault)
                        } else {
                            (pool.a_vault, pool.b_vault)
                        };

                        let x_vault_data = rpc_client.get_account(&x_vault)?;
                        let sol_vault_data = rpc_client.get_account(&sol_vault)?;

                        let x_vault_obj = meteora_vault_cpi::Vault::deserialize_unchecked(
                            &mut x_vault_data.data.as_slice(),
                        )?;
                        let sol_vault_obj = meteora_vault_cpi::Vault::deserialize_unchecked(
                            &mut sol_vault_data.data.as_slice(),
                        )?;

                        let x_token_vault = x_vault_obj.token_vault;
                        let sol_token_vault = sol_vault_obj.token_vault;
                        let x_lp_mint = x_vault_obj.lp_mint;
                        let sol_lp_mint = sol_vault_obj.lp_mint;

                        let (x_pool_lp, sol_pool_lp) = if sol_mint == pool.token_a_mint {
                            (pool.b_vault_lp, pool.a_vault_lp)
                        } else {
                            (pool.a_vault_lp, pool.b_vault_lp)
                        };

                        let (x_admin_fee, sol_admin_fee) = if sol_mint == pool.token_a_mint {
                            (pool.admin_token_b_fee, pool.admin_token_a_fee)
                        } else {
                            (pool.admin_token_a_fee, pool.admin_token_b_fee)
                        };

                        let (token_mint, base_mint) = if mint_pubkey == &pool.token_a_mint {
                            (pool.token_a_mint, pool.token_b_mint)
                        } else {
                            (pool.token_b_mint, pool.token_a_mint)
                        };

                        pool_data.add_meteora_damm_pool(
                            pool_address,
                            &x_vault.to_string(),
                            &sol_vault.to_string(),
                            &x_token_vault.to_string(),
                            &sol_token_vault.to_string(),
                            &x_lp_mint.to_string(),
                            &sol_lp_mint.to_string(),
                            &x_pool_lp.to_string(),
                            &sol_pool_lp.to_string(),
                            &x_admin_fee.to_string(),
                            &sol_admin_fee.to_string(),
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;

                        info!("Meteora DAMM pool added: {}", pool_address);
                        info!("    Token X vault: {}", x_token_vault.to_string());
                        info!("    SOL vault: {}", sol_token_vault.to_string());
                        info!("    Token X LP mint: {}", x_lp_mint.to_string());
                        info!("    SOL LP mint: {}", sol_lp_mint.to_string());
                        info!("    Token X pool LP: {}", x_pool_lp.to_string());
                        info!("    SOL pool LP: {}", sol_pool_lp.to_string());
                        info!("    Token X admin fee: {}", x_admin_fee.to_string());
                        info!("    SOL admin fee: {}", sol_admin_fee.to_string());
                        info!("");
                    }
                    Err(e) => {
                        error!(
                            "Error parsing Meteora DAMM pool data from pool {}: {:?}",
                            meteora_damm_pool_pubkey, e
                        );
                        return Err(anyhow::anyhow!("Error parsing Meteora DAMM pool data"));
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Meteora DAMM pool account {}: {:?}",
                    meteora_damm_pool_pubkey, e
                );
                return Err(anyhow::anyhow!("Error fetching Meteora DAMM pool account"));
            }
        }
    }
    Ok(())
}

fn initialize_meteora_damm_v2_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let meteora_damm_v2_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&meteora_damm_v2_pool_pubkey) {
            Ok(account) => {
                if account.owner != damm_v2_program_id() {
                    error!("Meteora DAMM V2 pool {} is not owned by the Meteora DAMM V2 program, skipping", pool_address);
                    continue;
                }

                match MeteoraDAmmV2Info::load_checked(&account.data) {
                    Ok(meteora_damm_v2_info) => {
                        info!("Meteora DAMM V2 pool added: {}", pool_address);
                        info!(
                            "    Base mint: {}",
                            meteora_damm_v2_info.base_mint.to_string()
                        );
                        info!(
                            "    Quote mint: {}",
                            meteora_damm_v2_info.quote_mint.to_string()
                        );
                        info!(
                            "    Base vault: {}",
                            meteora_damm_v2_info.base_vault.to_string()
                        );
                        info!(
                            "    Quote vault: {}",
                            meteora_damm_v2_info.quote_vault.to_string()
                        );
                        info!("");
                        let token_x_vault =
                            if Mints::WSOL == meteora_damm_v2_info.base_mint {
                                meteora_damm_v2_info.quote_vault
                            } else {
                                meteora_damm_v2_info.base_vault
                            };

                        let token_sol_vault =
                            if Mints::WSOL == meteora_damm_v2_info.base_mint {
                                meteora_damm_v2_info.base_vault
                            } else {
                                meteora_damm_v2_info.quote_vault
                            };

                        let (token_mint, base_mint) =
                            if mint_pubkey == &meteora_damm_v2_info.base_mint {
                                (
                                    meteora_damm_v2_info.base_mint,
                                    meteora_damm_v2_info.quote_mint,
                                )
                            } else {
                                (
                                    meteora_damm_v2_info.quote_mint,
                                    meteora_damm_v2_info.base_mint,
                                )
                            };

                        pool_data.add_meteora_damm_v2_pool(
                            pool_address,
                            &token_x_vault.to_string(),
                            &token_sol_vault.to_string(),
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;
                    }
                    Err(e) => {
                        error!(
                            "Error parsing Meteora DAMM V2 pool data from pool {}: {:?}",
                            meteora_damm_v2_pool_pubkey, e
                        );
                        continue;
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Meteora DAMM V2 pool account {}: {:?}",
                    meteora_damm_v2_pool_pubkey, e
                );
                continue;
            }
        }
    }
    Ok(())
}

fn initialize_solfi_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let solfi_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&solfi_pool_pubkey) {
            Ok(account) => {
                if account.owner != solfi_program_id() {
                    error!(
                        "Solfi pool {} is not owned by the Solfi program, skipping",
                        pool_address
                    );
                    continue;
                }

                match SolfiInfo::load_checked(&account.data) {
                    Ok(solfi_info) => {
                        info!("Solfi pool added: {}", pool_address);
                        info!("    Base mint: {}", solfi_info.base_mint.to_string());
                        info!("    Quote mint: {}", solfi_info.quote_mint.to_string());
                        info!("    Base vault: {}", solfi_info.base_vault.to_string());
                        info!("    Quote vault: {}", solfi_info.quote_vault.to_string());

                        let token_x_vault = if Mints::WSOL == solfi_info.base_mint {
                            solfi_info.quote_vault
                        } else {
                            solfi_info.base_vault
                        };

                        let token_sol_vault = if Mints::WSOL == solfi_info.base_mint
                        {
                            solfi_info.base_vault
                        } else {
                            solfi_info.quote_vault
                        };

                        let (token_mint, base_mint) = if mint_pubkey == &solfi_info.base_mint {
                            (solfi_info.base_mint, solfi_info.quote_mint)
                        } else {
                            (solfi_info.quote_mint, solfi_info.base_mint)
                        };

                        pool_data.add_solfi_pool(
                            pool_address,
                            &token_x_vault.to_string(),
                            &token_sol_vault.to_string(),
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;
                    }
                    Err(e) => {
                        error!(
                            "Error parsing Solfi pool data from pool {}: {:?}",
                            pool_address, e
                        );
                        continue;
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Solfi pool account {}: {:?}",
                    solfi_pool_pubkey, e
                );
                continue;
            }
        }
    }
    Ok(())
}

fn initialize_vertigo_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let vertigo_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&vertigo_pool_pubkey) {
            Ok(account) => {
                if account.owner != vertigo_program_id() {
                    error!(
                        "Error: Vertigo pool account is not owned by the Vertigo program. Expected: {}, Actual: {}",
                        vertigo_program_id(), account.owner
                    );
                    return Err(anyhow::anyhow!(
                        "Vertigo pool account is not owned by the Vertigo program"
                    ));
                }

                match VertigoInfo::load_checked(&account.data, &vertigo_pool_pubkey) {
                    Ok(vertigo_info) => {
                        info!("Vertigo pool added: {}", pool_address);
                        info!("    Mint A: {}", vertigo_info.mint_a.to_string());
                        info!("    Mint B: {}", vertigo_info.mint_b.to_string());

                        let base_mint = pool_data.mint.to_string();

                        let non_base_vault = if base_mint == vertigo_info.mint_a.to_string() {
                            derive_vault_address(&vertigo_pool_pubkey, &vertigo_info.mint_b).0
                        } else {
                            derive_vault_address(&vertigo_pool_pubkey, &vertigo_info.mint_a).0
                        };
                        let base_vault = if base_mint == vertigo_info.mint_a.to_string() {
                            derive_vault_address(&vertigo_pool_pubkey, &vertigo_info.mint_a).0
                        } else {
                            derive_vault_address(&vertigo_pool_pubkey, &vertigo_info.mint_b).0
                        };

                        let token_x_vault = base_vault;
                        let token_sol_vault = non_base_vault;

                        info!("    Token X Vault: {}", token_x_vault.to_string());
                        info!("    Token SOL Vault: {}", token_sol_vault.to_string());
                        info!("");

                        let (token_mint, base_mint) = if mint_pubkey == &vertigo_info.mint_a {
                            (vertigo_info.mint_a, vertigo_info.mint_b)
                        } else {
                            (vertigo_info.mint_b, vertigo_info.mint_a)
                        };

                        pool_data.add_vertigo_pool(
                            pool_address,
                            &vertigo_info.pool.to_string(),
                            &token_x_vault.to_string(),
                            &token_sol_vault.to_string(),
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;
                    }
                    Err(e) => {
                        error!(
                            "Error parsing Vertigo pool data from pool {}: {:?}",
                            vertigo_pool_pubkey, e
                        );
                        continue;
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Vertigo pool account {}: {:?}",
                    vertigo_pool_pubkey, e
                );
                continue;
            }
        }
    }
    Ok(())
}
