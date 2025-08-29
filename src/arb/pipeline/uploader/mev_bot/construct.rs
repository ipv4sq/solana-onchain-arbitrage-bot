use crate::arb::convention::chain::util::simulation::SimulationResult;
use crate::arb::convention::pool::interface::{InputAccountUtil, PoolDataLoader};
use crate::arb::convention::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::convention::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::convention::pool::pump_amm::input_account::PumpAmmInputAccounts;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::convention::pool::util::{ata, ata_sol_token};
use crate::arb::database::mev_simulation_log::model::{
    MevSimulationLogDetails, MevSimulationLogParams, SimulationAccount,
};
use crate::arb::database::mev_simulation_log::repository::MevSimulationLogRepository;
use crate::arb::database::mint_record::repository::MintRecordRepository;
use crate::arb::global::constant::mev_bot::MevBot;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::constant::token_program::TokenProgram;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::global::trace::types::Trace;
use crate::arb::util::alias::{MintAddress, TokenProgramAddress};
use crate::arb::util::traits::account_meta::ToAccountMeta;
use crate::util::random_select;
use anyhow::{anyhow, Result};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash::Hash;
use solana_sdk::message::v0::Message;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::VersionedTransaction;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use std::io::sink;
use tracing::{error, info};

const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 500_000;
const DEFAULT_UNIT_PRICE: u64 = 500_000;

pub async fn build_tx(
    wallet: &Keypair,
    minor_mint: &Pubkey,
    compute_unit_limit: u32,
    unit_price: u64,
    pools: &[AnyPoolConfig],
    blockhash: Hash,
    alts: &[AddressLookupTableAccount],
    minimum_profit: u64,
    never_abort: bool,
    include_create_token_account_ix: bool,
) -> Result<VersionedTransaction> {
    let (mut instructions, limit) = gas_instructions(compute_unit_limit, unit_price);
    let wallet_pub = wallet.pubkey();
    let mint_token_program = MintRecordRepository::get_mint_from_cache(minor_mint)
        .await?
        .ok_or_else(|| anyhow!("Mint not found in cache: {}", minor_mint))?
        .program
        .0;

    if include_create_token_account_ix {
        instructions.push(ensure_token_account_exists(
            &wallet_pub,
            minor_mint,
            &mint_token_program,
        ))
    }

    let swap_ix = create_invoke_mev_instruction(
        &wallet.pubkey(),
        minor_mint,
        &mint_token_program,
        compute_unit_limit,
        pools,
        minimum_profit,
        never_abort,
    )?;
    instructions.push(swap_ix);

    let message = Message::try_compile(&wallet.pubkey(), &instructions, alts, blockhash)?;
    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[wallet],
    )?;
    Ok(tx)
}

