use crate::config::Config;
use crate::dex::raydium::{raydium_authority, raydium_cp_authority};
use crate::dex::solfi::constants::solfi_program_id;
use crate::dex::vertigo::constants::vertigo_program_id;
use crate::pools::MintPoolData;
use crate::util::random_select;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::Instruction;
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash::Hash;
use solana_sdk::message::v0::Message;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::arb::global::constant::mev_bot::MevBot;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::state::rpc;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::dex::meteora::constants::{
    damm_program_id, damm_v2_event_authority, damm_v2_pool_authority, damm_v2_program_id,
    dlmm_event_authority, dlmm_program_id, vault_program_id,
};
use crate::dex::pump::{PUMP_FEE_WALLET, PUMP_PROGRAM_ID};
use crate::dex::raydium::constants::{
    raydium_clmm_program_id, raydium_cp_program_id, raydium_program_id,
};
use crate::dex::whirlpool::constants::whirlpool_program_id;
use clap::Error;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use spl_associated_token_account::ID as associated_token_program_id;
use spl_token::ID as token_program_id;
use std::str::FromStr;

pub async fn build_and_send_transaction(
    wallet_kp: &Keypair,
    config: &Config,
    mint_pool_data: &MintPoolData,
    rpc_clients: &[Arc<RpcClient>],
    blockhash: Hash,
    address_lookup_table_accounts: &[AddressLookupTableAccount],
) -> anyhow::Result<Vec<Signature>> {
    let enable_flashloan = config.flashloan.as_ref().map_or(false, |k| k.enabled);

    let compute_unit_limit = config.bot.compute_unit_limit;
    let mut instructions = vec![];
    // Add a random number here to make each transaction unique
    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(
        compute_unit_limit + rand::random::<u32>() % 1000,
    );
    instructions.push(compute_budget_ix);

    let compute_unit_price = config.spam.as_ref().map_or(1000, |s| s.compute_unit_price);
    let compute_budget_price_ix =
        ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price);
    instructions.push(compute_budget_price_ix);

    let swap_ix = create_swap_instruction(
        wallet_kp,
        mint_pool_data,
        compute_unit_limit,
        enable_flashloan,
    )?;

    let mut all_instructions = instructions.clone();

    debug!("Adding swap instruction");
    all_instructions.push(swap_ix);

    let message = Message::try_compile(
        &wallet_kp.pubkey(),
        &all_instructions,
        address_lookup_table_accounts,
        blockhash,
    )?;

    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[wallet_kp],
    )?;

    let max_retries = config
        .spam
        .as_ref()
        .and_then(|s| s.max_retries)
        .unwrap_or(3);

    let mut signatures = Vec::new();

    for (i, client) in rpc_clients.iter().enumerate() {
        debug!("Sending transaction through RPC client {}", i);

        let signature = match rpc::send_tx_with_retry(&tx, max_retries).await {
            Ok(sig) => sig,
            Err(e) => {
                error!("Failed to send transaction through RPC client {}: {}", i, e);
                continue;
            }
        };

        info!(
            "Transaction sent successfully through RPC client {}: {}",
            i, signature
        );
        signatures.push(signature);
    }

    Ok(signatures)
}

fn build_tx(
    config: &Config,
    wallet_kp: &Keypair,
    mint_pool_data: &MintPoolData,
    address_lookup_table_accounts: &[AddressLookupTableAccount],
    blockhash: Hash,
) -> anyhow::Result<VersionedTransaction> {
    let (mut instructions, limit) = gas_instructions(config);
    let use_flashloan = config.flashloan.as_ref().map_or(false, |k| k.enabled);
    let swap_ix = create_swap_instruction(wallet_kp, mint_pool_data, limit, use_flashloan)?;

    instructions.push(swap_ix);

    let message = Message::try_compile(
        &wallet_kp.pubkey(),
        &instructions,
        address_lookup_table_accounts,
        blockhash,
    )?;

    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[wallet_kp],
    )?;
    Ok(tx)
}

fn gas_instructions(config: &Config) -> (Vec<Instruction>, u32) {
    let compute_limit = config.bot.compute_unit_limit;
    let unit_price = config.spam.as_ref().map_or(1000, |s| s.compute_unit_price);
    let seed = rand::random::<u32>() % 1000;
    let compute_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(compute_limit + seed);
    let unit_price_ix = ComputeBudgetInstruction::set_compute_unit_price(unit_price);

    (vec![compute_limit_ix, unit_price_ix], compute_limit + seed)
}

