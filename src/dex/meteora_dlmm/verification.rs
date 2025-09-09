use crate::convention::chain::simulation::SimulationResponse;
use crate::convention::chain::util::alt::get_alt_by_key;
use crate::dex::interface::PoolConfig;
use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::meteora_dlmm::config::MeteoraDlmmConfig;
use crate::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
use crate::dex::meteora_dlmm::misc::input_data::MeteoraDlmmIxData;
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
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;
use spl_token::state::Account as TokenAccount;

async fn build_tx(
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
) -> AResult<SwapSimulationResult> {
    let config = MeteoraDlmmConfig::from_address(pool_address).await?;

    // For simulation, we always use X->Y direction (SOL->USDC)
    // The accounts are built assuming this direction
    let accounts = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
        payer,
        pool_address,
        &config.pool_data,
    )
    .await?
    .to_list_cloned();

    let tx = build_tx(*payer, accounts.clone(), amount_in, min_amount_out).await?;

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

#[cfg(test)]
mod tests {
    use crate::convention::chain::simulation::SimulationHelper;
    use crate::database::mint_record::repository::MintRecordRepository;
    use crate::dex::interface::PoolConfig;
    use crate::dex::legacy_interface::InputAccountUtil;
    use crate::dex::meteora_dlmm::config::MeteoraDlmmConfig;
    use crate::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
    use crate::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
    use crate::dex::meteora_dlmm::verification::simulate_swap_and_get_balance_diff;
    use crate::global::client::db::must_init_db;
    use crate::sdk::solana_rpc::rpc::_set_test_client;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::pubkey::ToPubkey;
    use solana_program::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use std::time::Duration;
    use tokio::time::sleep;

    static POOL: Pubkey = pubkey!("5rCf1DM8LjKTw4YqhnoLcngyZYeNnQqztScTogYHAS6");