pub fn create_invoke_mev_instruction(
    signer: &Pubkey,
    minor_mint: &MintAddress,
    token_program: &TokenProgramAddress,
    compute_unit_limit: u32,
    pools: &[AnyPoolConfig],
    minimum_profit: u64,
    never_abort: bool,
) -> Result<Instruction> {
    let use_flashloan = true;
    let fee_account = fee_collector(use_flashloan);
    let mut accounts = vec![
        signer.to_signer(),
        Mints::WSOL.to_readonly(),
        fee_account.to_writable(),
        ata_sol_token(&signer, &Mints::WSOL).to_writable(),
        TokenProgram::SPL_TOKEN.to_program(),
        system_program::ID.to_readonly(),
        spl_associated_token_account::ID.to_readonly(),
    ];

    if use_flashloan {
        accounts.extend([
            MevBot::FLASHLOAN_ACCOUNT.to_readonly(),
            derive_vault_token_account_mev_bot(
                &MevBot::EMV_BOT_PROGRAM,
                &Mints::WSOL, // default to wsol mint base for flashloan
            )
            .0
            .to_writable(),
        ]);
    }

    accounts.extend([
        minor_mint.to_readonly(),
        token_program.to_program(),
        ata(signer, minor_mint, token_program).to_writable(),
    ]);

    // let the_other_mint_account = ata(&signer(), )
    for pool in pools {
        match pool {
            AnyPoolConfig::MeteoraDlmm(c) => {
                let built = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
                    signer, &c.pool, &c.data,
                )?;
                accounts.extend(vec![
                    built.program,
                    c.data.pair().desired_mint()?.to_readonly(),
                    built.event_authority,
                    built.lb_pair,
                    built.reverse_x,
                    built.reverse_y,
                    built.oracle,
                ]);
                accounts.extend(built.bin_arrays);
            }
            AnyPoolConfig::MeteoraDammV2(c) => {
                let built = MeteoraDammV2InputAccount::build_accounts_no_matter_direction_size(
                    signer, &c.pool, &c.data,
                )?;
                accounts.extend(vec![
                    built.meteora_program,
                    c.data.pair().desired_mint()?.to_readonly(),
                    built.event_authority,
                    built.pool_authority,
                    c.pool.to_writable(),
                    built.token_a_vault,
                    built.token_b_vault,
                ]);
            }
            AnyPoolConfig::PumpAmm(c) => {
                let built = PumpAmmInputAccounts::build_accounts_no_matter_direction_size(
                    signer, &c.pool, &c.data,
                )?;
                let v = vec![
                    built.program,
                    //
                    c.data.pair().desired_mint()?.to_readonly(),
                    built.global_config,
                    built.event_authority,
                    built.protocol_fee_recipient,
                    built.pool,
                    built.pool_base_token_account,
                    built.pool_quote_token_account,
                    built.protocol_fee_recipient_token_account,
                    built.coin_creator_vault_ata,
                    built.coin_creator_vault_authority,
                    built.global_volume_accumulator.unwrap(),
                    built.user_volume_accumulator.unwrap(),
                ];
            }
            AnyPoolConfig::Unsupported => return Err(anyhow!("Unsupported pool type")),
        };
    }

    // Create instruction data
    let mut data = vec![28u8];

    // When true, the bot will not fail the transaction even when it can't find a profitable arbitrage. It will just do nothing and succeed.
    let no_failure_mode = never_abort;

    data.extend_from_slice(&minimum_profit.to_le_bytes());
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    data.extend_from_slice(if no_failure_mode { &[1] } else { &[0] });
    data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    data.extend_from_slice(if use_flashloan { &[1] } else { &[0] });

    Ok(Instruction {
        program_id: MevBot::EMV_BOT_PROGRAM,
        accounts,
        data,
    })
}

pub fn derive_vault_token_account_mev_bot(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault_token_account", mint.as_ref()], program_id)
}

fn fee_collector(use_flashloan: bool) -> Pubkey {
    if use_flashloan {
        MevBot::FLASHLOAN_FEE_ACCOUNT
    } else {
        let fee_accounts = [
            MevBot::NON_FLASHLOAN_ACCOUNT_1,
            MevBot::NON_FLASHLOAN_ACCOUNT_2,
            MevBot::NON_FLASHLOAN_ACCOUNT_3,
        ];
        *random_select(&fee_accounts).expect("fee_accounts should not be empty")
    }
}

fn gas_instructions(compute_limit: u32, unit_price: u64) -> (Vec<Instruction>, u32) {
    let seed = rand::random::<u32>() % 1000;
    let compute_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(compute_limit + seed);
    // 1 lamport = 1_000_000
    let unit_price_ix = ComputeBudgetInstruction::set_compute_unit_price(unit_price);

    (vec![compute_limit_ix, unit_price_ix], compute_limit + seed)
}

fn ensure_token_account_exists(
    belong_to: &Pubkey,
    mint: &Pubkey,
    mint_program: &Pubkey,
) -> Instruction {
    create_associated_token_account_idempotent(belong_to, belong_to, mint, &mint_program)
}

