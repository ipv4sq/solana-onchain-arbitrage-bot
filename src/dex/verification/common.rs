use crate::convention::chain::simulation::SimulationResponse;
use crate::convention::chain::util::alt::get_alt_by_key;
use crate::dex::interface::PoolConfig;
use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::meteora_dlmm::config::MeteoraDlmmConfig;
use crate::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
use crate::dex::meteora_damm_v2::config::MeteoraDammV2Config;
use crate::dex::meteora_damm_v2::misc::input_account::MeteoraDammV2InputAccount;
use crate::dex::meteora_dlmm::misc::input_data::MeteoraDlmmIxData;
use crate::dex::pump_amm::config::PumpAmmConfig;
use crate::dex::pump_amm::misc::input_account::PumpAmmInputAccounts;
use crate::global::constant::pool_program::PoolProgram;
use crate::pipeline::uploader::mev_bot::construct::gas_instructions;
use crate::sdk::solana_rpc::rpc::rpc_client;
use crate::util::alias::AResult;
use crate::util::traits::pubkey::ToPubkey;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{
    RpcSimulateTransactionAccountsConfig, RpcSimulateTransactionConfig,
};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::message::v0::Message;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;
use spl_token::state::Account as TokenAccount;

async fn build_test_swap_tx(
    signer: Pubkey,
    accounts: Vec<AccountMeta>,
    amount_in: u64,
    min_amount_out: u64,
) -> AResult<VersionedTransaction> {
    let (mut instructions, _limit) = gas_instructions(100_000, 0);
    let data = MeteoraDlmmIxData {
        amount_in,
        min_amount_out,
    };
    let swap_ix = Instruction {
        program_id: PoolProgram::METEORA_DLMM,
        accounts: accounts.clone(),
        data: hex::decode(data.to_hex())?,
    };
    instructions.push(swap_ix);
    let alt_keys = vec![
        // this seems to be legit
        "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
    ];

    let mut alts = Vec::new();
    for key in &alt_keys {
        alts.push(get_alt_by_key(key).await?);
    }
    let blockhash = rpc_client().get_latest_blockhash().await?;

    let message = Message::try_compile(&signer, &instructions, &alts, blockhash)?;

    let tx = VersionedTransaction {
        signatures: vec![Signature::default(); 1],
        message: solana_sdk::message::VersionedMessage::V0(message),
    };
    Ok(tx)
}

#[derive(Debug, Clone)]
pub struct SwapSimulationResult {
    pub balance_diff_in: i128,
    pub balance_diff_out: i128,
    pub compute_units: Option<u64>,
    pub error: Option<String>,
}

pub async fn simulate_swap_and_get_balance_diff(
    pool_address: &Pubkey,
    payer: &Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    swap_x_to_y: bool,
) -> AResult<SwapSimulationResult> {
    let config = MeteoraDlmmConfig::from_address(pool_address).await?;

    // Build accounts based on swap direction
    let accounts = if swap_x_to_y {
        MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
            payer,
            pool_address,
            &config.pool_data,
        )
        .await?
        .to_list_cloned()
    } else {
        // For Y->X swap, we need to swap the user token accounts
        let mut accs = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
            payer,
            pool_address,
            &config.pool_data,
        )
        .await?
        .to_list_cloned();
        // Swap user token accounts (indices 4 and 5)
        accs.swap(4, 5);
        accs
    };

    let tx = build_test_swap_tx(*payer, accounts.clone(), amount_in, min_amount_out).await?;

    let user_token_in = accounts[4].pubkey;
    let user_token_out = accounts[5].pubkey;

    // Get pre-simulation balances
    let pre_token_in = rpc_client().get_account(&user_token_in).await?;
    let pre_token_out = rpc_client().get_account(&user_token_out).await?;

    let pre_balance_in = if pre_token_in.lamports > 0 {
        TokenAccount::unpack(&pre_token_in.data)?.amount
    } else {
        0
    };

    let pre_balance_out = if pre_token_out.lamports > 0 {
        TokenAccount::unpack(&pre_token_out.data)?.amount
    } else {
        0
    };

    // Simulate the transaction
    let rpc_response = rpc_client()
        .simulate_transaction_with_config(
            &tx,
            RpcSimulateTransactionConfig {
                sig_verify: false,
                replace_recent_blockhash: true,
                commitment: None,
                encoding: Some(UiTransactionEncoding::Base64),
                accounts: Some(RpcSimulateTransactionAccountsConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    addresses: vec![user_token_in.to_string(), user_token_out.to_string()],
                }),
                min_context_slot: None,
                inner_instructions: true,
            },
        )
        .await?;

    let sim_response =
        SimulationResponse::from_rpc_response(rpc_response, &[user_token_in, user_token_out])?;

    if let Some(err) = &sim_response.error {
        return Ok(SwapSimulationResult {
            balance_diff_in: 0,
            balance_diff_out: 0,
            compute_units: sim_response.compute_units,
            error: Some(err.clone()),
        });
    }

    // Get post-simulation balances
    let post_balance_in = sim_response
        .get_account(&user_token_in)
        .and_then(|acc| acc.get_token_balance().ok().flatten())
        .unwrap_or(0);

    let post_balance_out = sim_response
        .get_account(&user_token_out)
        .and_then(|acc| acc.get_token_balance().ok().flatten())
        .unwrap_or(0);

    let balance_diff_in = post_balance_in as i128 - pre_balance_in as i128;
    let balance_diff_out = post_balance_out as i128 - pre_balance_out as i128;

    Ok(SwapSimulationResult {
        balance_diff_in,
        balance_diff_out,
        compute_units: sim_response.compute_units,
        error: None,
    })
}

