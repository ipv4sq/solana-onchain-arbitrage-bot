use crate::convention::chain::util::simulation::SimulationResult;
use crate::database::mev_simulation_log::model::{
    MevSimulationLogDetails, MevSimulationLogParams, SimulationAccount,
};
use crate::database::mev_simulation_log::repository::MevSimulationLogRepository;
use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::constant::mev_bot::MevBot;
use crate::global::constant::mint::Mints;
use crate::global::constant::token_program::{SystemProgram, TokenProgram};
use crate::global::enums::step_type::StepType;
use crate::global::trace::types::Trace;
use crate::pipeline::uploader::common::simulation_log;
use crate::pipeline::uploader::provider::jito::{
    get_jito_tips, get_random_tip_account, send_bundle,
};
use crate::return_error;
use crate::sdk::rpc::methods::simulation;
use crate::util::alias::{MintAddress, TokenProgramAddress};
use crate::util::random::random_select;
use crate::util::solana::pda::{ata, ata_sol_token};
use crate::util::traits::account_meta::ToAccountMeta;
use anyhow::Result;
use simulation::simulate_transaction_with_config;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash::Hash;
use solana_sdk::message::v0::Message;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction::transfer;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use tracing::{error, info};

const HELIUS_TIP_ACCOUNTS: &[&str] = &[
    "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE",
    "D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ",
    "9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta",
    "5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn",
    "2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD",
    "2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ",
    "wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF",
    "3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT",
    "4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey",
    "4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or",
];
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
    let (mut instructions, _limit) = gas_instructions(compute_unit_limit, unit_price);

    let wallet_pub = wallet.pubkey();
    let mint_token_program = MintRecordRepository::get_mint_or_err(minor_mint)
        .await?
        .program
        .0;
    let jito_tip_account = get_random_tip_account();
    let p75_jito_tip = get_jito_tips()
        .map(|t| t.landed_tips_75th_percentile)
        .unwrap_or(0.00001);

    let jito_tip_ix = transfer(
        &wallet_pub,
        &jito_tip_account,
        (p75_jito_tip * LAMPORTS_PER_SOL as f64) as u64,
    );

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
    )
    .await?;
    instructions.push(swap_ix);
    instructions.push(jito_tip_ix);

    let message = Message::try_compile(&wallet.pubkey(), &instructions, alts, blockhash)?;
    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[wallet],
    )?;
    Ok(tx)
}

pub async fn create_invoke_mev_instruction(
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
        SystemProgram.to_readonly(),
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

    for pool in pools {
        let specific_accounts = pool.build_mev_bot_ix_accounts(signer).await?;
        accounts.extend(specific_accounts);
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

pub fn gas_instructions(compute_limit: u32, unit_price: u64) -> (Vec<Instruction>, u32) {
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
    if trace.since_begin() > 300 {
        info!(
            "Gave up on simulation tx because it takes {} milliseconds from trigger to now",
            trace.since_begin()
        );
        return_error!("Gave up");
    }
    trace.step(StepType::MevSimulationTxRpcCall);

    // Use the simpler simulate_transaction for better performance
    // Note: This won't return metadata for failed simulations
    let response = simulate_transaction_with_config(
        tx,
        RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: false,
            commitment: Some(CommitmentConfig::processed()),
            encoding: Some(UiTransactionEncoding::Base64),
            accounts: None,
            min_context_slot: Some(trace.slot),
            inner_instructions: false,
        },
    )
    .await?;
    let result = SimulationResult::from(&response.value);
    trace.step(StepType::MevSimulationTxRpcReturned);

    Ok(result)
}

pub async fn real_mev_tx(tx: &VersionedTransaction, trace: &Trace) -> Result<String> {
    if trace.since_begin() > 400 {
        info!(
            "Gave up on landing tx because it takes {} milliseconds from trigger to now",
            trace.since_begin()
        );
        return_error!("Gave up");
    }
    trace.step(StepType::MevRealTxRpcCall);
    // let response = rpc_client().send_transaction(tx).await?;
    // sender(tx).await;
    let response = send_bundle(tx).await;
    match response {
        Ok(bundle_id) => {
            trace.step_with(
                StepType::MevRealTxRpcReturned,
                "jito_bundle_id",
                bundle_id.clone(),
            );
            info!("MEV transaction sent successfully: jito id: {}", bundle_id);
            Ok(bundle_id)
        }
        Err(e) => {
            trace.step_with(StepType::MevRealTxRpcReturned, "error", e.to_string());
            error!("Failed to send MEV transaction: {}", e);
            Err(e)
        }
    }
}

pub async fn simulate_and_log_mev(
    owner: Pubkey,
    tx: &VersionedTransaction,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    pools: &[AnyPoolConfig],
    _minimum_profit: u64,
    trace: Trace,
) -> Result<(SimulationResult, Trace)> {
    let result = simulate_mev_tx(tx, &trace).await?;

    if let Err(e) = simulation_log::log_mev_simulation(
        &result,
        &trace,
        &owner,
        tx,
        minor_mint,
        desired_mint,
        pools,
    )
    .await
    {
        error!("Failed to log MEV simulation: {}", e);
    }

    Ok((result, trace))
}