pub async fn simulate_mev_tx(tx: &VersionedTransaction, trace: &Trace) -> Result<SimulationResult> {
    trace.step(StepType::MevSimulationTxRpcCall);

    // Use the simpler simulate_transaction for better performance
    // Note: This won't return metadata for failed simulations
    let response = rpc_client().simulate_transaction(tx).await?;
    let result = SimulationResult::from(&response.value);
    trace.step(StepType::MevSimulationTxRpcReturned);

    Ok(result)
}

pub async fn real_mev_tx(tx: &VersionedTransaction, trace: &Trace) -> Result<Signature> {
    trace.step(StepType::MevRealTxRpcCall);
    let response = rpc_client().send_transaction(tx).await?;
    trace.step(StepType::MevRealTxRpcReturned);
    info!(
        "MEV transaction sent successfully: {}",
        response.to_string()
    );
    Ok(response)
}

pub async fn log_mev_simulation(
    result: &SimulationResult,
    trace: &Trace,
    owner: &Pubkey,
    tx: &VersionedTransaction,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    pools: &[AnyPoolConfig],
) -> Result<()> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();

    let simulation_status = if result.err.is_some() {
        "failed"
    } else {
        "success"
    };

    let error_message = result.err.clone();
    let logs = Some(result.logs.clone());
    let compute_units_consumed = result.units_consumed.map(|u| u as i64);

    let return_data = if let Some(ref meta) = result.meta {
        // Extract return data from meta if available
        None // TransactionMeta doesn't have return_data field based on the struct
    } else {
        None
    };

    let pool_addresses: Vec<String> = pools
        .iter()
        .map(|p| match p {
            AnyPoolConfig::MeteoraDlmm(c) => c.pool.to_string(),
            AnyPoolConfig::MeteoraDammV2(c) => c.pool.to_string(),
            AnyPoolConfig::PumpAmm(c) => c.pool.to_string(),
            AnyPoolConfig::Unsupported => "unsupported".to_string(),
        })
        .collect();

    let pool_types: Vec<String> = pools.iter().map(|p| p.dex_type().to_string()).collect();

    let minor_mint_record = MintRecordRepository::get_mint_from_cache(minor_mint)
        .await?
        .ok_or_else(|| anyhow!("Minor mint not found"))?;
    let desired_mint_record = MintRecordRepository::get_mint_from_cache(desired_mint)
        .await?
        .ok_or_else(|| anyhow!("Desired mint not found"))?;

    // Calculate actual profit from simulation results
    // Find the user's token account for the desired mint
    let user_ata = ata(owner, desired_mint, &spl_token::ID);
    let user_ata_str = user_ata.to_string();

    // When simulation fails or doesn't return metadata, we can't calculate actual profit
    // Mark both profitable and profitability as None to distinguish from actual 0 values
    let (profitable, profitability) = if let Some(ref meta) = result.meta {
        // Find the token balance changes for the user's ATA
        let actual_profit = meta
            .post_token_balances
            .iter()
            .find(|tb| {
                tb.mint == desired_mint.to_string() && tb.owner.as_ref() == Some(&user_ata_str)
            })
            .and_then(|post| {
                meta.pre_token_balances
                    .iter()
                    .find(|pre| {
                        pre.mint == desired_mint.to_string()
                            && pre.account_index == post.account_index
                    })
                    .map(|pre| {
                        // Parse as i128 first to avoid overflow, then check if it fits in i64
                        let post_amount: i128 = post.ui_token_amount.amount.parse().unwrap_or(0);
                        let pre_amount: i128 = pre.ui_token_amount.amount.parse().unwrap_or(0);
                        let profit = post_amount - pre_amount;

                        // Clamp to i64 range if overflow would occur
                        if profit > i64::MAX as i128 {
                            i64::MAX
                        } else if profit < i64::MIN as i128 {
                            i64::MIN
                        } else {
                            profit as i64
                        }
                    })
            })
            .unwrap_or(0);

        // Set profitable based on actual profit value (positive, zero, or negative)
        // Only when simulation succeeded
        let is_profitable = if simulation_status == "success" {
            Some(actual_profit > 0)
        } else {
            // Failed simulations are not profitable
            Some(false)
        };

        // Always record the actual profit/loss value (can be negative)
        (is_profitable, Some(actual_profit))
    } else {
        // No meta available - can't determine profitability
        (None, None)
    };

    // Extract accounts from the transaction
    let accounts: Vec<SimulationAccount> = match &tx.message {
        solana_sdk::message::VersionedMessage::Legacy(msg) => {
            msg.account_keys
                .iter()
                .enumerate()
                .map(|(idx, pubkey)| {
                    let is_signer = idx < msg.header.num_required_signatures as usize;
                    let is_writable = if is_signer {
                        // For signers, writable if index is before readonly signed accounts
                        idx < (msg.header.num_required_signatures
                            - msg.header.num_readonly_signed_accounts)
                            as usize
                    } else {
                        // For non-signers, writable if before the readonly unsigned section
                        let non_signer_idx = idx - msg.header.num_required_signatures as usize;
                        let num_writable_unsigned = msg.account_keys.len()
                            - msg.header.num_required_signatures as usize
                            - msg.header.num_readonly_unsigned_accounts as usize;
                        non_signer_idx < num_writable_unsigned
                    };
                    SimulationAccount {
                        pubkey: *pubkey,
                        is_signer,
                        is_writable,
                    }
                })
                .collect()
        }
        solana_sdk::message::VersionedMessage::V0(msg) => {
            msg.account_keys
                .iter()
                .enumerate()
                .map(|(idx, pubkey)| {
                    let is_signer = idx < msg.header.num_required_signatures as usize;
                    let is_writable = if is_signer {
                        // For signers, writable if index is before readonly signed accounts
                        idx < (msg.header.num_required_signatures
                            - msg.header.num_readonly_signed_accounts)
                            as usize
                    } else {
                        // For non-signers, writable if before the readonly unsigned section
                        let non_signer_idx = idx - msg.header.num_required_signatures as usize;
                        let num_writable_unsigned = msg.account_keys.len()
                            - msg.header.num_required_signatures as usize
                            - msg.header.num_readonly_unsigned_accounts as usize;
                        non_signer_idx < num_writable_unsigned
                    };
                    SimulationAccount {
                        pubkey: *pubkey,
                        is_signer,
                        is_writable,
                    }
                })
                .collect()
        }
    };

    let params = MevSimulationLogParams {
        minor_mint: *minor_mint,
        desired_mint: *desired_mint,
        minor_mint_sym: minor_mint_record.symbol,
        desired_mint_sym: desired_mint_record.symbol,
        pools: pool_addresses,
        pool_types,
        profitable,
        profitability,
        details: MevSimulationLogDetails { accounts },
        tx_size: Some(tx_size as i32),
        simulation_status: Some(simulation_status.to_string()),
        compute_units_consumed,
        error_message,
        logs,
        return_data,
        units_per_byte: None,
        trace: Some(trace.dump_json()),
        reason: generate_reason(result),
    };

    if let Err(e) = MevSimulationLogRepository::insert(params).await {
        error!("Failed to log MEV simulation: {}", e);
    }

    Ok(())
}

pub async fn simulate_and_log_mev(
    owner: Pubkey,
    tx: &VersionedTransaction,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    pools: &[AnyPoolConfig],
    minimum_profit: u64,
    trace: Trace,
) -> Result<(SimulationResult, Trace)> {
    let result = simulate_mev_tx(tx, &trace).await?;

    if let Err(e) =
        log_mev_simulation(&result, &trace, &owner, tx, minor_mint, desired_mint, pools).await
    {
        error!("Failed to log MEV simulation: {}", e);
    }

    Ok((result, trace))
}

fn generate_reason(result: &SimulationResult) -> Option<String> {
    for log in &result.logs {
        let log_lower = log.to_lowercase();

        if log_lower.contains("no profitable") {
            return Some("No profitable path".to_string());
        }

        if log_lower.contains("insufficient funds") {
            return Some("Insufficient funds".to_string());
        }
    }

    None
}