    #[tokio::test]
    async fn verify_meteora_dlmm() {
        must_init_db().await;
        // _set_test_client();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn verify_get_amount_out_matches_simulation() -> AResult<()> {
        _set_test_client();
        must_init_db().await;

        // Give services time to initialize
        sleep(Duration::from_millis(100)).await;

        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let amount_in = 10000000000; // 10 tokens with 9 decimals
        let min_amount_out = 0;

        // Get pool config for token info
        let config = MeteoraDlmmConfig::from_address(&POOL).await?;
        let token_x_mint = config.pool_data.token_x_mint;
        let token_y_mint = config.pool_data.token_y_mint;

        // Fetch token symbols and decimals from database
        let token_x_record = MintRecordRepository::get_mint(&token_x_mint).await?;
        let token_x_symbol = token_x_record
            .as_ref()
            .map(|m| m.symbol.clone())
            .unwrap_or_else(|| token_x_mint.to_string()[..6].to_string());
        let token_x_decimals = token_x_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(9u32);

        let token_y_record = MintRecordRepository::get_mint(&token_y_mint).await?;
        let token_y_symbol = token_y_record
            .as_ref()
            .map(|m| m.symbol.clone())
            .unwrap_or_else(|| token_y_mint.to_string()[..6].to_string());
        let token_y_decimals = token_y_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(6u32);

        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", token_x_symbol, token_y_symbol);
        println!(
            "Amount in: {}",
            SimulationHelper::format_amount(amount_in, token_x_decimals)
        );

        // Calculate expected output using get_amount_out
        let expected_out = config.get_amount_out(amount_in, &token_x_mint, &token_y_mint).await?;
        println!(
            "\nExpected output (get_amount_out): {} ({})",
            SimulationHelper::format_amount(expected_out, token_y_decimals),
            expected_out
        );

        // Simulate actual swap to get real output
        let result =
            simulate_swap_and_get_balance_diff(&POOL, &payer, amount_in, min_amount_out).await?;

        println!("\nUnits consumed: {:?}", result.compute_units);

        if let Some(err) = &result.error {
            println!("Simulation error: {}", err);
            assert!(false, "Simulation failed: {}", err);
        }
        
        println!("Simulation successful!");
        println!("\nBalance changes from simulation:");

        println!(
            "  {} (in):  {}{}",
            token_x_symbol,
            if result.balance_diff_in < 0 { "-" } else { "" },
            SimulationHelper::format_amount(
                result.balance_diff_in.unsigned_abs() as u64,
                token_x_decimals
            )
        );

        let actual_out = result.balance_diff_out as u64;
        println!(
            "  {} (out): +{} ({})",
            token_y_symbol,
            SimulationHelper::format_amount(actual_out, token_y_decimals),
            actual_out
        );

        // Verify get_amount_out matches simulation
        println!("\nVerification:");
        println!("  Expected (get_amount_out): {}", expected_out);
        println!("  Actual (simulation):       {}", actual_out);
        
        let tolerance = 0.01; // 1% tolerance for fees/slippage
        let diff_percent = if expected_out > 0 {
            ((expected_out as f64 - actual_out as f64).abs() / expected_out as f64) * 100.0
        } else {
            0.0
        };
        
        println!("  Difference: {:.4}%", diff_percent);
        
        if diff_percent <= tolerance * 100.0 {
            println!("  ✓ get_amount_out matches simulation within {:.1}% tolerance", tolerance * 100.0);
        } else {
            println!("  ✗ get_amount_out differs from simulation by more than {:.1}%", tolerance * 100.0);
            if expected_out > actual_out {
                let diff = expected_out - actual_out;
                println!(
                    "    get_amount_out overestimated by {} ({})",
                    SimulationHelper::format_amount(diff, token_y_decimals),
                    diff
                );
            } else {
                let diff = actual_out - expected_out;
                println!(
                    "    get_amount_out underestimated by {} ({})",
                    SimulationHelper::format_amount(diff, token_y_decimals),
                    diff
                );
            }
        }
        
        // Assert within tolerance
        assert!(
            diff_percent <= tolerance * 100.0,
            "get_amount_out differs from simulation by {:.4}% (exceeds {:.1}% tolerance)",
            diff_percent,
            tolerance * 100.0
        );
        
        unit_ok!()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn verify_usdc_to_sol_calculation() -> AResult<()> {
        _set_test_client();
        must_init_db().await;

        // Give services time to initialize
        sleep(Duration::from_millis(100)).await;

        let usdc_amount = 1000_000_000; // 1000 USDC (6 decimals)

        // Get pool config for token info
        let config = MeteoraDlmmConfig::from_address(&POOL).await?;
        let token_x_mint = config.pool_data.token_x_mint; // SOL
        let token_y_mint = config.pool_data.token_y_mint; // USDC
        
        // Fetch token symbols and decimals from database
        let token_x_record = MintRecordRepository::get_mint(&token_x_mint).await?;
        let token_x_symbol = token_x_record
            .as_ref()
            .map(|m| m.symbol.clone())
            .unwrap_or_else(|| token_x_mint.to_string()[..6].to_string());
        let token_x_decimals = token_x_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(9u32);

        let token_y_record = MintRecordRepository::get_mint(&token_y_mint).await?;
        let token_y_symbol = token_y_record
            .as_ref()
            .map(|m| m.symbol.clone())
            .unwrap_or_else(|| token_y_mint.to_string()[..6].to_string());
        let token_y_decimals = token_y_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(6u32);

        println!("Pool: {}", POOL);
        println!("\n=== Testing USDC -> SOL swap ===");
        println!(
            "Amount in: {} USDC",
            SimulationHelper::format_amount(usdc_amount, token_y_decimals)
        );

        // Calculate expected output using get_amount_out (USDC -> SOL)
        let sol_out = config.get_amount_out(usdc_amount, &token_y_mint, &token_x_mint).await?;
        println!(
            "Output (get_amount_out): {} SOL ({})",
            SimulationHelper::format_amount(sol_out, token_x_decimals),
            sol_out
        );
        
        // Now test the reverse: use that SOL amount to buy back USDC
        println!("\n=== Testing reverse: SOL -> USDC swap ===");
        println!(
            "Amount in: {} SOL", 
            SimulationHelper::format_amount(sol_out, token_x_decimals)
        );
        
        let usdc_back = config.get_amount_out(sol_out, &token_x_mint, &token_y_mint).await?;
        println!(
            "Output (get_amount_out): {} USDC ({})",
            SimulationHelper::format_amount(usdc_back, token_y_decimals),
            usdc_back
        );
        
        // Check round-trip efficiency
        let round_trip_loss = if usdc_amount > usdc_back {
            usdc_amount - usdc_back
        } else {
            0
        };
        let loss_percent = (round_trip_loss as f64 / usdc_amount as f64) * 100.0;
        
        println!("\n=== Round-trip analysis ===");
        println!("Started with: {} USDC", SimulationHelper::format_amount(usdc_amount, token_y_decimals));
        println!("Ended with:   {} USDC", SimulationHelper::format_amount(usdc_back, token_y_decimals));
        println!("Loss:         {} USDC ({:.2}%)", 
                SimulationHelper::format_amount(round_trip_loss, token_y_decimals), 
                loss_percent);
        
        // Verify the SOL amount is reasonable (should be around 4-5 SOL for 1000 USDC at current prices)
        let sol_amount_float = sol_out as f64 / 10_f64.powi(token_x_decimals as i32);
        let reasonable_range = 2.0..=10.0; // Allow range from 2 to 10 SOL for 1000 USDC
        
        assert!(
            reasonable_range.contains(&sol_amount_float),
            "SOL output {} is outside reasonable range {:?} for 1000 USDC",
            sol_amount_float,
            reasonable_range
        );
        
        println!("\n✓ USDC->SOL calculation produces reasonable output: {:.6} SOL for 1000 USDC", sol_amount_float);
        
        // Round-trip loss should be minimal (mainly fees)
        assert!(
            loss_percent < 1.0, // Less than 1% loss on round-trip
            "Round-trip loss {:.2}% exceeds 1% threshold",
            loss_percent
        );
        
        println!("✓ Round-trip loss is acceptable: {:.2}%", loss_percent);
        
        unit_ok!()
    }

    #[tokio::test]
    async fn test_simulate_swap() -> AResult<()> {
        _set_test_client();
        must_init_db().await;

        let pool = POOL;
        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let amount_in = 10000000000; // 10 tokens with 9 decimals
        let min_amount_out = 0;

        let result =
            simulate_swap_and_get_balance_diff(&pool, &payer, amount_in, min_amount_out).await?;

        if let Some(error) = result.error {
            println!("Simulation failed: {}", error);
        } else {
            println!("Simulation successful!");
            println!("Balance diff in: {}", result.balance_diff_in);
            println!("Balance diff out: {}", result.balance_diff_out);
            println!("Compute units: {:?}", result.compute_units);
        }

        unit_ok!()
    }

    #[tokio::test]
    async fn generate_solana_validator_cmd() -> AResult<()> {
        must_init_db().await;

        let config = MeteoraDlmmConfig::from_address(&POOL).await?;
        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let accounts = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
            &payer,
            &POOL,
            &config.pool_data,
        )
        .await?
        .to_list_cloned();

        // Build the validator command from the accounts array
        let bootstrap_cmd = format!(
            "solana-test-validator --reset \\
  --url https://api.mainnet-beta.solana.com \\
  {}",
            accounts
                .iter()
                .map(|account| format!("--clone {}", account.pubkey))
                .collect::<Vec<_>>()
                .join(" \\\n  ")
        );

        println!("\n{}\n", bootstrap_cmd);

        unit_ok!()
    }
}