/// Helper function to derive the vault token account PDA address for a given mint
pub fn derive_vault_token_account(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault_token_account", mint.as_ref()], program_id)
}

// See https://docs.solanamevbot.com/home/onchain-bot/onchain-program for more information
fn create_swap_instruction(
    wallet_kp: &Keypair,
    mint_pool_data: &MintPoolData,
    compute_unit_limit: u32,
    use_flashloan: bool,
) -> anyhow::Result<Instruction> {
    debug!("Creating swap instruction for all DEX types");

    let executor_program_id = MevBot::EMV_BOT_PROGRAM;

    let fee_collector = if use_flashloan {
        MevBot::FLASHLOAN_FEE_ACCOUNT
    } else {
        let fee_accounts = [
            MevBot::NON_FLASHLOAN_ACCOUNT_1,
            MevBot::NON_FLASHLOAN_ACCOUNT_2,
            MevBot::NON_FLASHLOAN_ACCOUNT_3,
        ];
        *random_select(&fee_accounts).expect("fee_accounts should not be empty")
    };

    let pump_global_config = Pubkey::from_str("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw")?;
    let pump_authority = Pubkey::from_str("GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR")?;
    let sysvar_instructions = Pubkey::from_str("Sysvar1nstructions1111111111111111111111111")?;
    let memo_program = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")?;

    let wallet = wallet_kp.pubkey();
    let sol_mint_pubkey = Mints::WSOL;
    let wallet_sol_account = mint_pool_data.wallet_wsol_account;
    let usdc_mint = Mints::USDC;

    let mut accounts = vec![
        AccountMeta::new(wallet, true), // 0. Wallet (signer)
        AccountMeta::new_readonly(sol_mint_pubkey, false), // 1. SOL mint
        AccountMeta::new(fee_collector, false), // 2. Fee collector
        AccountMeta::new(wallet_sol_account, false), // 3. Wallet SOL account
        AccountMeta::new_readonly(token_program_id, false), // 4. Token program
        AccountMeta::new_readonly(system_program::ID, false), // 5. System program
        AccountMeta::new_readonly(associated_token_program_id, false), // 6. Associated Token program
    ];

    // Determine the base mint for flashloan if needed
    let flashloan_base_mint = if use_flashloan {
        // For flashloan, we need a common base mint across all pools
        // Check if all pools use SOL as base mint
        let mut all_sol_base = true;
        let mut all_usdc_base = true;

        // Check all pool types to see their base mints
        for pool in &mint_pool_data.raydium_pools {
            if pool.base_mint != sol_mint_pubkey {
                all_sol_base = false;
            }
            if pool.base_mint != usdc_mint {
                all_usdc_base = false;
            }
        }
        for pool in &mint_pool_data.raydium_cp_pools {
            if pool.base_mint != sol_mint_pubkey {
                all_sol_base = false;
            }
            if pool.base_mint != usdc_mint {
                all_usdc_base = false;
            }
        }
        // Add other pool type checks as needed...

        if all_sol_base {
            sol_mint_pubkey
        } else if all_usdc_base {
            usdc_mint
        } else {
            // Mixed base mints - default to SOL for now
            sol_mint_pubkey
        }
    } else {
        sol_mint_pubkey
    };

    if use_flashloan {
        accounts.push(AccountMeta::new_readonly(MevBot::FLASHLOAN_ACCOUNT, false));
        let token_pda = derive_vault_token_account(&MevBot::EMV_BOT_PROGRAM, &flashloan_base_mint);
        accounts.push(AccountMeta::new(token_pda.0, false));
    }

    // Check for mixed mode (USDC base)
    let mut has_usdc_base = false;

    // Check all pools to see if any have USDC as base mint
    for pool in &mint_pool_data.raydium_pools {
        if pool.base_mint == usdc_mint {
            has_usdc_base = true;
            break;
        }
    }
    if !has_usdc_base {
        for pool in &mint_pool_data.raydium_cp_pools {
            if pool.base_mint == usdc_mint {
                has_usdc_base = true;
                break;
            }
        }
    }
    // Check other pool types as needed...

    // If mixed mode is detected, add the required accounts
    if has_usdc_base {
        let wallet_usdc_account =
            spl_associated_token_account::get_associated_token_address(&wallet, &usdc_mint);
        let raydium_sol_usdc_pool =
            Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap();
        let raydium_usdc_vault =
            Pubkey::from_str("HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz").unwrap();
        let raydium_sol_vault =
            Pubkey::from_str("DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz").unwrap();

        accounts.push(AccountMeta::new_readonly(usdc_mint, false));
        accounts.push(AccountMeta::new(wallet_usdc_account, false));
        accounts.push(AccountMeta::new_readonly(raydium_program_id(), false));
        accounts.push(AccountMeta::new_readonly(raydium_authority(), false));
        accounts.push(AccountMeta::new_readonly(sysvar_instructions, false));
        accounts.push(AccountMeta::new(raydium_sol_usdc_pool, false));
        accounts.push(AccountMeta::new(raydium_usdc_vault, false));
        accounts.push(AccountMeta::new(raydium_sol_vault, false));
    }

    // Add token mint and pools
    accounts.push(AccountMeta::new_readonly(mint_pool_data.mint, false));
    accounts.push(AccountMeta::new_readonly(
        mint_pool_data.token_program,
        false,
    )); // Token program (SPL Token or Token 2022)
    let wallet_x_account =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &wallet,
            &mint_pool_data.mint,
            &mint_pool_data.token_program,
        );
    accounts.push(AccountMeta::new(wallet_x_account, false));

    // Add Raydium pools
    for pool in &mint_pool_data.raydium_pools {
        accounts.push(AccountMeta::new_readonly(raydium_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(raydium_authority(), false));
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new(pool.token_vault, false));
        accounts.push(AccountMeta::new(pool.sol_vault, false));
    }

    // Add Raydium CP pools
    for pool in &mint_pool_data.raydium_cp_pools {
        accounts.push(AccountMeta::new_readonly(raydium_cp_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(raydium_cp_authority(), false));
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new_readonly(pool.amm_config, false));
        accounts.push(AccountMeta::new(pool.token_vault, false));
        accounts.push(AccountMeta::new(pool.sol_vault, false));
        accounts.push(AccountMeta::new(pool.observation, false));
    }

    let pump_program_id = PUMP_PROGRAM_ID.to_pubkey();
    let pump_fee_wallet = PUMP_FEE_WALLET.to_pubkey();
    // Add Pump pools
    for pool in &mint_pool_data.pump_pools {
        accounts.push(AccountMeta::new_readonly(pump_program_id, false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(pump_global_config, false));
        accounts.push(AccountMeta::new_readonly(pump_authority, false));
        accounts.push(AccountMeta::new(pump_fee_wallet, false));
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new(pool.token_vault, false));
        accounts.push(AccountMeta::new(pool.sol_vault, false));
        accounts.push(AccountMeta::new(pool.fee_token_wallet, false));
        accounts.push(AccountMeta::new(pool.coin_creator_vault_ata, false));
        accounts.push(AccountMeta::new_readonly(
            pool.coin_creator_vault_authority,
            false,
        ));
        let pump_program_id =
            Pubkey::from_str("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA").unwrap();
        let (global_volume_accumulator, _) =
            Pubkey::find_program_address(&[b"global_volume_accumulator"], &pump_program_id);
        let (user_volume_accumulator, _) = Pubkey::find_program_address(
            &[b"user_volume_accumulator", wallet.as_ref()],
            &pump_program_id,
        );
        accounts.push(AccountMeta::new(global_volume_accumulator, false));
        accounts.push(AccountMeta::new(user_volume_accumulator, false));
    }

    // Add DLMM pairs
    for pair in &mint_pool_data.meteora_dlmm_pools {
        accounts.push(AccountMeta::new_readonly(dlmm_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pair.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(dlmm_event_authority(), false));
        if let Some(memo_program) = pair.memo_program {
            accounts.push(AccountMeta::new_readonly(memo_program, false));
        }
        accounts.push(AccountMeta::new(pair.pair, false));
        accounts.push(AccountMeta::new(pair.token_vault, false));
        accounts.push(AccountMeta::new(pair.sol_vault, false));
        accounts.push(AccountMeta::new(pair.oracle, false));
        for bin_array in &pair.bin_arrays {
            accounts.push(AccountMeta::new(*bin_array, false));
        }
    }

    // Add Whirlpool pools
    for pool in &mint_pool_data.whirlpool_pools {
        accounts.push(AccountMeta::new_readonly(whirlpool_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(memo_program, false)); // Always add memo program for Whirlpool
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new(pool.oracle, false)); // Oracle NEEDS to be writable for Whirlpool
        accounts.push(AccountMeta::new(pool.x_vault, false));
        accounts.push(AccountMeta::new(pool.y_vault, false));
        for tick_array in &pool.tick_arrays {
            accounts.push(AccountMeta::new(*tick_array, false));
        }
    }

    // Add Raydium CLMM pools
    for pool in &mint_pool_data.raydium_clmm_pools {
        accounts.push(AccountMeta::new_readonly(raydium_clmm_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        if let Some(memo_program) = pool.memo_program {
            accounts.push(AccountMeta::new_readonly(memo_program, false));
        }
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new_readonly(pool.amm_config, false));
        accounts.push(AccountMeta::new(pool.observation_state, false));
        accounts.push(AccountMeta::new(pool.bitmap_extension, false));
        accounts.push(AccountMeta::new(pool.x_vault, false));
        accounts.push(AccountMeta::new(pool.y_vault, false));
        for tick_array in &pool.tick_arrays {
            accounts.push(AccountMeta::new(*tick_array, false));
        }
    }

    // Add Meteora DAMM pools
    for pool in &mint_pool_data.meteora_damm_pools {
        accounts.push(AccountMeta::new_readonly(damm_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(vault_program_id(), false));
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new(pool.token_x_vault, false));
        accounts.push(AccountMeta::new(pool.token_sol_vault, false));
        accounts.push(AccountMeta::new(pool.token_x_token_vault, false));
        accounts.push(AccountMeta::new(pool.token_sol_token_vault, false));
        accounts.push(AccountMeta::new(pool.token_x_lp_mint, false));
        accounts.push(AccountMeta::new(pool.token_sol_lp_mint, false));
        accounts.push(AccountMeta::new(pool.token_x_pool_lp, false));
        accounts.push(AccountMeta::new(pool.token_sol_pool_lp, false));
        accounts.push(AccountMeta::new(pool.admin_token_fee_x, false));
        accounts.push(AccountMeta::new(pool.admin_token_fee_sol, false));
    }

    // Add Meteora DAMM V2 pools
    for pool in &mint_pool_data.meteora_damm_v2_pools {
        accounts.push(AccountMeta::new_readonly(damm_v2_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(damm_v2_event_authority(), false));
        accounts.push(AccountMeta::new_readonly(damm_v2_pool_authority(), false));
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new(pool.token_x_vault, false));
        accounts.push(AccountMeta::new(pool.token_sol_vault, false));
    }

    // Add Solfi pools
    for pool in &mint_pool_data.solfi_pools {
        accounts.push(AccountMeta::new_readonly(solfi_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new_readonly(sysvar_instructions, false));
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new(pool.token_x_vault, false));
        accounts.push(AccountMeta::new(pool.token_sol_vault, false));
    }

    // Add Vertigo pools
    for pool in &mint_pool_data.vertigo_pools {
        accounts.push(AccountMeta::new_readonly(vertigo_program_id(), false));
        accounts.push(AccountMeta::new_readonly(pool.base_mint, false)); // V9: Add base mint
        accounts.push(AccountMeta::new(pool.pool, false));
        accounts.push(AccountMeta::new_readonly(pool.pool_owner, false));
        accounts.push(AccountMeta::new(pool.token_x_vault, false));
        accounts.push(AccountMeta::new(pool.token_sol_vault, false));
    }

    // Create instruction data
    let mut data = vec![28u8];

    let minimum_profit: u64 = 0;
    // When true, the bot will not fail the transaction even when it can't find a profitable arbitrage. It will just do nothing and succeed.
    let no_failure_mode = false;

    data.extend_from_slice(&minimum_profit.to_le_bytes());
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    data.extend_from_slice(if no_failure_mode { &[1] } else { &[0] });
    data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    data.extend_from_slice(if use_flashloan { &[1] } else { &[0] });

    Ok(Instruction {
        program_id: executor_program_id,
        accounts,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_vault_token_account() {
        let program_id = Pubkey::from_str("MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz").unwrap();
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();

        let (pda, bump) = derive_vault_token_account(&program_id, &sol_mint);
        println!("PDA: {}, Bump: {}", pda, bump);
    }
}