pub async fn simulate_damm_v2_swap_and_get_balance_diff(
    pool_address: &Pubkey,
    payer: &Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    swap_a_to_b: bool,
) -> AResult<SwapSimulationResult> {
    let config = MeteoraDammV2Config::from_address(pool_address).await?;

    // Build accounts based on swap direction
    let (input_mint, output_mint) = if swap_a_to_b {
        (config.pool_data.token_a_mint, config.pool_data.token_b_mint)
    } else {
        (config.pool_data.token_b_mint, config.pool_data.token_a_mint)
    };

    let accounts = MeteoraDammV2InputAccount::build_accounts_with_direction_and_size(
        payer,
        pool_address,
        &config.pool_data,
        &input_mint,
        &output_mint,
        Some(amount_in),
        None,
    )
    .await?
    .to_list_cloned();

    // Build the swap instruction data for DAMM V2
    let discriminator = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8];
    let mut data = discriminator.to_vec();
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&min_amount_out.to_le_bytes());

    // Build the swap instruction
    let (mut instructions, _limit) = gas_instructions(100_000, 0);
    let swap_ix = Instruction {
        program_id: PoolProgram::METEORA_DAMM_V2,
        accounts: accounts.clone(),
        data,
    };
    instructions.push(swap_ix);

    // Use the same ALT as DLMM for now
    let alt_keys = vec![
        "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
    ];

    let mut alts = Vec::new();
    for key in &alt_keys {
        alts.push(get_alt_by_key(key).await?);
    }
    let blockhash = rpc_client().get_latest_blockhash().await?;

    let message = Message::try_compile(payer, &instructions, &alts, blockhash)?;

    let tx = VersionedTransaction {
        signatures: vec![Signature::default(); 1],
        message: solana_sdk::message::VersionedMessage::V0(message),
    };

    // User token accounts are at indices 2 and 3 for DAMM V2
    let user_token_in = accounts[2].pubkey;
    let user_token_out = accounts[3].pubkey;

    // Get pre-simulation balances
    let pre_token_in = rpc_client().get_account(&user_token_in).await?;
    let pre_token_out = rpc_client().get_account(&user_token_out).await?;

    let pre_balance_in = if pre_token_in.lamports > 0 {
        TokenAccount::unpack(&pre_token_in.data)?.amount
    } else {
        0
    };

    let pre_balance_out = if pre_token_out.lamports > 0 {
        TokenAccount::unpack(&pre_token_out.data)?.amount
    } else {
        0
    };

    // Simulate the transaction
    let rpc_response = rpc_client()
        .simulate_transaction_with_config(
            &tx,
            RpcSimulateTransactionConfig {
                sig_verify: false,
                replace_recent_blockhash: true,
                commitment: None,
                encoding: Some(UiTransactionEncoding::Base64),
                accounts: Some(RpcSimulateTransactionAccountsConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    addresses: vec![user_token_in.to_string(), user_token_out.to_string()],
                }),
                min_context_slot: None,
                inner_instructions: true,
            },
        )
        .await?;

    let sim_response =
        SimulationResponse::from_rpc_response(rpc_response, &[user_token_in, user_token_out])?;

    if let Some(err) = &sim_response.error {
        return Ok(SwapSimulationResult {
            balance_diff_in: 0,
            balance_diff_out: 0,
            compute_units: sim_response.compute_units,
            error: Some(err.clone()),
        });
    }

    // Get post-simulation balances
    let post_balance_in = sim_response
        .get_account(&user_token_in)
        .and_then(|acc| acc.get_token_balance().ok().flatten())
        .unwrap_or(0);

    let post_balance_out = sim_response
        .get_account(&user_token_out)
        .and_then(|acc| acc.get_token_balance().ok().flatten())
        .unwrap_or(0);

    let balance_diff_in = post_balance_in as i128 - pre_balance_in as i128;
    let balance_diff_out = post_balance_out as i128 - pre_balance_out as i128;

    Ok(SwapSimulationResult {
        balance_diff_in,
        balance_diff_out,
        compute_units: sim_response.compute_units,
        error: None,
    })
}

