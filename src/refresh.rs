use crate::constants::{
    addresses::{TokenMint, TokenProgram},
    helpers::ToPubkey,
};
use crate::dex::meteora::constants::{damm_program_id, damm_v2_program_id};
use crate::dex::meteora::pool_damm_v2_info::MeteoraDAmmV2Info;
use crate::dex::meteora::{constants::dlmm_program_id, pool_dlmm_info::DlmmInfo};
use crate::dex::raydium::{
    get_tick_array_pubkeys, raydium_clmm_program_id, raydium_cp_program_id, raydium_program_id,
    PoolState, RaydiumAmmInfo, RaydiumCpAmmInfo,
};
use crate::dex::solfi::constants::solfi_program_id;
use crate::dex::solfi::pool_info::SolfiInfo;
use crate::dex::vertigo::{derive_vault_address, vertigo_program_id, VertigoInfo};
use crate::dex::whirlpool::{
    constants::whirlpool_program_id, pool_clmm::Whirlpool, update_tick_array_accounts_for_onchain,
};
use crate::pools::*;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{error, info};

pub async fn initialize_pool_data(
    mint: &str,
    wallet_account: &str,
    raydium_pools: Option<&Vec<String>>,
    raydium_cp_pools: Option<&Vec<String>>,
    pump_pools_config: Option<&Vec<String>>,
    dlmm_pools: Option<&Vec<String>>,
    whirlpool_pools: Option<&Vec<String>>,
    raydium_clmm_pools: Option<&Vec<String>>,
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
    let token_2022_program_id = TokenProgram::TOKEN_2022.to_pubkey();
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

    // pump_pools_config
    //     .into_iter()
    //     .flatten()
    //     .map(|it| fetch_pump_pool(it, &mint_pubkey, &rpc_client))
    //     .for_each(|r| match r {
    //         Ok(pool) => global_pool_config.pump_pools.push(pool),
    //         Err(e) => error!("Failed to fetch pump pool: {}", e),
    //     });

    raydium_pools
        .map(|pools| {
            initialize_raydium_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

    raydium_cp_pools
        .map(|pools| {
            initialize_raydium_cp_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

    dlmm_pools
        .map(|pools| {
            initialize_dlmm_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

    whirlpool_pools
        .map(|pools| {
            initialize_whirlpool_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

    raydium_clmm_pools
        .map(|pools| {
            initialize_raydium_clmm_pools(pools, &mint_pubkey, &mut global_pool_config, &rpc_client)
        })
        .transpose()?;

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

fn initialize_raydium_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let raydium_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&raydium_pool_pubkey) {
            Ok(account) => {
                if account.owner != raydium_program_id() {
                    error!(
                        "Error: Raydium pool account is not owned by the Raydium program. Expected: {}, Actual: {}",
                        raydium_program_id(), account.owner
                    );
                    return Err(anyhow::anyhow!(
                        "Raydium pool account is not owned by the Raydium program"
                    ));
                }

                match RaydiumAmmInfo::load_checked(&account.data) {
                    Ok(amm_info) => {
                        if amm_info.coin_mint != pool_data.mint
                            && amm_info.pc_mint != pool_data.mint
                        {
                            error!(
                                "Mint {} is not present in Raydium pool {}, skipping",
                                pool_data.mint, raydium_pool_pubkey
                            );
                            return Err(anyhow::anyhow!(
                                "Invalid Raydium pool: {}",
                                raydium_pool_pubkey
                            ));
                        }

                        if amm_info.coin_mint != TokenMint::SOL.to_pubkey()
                            && amm_info.pc_mint != TokenMint::SOL.to_pubkey()
                        {
                            error!("SOL is not present in Raydium pool {}", raydium_pool_pubkey);
                            return Err(anyhow::anyhow!(
                                "SOL is not present in Raydium pool: {}",
                                raydium_pool_pubkey
                            ));
                        }

                        let (sol_vault, token_vault) =
                            if TokenMint::SOL.to_pubkey() == amm_info.coin_mint {
                                (amm_info.coin_vault, amm_info.pc_vault)
                            } else {
                                (amm_info.pc_vault, amm_info.coin_vault)
                            };

                        let (token_mint, base_mint) = if mint_pubkey == &amm_info.coin_mint {
                            (amm_info.coin_mint, amm_info.pc_mint)
                        } else {
                            (amm_info.pc_mint, amm_info.coin_mint)
                        };

                        pool_data.add_raydium_pool(
                            pool_address,
                            &token_vault.to_string(),
                            &sol_vault.to_string(),
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;
                        info!("Raydium pool added: {}", pool_address);
                        info!("    Coin mint: {}", amm_info.coin_mint.to_string());
                        info!("    PC mint: {}", amm_info.pc_mint.to_string());
                        info!("    Token vault: {}", token_vault.to_string());
                        info!("    Sol vault: {}", sol_vault.to_string());
                        info!("    Initialized Raydium pool: {}\n", raydium_pool_pubkey);
                    }
                    Err(e) => {
                        error!(
                            "Error parsing AmmInfo from Raydium pool {}: {:?}",
                            raydium_pool_pubkey, e
                        );
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Raydium pool account {}: {:?}",
                    raydium_pool_pubkey, e
                );
                return Err(anyhow::anyhow!("Error fetching Raydium pool account"));
            }
        }
    }
    Ok(())
}

fn initialize_raydium_cp_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let raydium_cp_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&raydium_cp_pool_pubkey) {
            Ok(account) => {
                if account.owner != raydium_cp_program_id() {
                    error!(
                        "Error: Raydium CP pool account is not owned by the Raydium CP program. Expected: {}, Actual: {}",
                        raydium_cp_program_id(), account.owner
                    );
                    return Err(anyhow::anyhow!(
                        "Raydium CP pool account is not owned by the Raydium CP program"
                    ));
                }

                match RaydiumCpAmmInfo::load_checked(&account.data) {
                    Ok(amm_info) => {
                        if amm_info.token_0_mint != pool_data.mint
                            && amm_info.token_1_mint != pool_data.mint
                        {
                            error!(
                                "Mint {} is not present in Raydium CP pool {}, skipping",
                                pool_data.mint, raydium_cp_pool_pubkey
                            );
                            return Err(anyhow::anyhow!(
                                "Invalid Raydium CP pool: {}",
                                raydium_cp_pool_pubkey
                            ));
                        }

                        let (sol_vault, token_vault) =
                            if TokenMint::SOL.to_pubkey() == amm_info.token_0_mint {
                                (amm_info.token_0_vault, amm_info.token_1_vault)
                            } else if TokenMint::SOL.to_pubkey() == amm_info.token_1_mint {
                                (amm_info.token_1_vault, amm_info.token_0_vault)
                            } else {
                                error!(
                                    "SOL is not present in Raydium CP pool {}",
                                    raydium_cp_pool_pubkey
                                );
                                return Err(anyhow::anyhow!(
                                    "SOL is not present in Raydium CP pool: {}",
                                    raydium_cp_pool_pubkey
                                ));
                            };

                        let (token_mint, base_mint) = if mint_pubkey == &amm_info.token_0_mint {
                            (amm_info.token_0_mint, amm_info.token_1_mint)
                        } else {
                            (amm_info.token_1_mint, amm_info.token_0_mint)
                        };

                        pool_data.add_raydium_cp_pool(
                            pool_address,
                            &token_vault.to_string(),
                            &sol_vault.to_string(),
                            &amm_info.amm_config.to_string(),
                            &amm_info.observation_key.to_string(),
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;
                        info!("Raydium CP pool added: {}", pool_address);
                        info!("    Token vault: {}", token_vault.to_string());
                        info!("    Sol vault: {}", sol_vault.to_string());
                        info!("    AMM Config: {}", amm_info.amm_config.to_string());
                        info!(
                            "    Observation Key: {}\n",
                            amm_info.observation_key.to_string()
                        );
                    }
                    Err(e) => {
                        error!(
                            "Error parsing AmmInfo from Raydium CP pool {}: {:?}",
                            raydium_cp_pool_pubkey, e
                        );
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Raydium CP pool account {}: {:?}",
                    raydium_cp_pool_pubkey, e
                );
                return Err(anyhow::anyhow!("Error fetching Raydium CP pool account"));
            }
        }
    }
    Ok(())
}

fn initialize_dlmm_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let dlmm_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&dlmm_pool_pubkey) {
            Ok(account) => {
                if account.owner != dlmm_program_id() {
                    error!(
                        "Error: DLMM pool account is not owned by the DLMM program. Expected: {}, Actual: {}",
                        dlmm_program_id(), account.owner
                    );
                    return Err(anyhow::anyhow!(
                        "DLMM pool account is not owned by the DLMM program"
                    ));
                }

                match DlmmInfo::load_checked(&account.data) {
                    Ok(amm_info) => {
                        let sol_mint = TokenMint::SOL.to_pubkey();
                        let (token_vault, sol_vault) =
                            amm_info.get_token_and_sol_vaults(&pool_data.mint, &sol_mint);

                        let bin_arrays = match amm_info.calculate_bin_arrays(&dlmm_pool_pubkey) {
                            Ok(arrays) => arrays,
                            Err(e) => {
                                error!(
                                    "Error calculating bin arrays for DLMM pool {}: {:?}",
                                    dlmm_pool_pubkey, e
                                );
                                return Err(e);
                            }
                        };

                        let bin_array_strings: Vec<String> =
                            bin_arrays.iter().map(|pubkey| pubkey.to_string()).collect();
                        let bin_array_str_refs: Vec<&str> =
                            bin_array_strings.iter().map(|s| s.as_str()).collect();

                        let (token_mint, base_mint) = if mint_pubkey == &amm_info.token_x_mint {
                            (amm_info.token_x_mint, amm_info.token_y_mint)
                        } else {
                            (amm_info.token_y_mint, amm_info.token_x_mint)
                        };

                        pool_data.add_dlmm_pool(
                            pool_address,
                            &token_vault.to_string(),
                            &sol_vault.to_string(),
                            &amm_info.oracle.to_string(),
                            bin_array_str_refs,
                            None,
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;

                        info!("DLMM pool added: {}", pool_address);
                        info!("    Token X Mint: {}", amm_info.token_x_mint.to_string());
                        info!("    Token Y Mint: {}", amm_info.token_y_mint.to_string());
                        info!("    Token vault: {}", token_vault.to_string());
                        info!("    Sol vault: {}", sol_vault.to_string());
                        info!("    Oracle: {}", amm_info.oracle.to_string());
                        info!("    Active ID: {}", amm_info.active_id);

                        for (i, array) in bin_array_strings.iter().enumerate() {
                            info!("    Bin Array {}: {}", i, array);
                        }
                        info!("");
                    }
                    Err(e) => {
                        error!(
                            "Error parsing AmmInfo from DLMM pool {}: {:?}",
                            dlmm_pool_pubkey, e
                        );
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching DLMM pool account {}: {:?}",
                    dlmm_pool_pubkey, e
                );
                return Err(anyhow::anyhow!("Error fetching DLMM pool account"));
            }
        }
    }
    Ok(())
}

fn initialize_whirlpool_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    for pool_address in pools {
        let whirlpool_pool_pubkey = pool_address.to_pubkey();

        match rpc_client.get_account(&whirlpool_pool_pubkey) {
            Ok(account) => {
                if account.owner != whirlpool_program_id() {
                    error!(
                        "Error: Whirlpool pool account is not owned by the Whirlpool program. Expected: {}, Actual: {}",
                        whirlpool_program_id(), account.owner
                    );
                    return Err(anyhow::anyhow!(
                        "Whirlpool pool account is not owned by the Whirlpool program"
                    ));
                }

                match Whirlpool::try_deserialize(&account.data) {
                    Ok(whirlpool) => {
                        if whirlpool.token_mint_a != pool_data.mint
                            && whirlpool.token_mint_b != pool_data.mint
                        {
                            error!(
                                "Mint {} is not present in Whirlpool pool {}, skipping",
                                pool_data.mint, whirlpool_pool_pubkey
                            );
                            return Err(anyhow::anyhow!(
                                "Invalid Whirlpool pool: {}",
                                whirlpool_pool_pubkey
                            ));
                        }

                        let sol_mint = TokenMint::SOL.to_pubkey();
                        let (sol_vault, token_vault) = if sol_mint == whirlpool.token_mint_a {
                            (whirlpool.token_vault_a, whirlpool.token_vault_b)
                        } else if sol_mint == whirlpool.token_mint_b {
                            (whirlpool.token_vault_b, whirlpool.token_vault_a)
                        } else {
                            error!(
                                "SOL is not present in Whirlpool pool {}",
                                whirlpool_pool_pubkey
                            );
                            return Err(anyhow::anyhow!(
                                "SOL is not present in Whirlpool pool: {}",
                                whirlpool_pool_pubkey
                            ));
                        };

                        let whirlpool_oracle = Pubkey::find_program_address(
                            &[b"oracle", whirlpool_pool_pubkey.as_ref()],
                            &whirlpool_program_id(),
                        )
                        .0;

                        let whirlpool_tick_arrays = update_tick_array_accounts_for_onchain(
                            &whirlpool,
                            &whirlpool_pool_pubkey,
                            &whirlpool_program_id(),
                        );

                        let tick_array_strings: Vec<String> = whirlpool_tick_arrays
                            .iter()
                            .map(|meta| meta.pubkey.to_string())
                            .collect();

                        let tick_array_str_refs: Vec<&str> =
                            tick_array_strings.iter().map(|s| s.as_str()).collect();

                        let (token_mint, base_mint) = if mint_pubkey == &whirlpool.token_mint_a {
                            (whirlpool.token_mint_a, whirlpool.token_mint_b)
                        } else {
                            (whirlpool.token_mint_b, whirlpool.token_mint_a)
                        };

                        pool_data.add_whirlpool_pool(
                            pool_address,
                            &whirlpool_oracle.to_string(),
                            &token_vault.to_string(),
                            &sol_vault.to_string(),
                            tick_array_str_refs,
                            None,
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;

                        info!("Whirlpool pool added: {}", pool_address);
                        info!("    Token mint A: {}", whirlpool.token_mint_a.to_string());
                        info!("    Token mint B: {}", whirlpool.token_mint_b.to_string());
                        info!("    Token vault: {}", token_vault.to_string());
                        info!("    Sol vault: {}", sol_vault.to_string());
                        info!("    Oracle: {}", whirlpool_oracle.to_string());

                        for (i, array) in tick_array_strings.iter().enumerate() {
                            info!("    Tick Array {}: {}", i, array);
                        }
                        info!("");
                    }
                    Err(e) => {
                        error!(
                            "Error parsing Whirlpool data from pool {}: {:?}",
                            whirlpool_pool_pubkey, e
                        );
                        return Err(anyhow::anyhow!("Error parsing Whirlpool data"));
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Whirlpool pool account {}: {:?}",
                    whirlpool_pool_pubkey, e
                );
                return Err(anyhow::anyhow!("Error fetching Whirlpool pool account"));
            }
        }
    }
    Ok(())
}

fn initialize_raydium_clmm_pools(
    pools: &Vec<String>,
    mint_pubkey: &Pubkey,
    pool_data: &mut MintPoolData,
    rpc_client: &RpcClient,
) -> anyhow::Result<()> {
    let raydium_clmm_program_id = raydium_clmm_program_id();

    for pool_address in pools {
        match rpc_client.get_account(&pool_address.to_pubkey()) {
            Ok(account) => {
                if account.owner != raydium_clmm_program_id {
                    error!(
                        "Raydium CLMM pool {} is not owned by the Raydium CLMM program, skipping",
                        pool_address
                    );
                    continue;
                }

                match PoolState::load_checked(&account.data) {
                    Ok(raydium_clmm) => {
                        if raydium_clmm.token_mint_0 != pool_data.mint
                            && raydium_clmm.token_mint_1 != pool_data.mint
                        {
                            error!(
                                "Mint {} is not present in Raydium CLMM pool {}, skipping",
                                pool_data.mint, pool_address
                            );
                            continue;
                        }

                        let sol_mint = TokenMint::SOL.to_pubkey();
                        let (token_vault, sol_vault) = if sol_mint == raydium_clmm.token_mint_0 {
                            (raydium_clmm.token_vault_1, raydium_clmm.token_vault_0)
                        } else if sol_mint == raydium_clmm.token_mint_1 {
                            (raydium_clmm.token_vault_0, raydium_clmm.token_vault_1)
                        } else {
                            error!("SOL is not present in Raydium CLMM pool {}", pool_address);
                            continue;
                        };

                        let tick_array_pubkeys = get_tick_array_pubkeys(
                            &pool_address.to_pubkey(),
                            raydium_clmm.tick_current,
                            raydium_clmm.tick_spacing,
                            &[-1, 0, 1],
                            &raydium_clmm_program_id,
                        )?;

                        let tick_array_strings: Vec<String> = tick_array_pubkeys
                            .iter()
                            .map(|pubkey| pubkey.to_string())
                            .collect();

                        let tick_array_str_refs: Vec<&str> =
                            tick_array_strings.iter().map(|s| s.as_str()).collect();

                        let (token_mint, base_mint) = if mint_pubkey == &raydium_clmm.token_mint_0 {
                            (raydium_clmm.token_mint_0, raydium_clmm.token_mint_1)
                        } else {
                            (raydium_clmm.token_mint_1, raydium_clmm.token_mint_0)
                        };

                        pool_data.add_raydium_clmm_pool(
                            pool_address,
                            &raydium_clmm.amm_config.to_string(),
                            &raydium_clmm.observation_key.to_string(),
                            &token_vault.to_string(),
                            &sol_vault.to_string(),
                            tick_array_str_refs,
                            None,
                            &token_mint.to_string(),
                            &base_mint.to_string(),
                        )?;

                        info!("Raydium CLMM pool added: {}", pool_address);
                        info!(
                            "    Token mint 0: {}",
                            raydium_clmm.token_mint_0.to_string()
                        );
                        info!(
                            "    Token mint 1: {}",
                            raydium_clmm.token_mint_1.to_string()
                        );
                        info!("    Token vault: {}", token_vault.to_string());
                        info!("    Sol vault: {}", sol_vault.to_string());
                        info!("    AMM config: {}", raydium_clmm.amm_config.to_string());
                        info!(
                            "    Observation key: {}",
                            raydium_clmm.observation_key.to_string()
                        );

                        for (i, array) in tick_array_strings.iter().enumerate() {
                            info!("    Tick Array {}: {}", i, array);
                        }
                        info!("");
                    }
                    Err(e) => {
                        error!(
                            "Error parsing Raydium CLMM data from pool {}: {:?}",
                            pool_address, e
                        );
                        continue;
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error fetching Raydium CLMM pool account {}: {:?}",
                    pool_address, e
                );
                continue;
            }
        }
    }
    Ok(())
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

                        let sol_mint = TokenMint::SOL.to_pubkey();
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
                            if TokenMint::SOL.to_pubkey() == meteora_damm_v2_info.base_mint {
                                meteora_damm_v2_info.quote_vault
                            } else {
                                meteora_damm_v2_info.base_vault
                            };

                        let token_sol_vault =
                            if TokenMint::SOL.to_pubkey() == meteora_damm_v2_info.base_mint {
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

                        let token_x_vault = if TokenMint::SOL.to_pubkey() == solfi_info.base_mint {
                            solfi_info.quote_vault
                        } else {
                            solfi_info.base_vault
                        };

                        let token_sol_vault = if TokenMint::SOL.to_pubkey() == solfi_info.base_mint
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
