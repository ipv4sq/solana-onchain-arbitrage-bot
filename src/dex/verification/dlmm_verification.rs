#[cfg(test)]
mod tests {
    use crate::convention::chain::simulation::SimulationHelper;
    use crate::database::mint_record::repository::MintRecordRepository;
    use crate::dex::interface::PoolConfig;
    use crate::dex::legacy_interface::InputAccountUtil;
    use crate::dex::meteora_dlmm::config::MeteoraDlmmConfig;
    use crate::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
    use crate::dex::verification::common::simulate_swap_and_get_balance_diff;
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
    async fn verify_x_to_y_matches_simulation() -> AResult<()> {
        _set_test_client();
        must_init_db().await;

        // Give services time to initialize
        sleep(Duration::from_millis(100)).await;

        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let amount_in = 10000000000; // 10 SOL
        let min_amount_out = 0;

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

        println!("=== Testing X->Y (SOL->USDC) Direction ===");
        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", token_x_symbol, token_y_symbol);
        println!(
            "Amount in: {} {}",
            SimulationHelper::format_amount(amount_in, token_x_decimals),
            token_x_symbol
        );

        // Calculate expected output using get_amount_out
        let expected_out = config
            .get_amount_out(amount_in, &token_x_mint, &token_y_mint)
            .await?;
        println!(
            "\nExpected output (get_amount_out): {} {} (raw: {})",
            SimulationHelper::format_amount(expected_out, token_y_decimals),
            token_y_symbol,
            expected_out
        );

        // Simulate actual swap to get real output (X->Y direction)
        let result = simulate_swap_and_get_balance_diff(
            &POOL,
            &payer,
            amount_in,
            min_amount_out,
            true, // swap_x_to_y = true
        )
        .await?;

        if let Some(err) = &result.error {
            println!("Simulation error: {}", err);
            assert!(false, "Simulation failed: {}", err);
        }

        println!("\nSimulation successful!");
        println!("Compute units consumed: {:?}", result.compute_units);

        let actual_out = result.balance_diff_out as u64;
        println!(
            "Actual output (simulation):      {} {} (raw: {})",
            SimulationHelper::format_amount(actual_out, token_y_decimals),
            token_y_symbol,
            actual_out
        );

        // Verify get_amount_out matches simulation
        let tolerance = 0.001; // 0.1% tolerance
        let diff_percent = if expected_out > 0 {
            ((expected_out as f64 - actual_out as f64).abs() / expected_out as f64) * 100.0
        } else {
            0.0
        };

        println!("\n=== Verification Results ===");
        println!("Expected (get_amount_out): {}", expected_out);
        println!("Actual (simulation):       {}", actual_out);
        println!("Difference:                {:.6}%", diff_percent);

        if diff_percent <= tolerance * 100.0 {
            println!(
                "✓ PASS: get_amount_out matches simulation within {:.1}% tolerance",
                tolerance * 100.0
            );
        } else {
            println!(
                "✗ FAIL: get_amount_out differs from simulation by {:.6}% (exceeds {:.1}% tolerance)",
                diff_percent,
                tolerance * 100.0
            );
            if expected_out > actual_out {
                let diff = expected_out - actual_out;
                println!(
                    "  get_amount_out overestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, token_y_decimals),
                    token_y_symbol,
                    diff
                );
            } else {
                let diff = actual_out - expected_out;
                println!(
                    "  get_amount_out underestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, token_y_decimals),
                    token_y_symbol,
                    diff
                );
            }
        }

        // Assert within tolerance
        assert!(
            diff_percent <= tolerance * 100.0,
            "get_amount_out differs from simulation by {:.6}% (exceeds {:.1}% tolerance)",
            diff_percent,
            tolerance * 100.0
        );

        unit_ok!()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn verify_y_to_x_matches_simulation() -> AResult<()> {
        _set_test_client();
        must_init_db().await;

        // Give services time to initialize
        sleep(Duration::from_millis(100)).await;

        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let amount_in = 1000_000_000; // 1000 USDC
        let min_amount_out = 0;

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

        println!("=== Testing Y->X (USDC->SOL) Direction ===");
        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", token_y_symbol, token_x_symbol);
        println!(
            "Amount in: {} {}",
            SimulationHelper::format_amount(amount_in, token_y_decimals),
            token_y_symbol
        );

        // Calculate expected output using get_amount_out
        let expected_out = config
            .get_amount_out(amount_in, &token_y_mint, &token_x_mint)
            .await?;
        println!(
            "\nExpected output (get_amount_out): {} {} (raw: {})",
            SimulationHelper::format_amount(expected_out, token_x_decimals),
            token_x_symbol,
            expected_out
        );

        // Simulate actual swap to get real output (Y->X direction)
        let result = simulate_swap_and_get_balance_diff(
            &POOL,
            &payer,
            amount_in,
            min_amount_out,
            false, // swap_x_to_y = false for Y->X
        )
        .await?;

        if let Some(err) = &result.error {
            println!("Simulation error: {}", err);
            assert!(false, "Simulation failed: {}", err);
        }

        println!("\nSimulation successful!");
        println!("Compute units consumed: {:?}", result.compute_units);

        let actual_out = result.balance_diff_out as u64;
        println!(
            "Actual output (simulation):      {} {} (raw: {})",
            SimulationHelper::format_amount(actual_out, token_x_decimals),
            token_x_symbol,
            actual_out
        );

        // Verify get_amount_out matches simulation
        let tolerance = 0.001; // 0.1% tolerance
        let diff_percent = if expected_out > 0 {
            ((expected_out as f64 - actual_out as f64).abs() / expected_out as f64) * 100.0
        } else {
            0.0
        };

        println!("\n=== Verification Results ===");
        println!("Expected (get_amount_out): {}", expected_out);
        println!("Actual (simulation):       {}", actual_out);
        println!("Difference:                {:.6}%", diff_percent);

        if diff_percent <= tolerance * 100.0 {
            println!(
                "✓ PASS: get_amount_out matches simulation within {:.1}% tolerance",
                tolerance * 100.0
            );
        } else {
            println!(
                "✗ FAIL: get_amount_out differs from simulation by {:.6}% (exceeds {:.1}% tolerance)",
                diff_percent,
                tolerance * 100.0
            );
            if expected_out > actual_out {
                let diff = expected_out - actual_out;
                println!(
                    "  get_amount_out overestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, token_x_decimals),
                    token_x_symbol,
                    diff
                );
            } else {
                let diff = actual_out - expected_out;
                println!(
                    "  get_amount_out underestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, token_x_decimals),
                    token_x_symbol,
                    diff
                );
            }
        }

        // Assert within tolerance
        assert!(
            diff_percent <= tolerance * 100.0,
            "get_amount_out differs from simulation by {:.6}% (exceeds {:.1}% tolerance)",
            diff_percent,
            tolerance * 100.0
        );

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

        // Test X->Y swap
        let result = simulate_swap_and_get_balance_diff(
            &pool,
            &payer,
            amount_in,
            min_amount_out,
            true, // X->Y direction
        )
        .await?;

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