pub async fn simulate_pump_amm_swap_and_get_balance_diff(
    pool_address: &Pubkey,
    payer: &Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    swap_base_to_quote: bool,
) -> AResult<SwapSimulationResult> {
    let config = PumpAmmConfig::from_address(pool_address).await?;

    // Build accounts (Pump AMM always has base mint and quote mint)
    let accounts = PumpAmmInputAccounts::build_accounts_no_matter_direction_size(
        payer,
        pool_address,
        &config.pool_data,
    )
    .await?
    .to_list_cloned();

    // Build the swap instruction data for Pump AMM
    // Pump AMM uses different discriminators for buy and sell
    // The discriminators are swapped compared to what you might expect:
    // - "Buy" discriminator [0x33...] is for selling base tokens (base->quote)
    // - "Sell" discriminator [0x66...] is for buying base tokens (quote->base)
    let data = if swap_base_to_quote {
        // Sell base tokens for quote tokens (base -> quote)
        // Uses the "buy" discriminator which handles base_amount_in
        let discriminator = [0x33, 0xe6, 0x85, 0xa4, 0x01, 0x7f, 0x83, 0xad];
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&amount_in.to_le_bytes()); // base_amount_in
        data.extend_from_slice(&min_amount_out.to_le_bytes()); // min_quote_amount_out
        data
    } else {
        // Buy base tokens with quote tokens (quote -> base) 
        // Uses the "sell" discriminator for exact out
        let discriminator = [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea];
        let mut data = discriminator.to_vec();
        // For exact out, we specify the output amount first, then max input
        data.extend_from_slice(&min_amount_out.to_le_bytes()); // base_amount_out (what we want)
        data.extend_from_slice(&amount_in.to_le_bytes()); // max_quote_amount_in
        data
    };

    // Build the swap instruction
    let (mut instructions, _limit) = gas_instructions(100_000, 0);
    let swap_ix = Instruction {
        program_id: PoolProgram::PUMP_AMM,
        accounts: accounts.clone(),
        data,
    };
    instructions.push(swap_ix);

    // Use the same ALT as other DEXs for now
    let alt_keys = vec![
        "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
    ];

    let mut alts = Vec::new();
    for key in &alt_keys {
        alts.push(get_alt_by_key(key).await?);
    }
    let blockhash = rpc_client().get_latest_blockhash().await?;

    let message = Message::try_compile(payer, &instructions, &alts, blockhash)?;

    let tx = VersionedTransaction {
        signatures: vec![Signature::default(); 1],
        message: solana_sdk::message::VersionedMessage::V0(message),
    };

    // User token accounts are at indices 5 and 6 for Pump AMM
    let (user_token_in, user_token_out) = if swap_base_to_quote {
        (accounts[5].pubkey, accounts[6].pubkey) // base in, quote out
    } else {
        (accounts[6].pubkey, accounts[5].pubkey) // quote in, base out
    };

    // Get pre-simulation balances
    let pre_token_in = rpc_client().get_account(&user_token_in).await?;
    let pre_token_out = rpc_client().get_account(&user_token_out).await?;

    let pre_balance_in = if pre_token_in.lamports > 0 {
        TokenAccount::unpack(&pre_token_in.data)?.amount
    } else {
        0
    };

    let pre_balance_out = if pre_token_out.lamports > 0 {
        TokenAccount::unpack(&pre_token_out.data)?.amount
    } else {
        0
    };

    // Simulate the transaction
    let rpc_response = rpc_client()
        .simulate_transaction_with_config(
            &tx,
            RpcSimulateTransactionConfig {
                sig_verify: false,
                replace_recent_blockhash: true,
                commitment: None,
                encoding: Some(UiTransactionEncoding::Base64),
                accounts: Some(RpcSimulateTransactionAccountsConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    addresses: vec![user_token_in.to_string(), user_token_out.to_string()],
                }),
                min_context_slot: None,
                inner_instructions: true,
            },
        )
        .await?;

    let sim_response =
        SimulationResponse::from_rpc_response(rpc_response, &[user_token_in, user_token_out])?;

    if let Some(err) = &sim_response.error {
        return Ok(SwapSimulationResult {
            balance_diff_in: 0,
            balance_diff_out: 0,
            compute_units: sim_response.compute_units,
            error: Some(err.clone()),
        });
    }

    // Get post-simulation balances
    let post_balance_in = sim_response
        .get_account(&user_token_in)
        .and_then(|acc| acc.get_token_balance().ok().flatten())
        .unwrap_or(0);

    let post_balance_out = sim_response
        .get_account(&user_token_out)
        .and_then(|acc| acc.get_token_balance().ok().flatten())
        .unwrap_or(0);

    let balance_diff_in = post_balance_in as i128 - pre_balance_in as i128;
    let balance_diff_out = post_balance_out as i128 - pre_balance_out as i128;

    Ok(SwapSimulationResult {
        balance_diff_in,
        balance_diff_out,
        compute_units: sim_response.compute_units,
        error: None,
    })
}
