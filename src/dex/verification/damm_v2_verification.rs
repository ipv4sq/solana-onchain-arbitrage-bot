
#[cfg(test)]
mod tests {
    use crate::convention::chain::simulation::SimulationHelper;
    use crate::database::mint_record::repository::MintRecordRepository;
    use crate::dex::interface::PoolConfig;
    use crate::dex::legacy_interface::InputAccountUtil;
    use crate::dex::meteora_damm_v2::config::MeteoraDammV2Config;
    use crate::dex::meteora_damm_v2::misc::input_account::MeteoraDammV2InputAccount;
    use crate::dex::verification::common::simulate_damm_v2_swap_and_get_balance_diff;
    use crate::global::client::db::must_init_db;
    use crate::sdk::solana_rpc::rpc::_set_test_client;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::pubkey::ToPubkey;
    use solana_program::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use std::time::Duration;
    use tokio::time::sleep;

    static POOL: Pubkey = pubkey!("8Pm2kZpnxD3hoMmt4bjStX2Pw2Z9abpbHzZxMPqxPmie");

    #[tokio::test(flavor = "multi_thread")]
    async fn verify_a_to_b_matches_simulation() -> AResult<()> {
        _set_test_client();
        must_init_db().await;

        // Give services time to initialize
        sleep(Duration::from_millis(100)).await;

        let payer = "HbjRwJqFQJxEEhczcPznd8BJci3wj9fRzAPsP8bSuvCN".to_pubkey();
        let amount_in = 1000000000; // 1 SOL (token A has 9 decimals)
        let min_amount_out = 0;

        // Get pool config for token info
        let config = MeteoraDammV2Config::from_address(&POOL).await?;
        let token_a_mint = config.pool_data.token_a_mint;
        let token_b_mint = config.pool_data.token_b_mint;

        // Fetch token symbols and decimals from database
        let token_a_record = MintRecordRepository::get(&token_a_mint).await;
        let token_a_symbol = token_a_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| token_a_mint.to_string()[..6].to_string());
        let token_a_decimals = token_a_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(9u32);

        let token_b_record = MintRecordRepository::get(&token_b_mint).await;
        let token_b_symbol = token_b_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| token_b_mint.to_string()[..6].to_string());
        let token_b_decimals = token_b_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(6u32);

        println!("=== Testing A->B Direction ===");
        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", token_a_symbol, token_b_symbol);
        println!(
            "Amount in: {} {}",
            SimulationHelper::format_amount(amount_in, token_a_decimals),
            token_a_symbol
        );

        // Calculate expected output using get_amount_out
        let expected_out = config
            .get_amount_out(amount_in, &token_a_mint, &token_b_mint)
            .await?;
        println!(
            "\nExpected output (get_amount_out): {} {} (raw: {})",
            SimulationHelper::format_amount(expected_out, token_b_decimals),
            token_b_symbol,
            expected_out
        );

        // Simulate actual swap to get real output (A->B direction)
        let result = simulate_damm_v2_swap_and_get_balance_diff(
            &POOL,
            &payer,
            amount_in,
            min_amount_out,
            true, // swap_a_to_b = true
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
            SimulationHelper::format_amount(actual_out, token_b_decimals),
            token_b_symbol,
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
                    SimulationHelper::format_amount(diff, token_b_decimals),
                    token_b_symbol,
                    diff
                );
            } else {
                let diff = actual_out - expected_out;
                println!(
                    "  get_amount_out underestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, token_b_decimals),
                    token_b_symbol,
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
    async fn verify_b_to_a_matches_simulation() -> AResult<()> {
        _set_test_client();
        must_init_db().await;

        // Give services time to initialize
        sleep(Duration::from_millis(100)).await;

        let payer = "HbjRwJqFQJxEEhczcPznd8BJci3wj9fRzAPsP8bSuvCN".to_pubkey();
        let amount_in = 100_000_000; // 100 USDC (token B has 6 decimals typically)
        let min_amount_out = 0;

        // Get pool config for token info
        let config = MeteoraDammV2Config::from_address(&POOL).await?;
        let token_a_mint = config.pool_data.token_a_mint;
        let token_b_mint = config.pool_data.token_b_mint;

        // Fetch token symbols and decimals from database
        let token_a_record = MintRecordRepository::get(&token_a_mint).await;
        let token_a_symbol = token_a_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| token_a_mint.to_string()[..6].to_string());
        let token_a_decimals = token_a_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(9u32);

        let token_b_record = MintRecordRepository::get(&token_b_mint).await;
        let token_b_symbol = token_b_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| token_b_mint.to_string()[..6].to_string());
        let token_b_decimals = token_b_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(6u32);

        println!("=== Testing B->A Direction ===");
        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", token_b_symbol, token_a_symbol);
        println!(
            "Amount in: {} {}",
            SimulationHelper::format_amount(amount_in, token_b_decimals),
            token_b_symbol
        );

        // Calculate expected output using get_amount_out
        let expected_out = config
            .get_amount_out(amount_in, &token_b_mint, &token_a_mint)
            .await?;
        println!(
            "\nExpected output (get_amount_out): {} {} (raw: {})",
            SimulationHelper::format_amount(expected_out, token_a_decimals),
            token_a_symbol,
            expected_out
        );

        // Simulate actual swap to get real output (B->A direction)
        let result = simulate_damm_v2_swap_and_get_balance_diff(
            &POOL,
            &payer,
            amount_in,
            min_amount_out,
            false, // swap_a_to_b = false for B->A
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
            SimulationHelper::format_amount(actual_out, token_a_decimals),
            token_a_symbol,
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
                    SimulationHelper::format_amount(diff, token_a_decimals),
                    token_a_symbol,
                    diff
                );
            } else {
                let diff = actual_out - expected_out;
                println!(
                    "  get_amount_out underestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, token_a_decimals),
                    token_a_symbol,
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
    async fn generate_solana_validator_cmd() -> AResult<()> {
        must_init_db().await;

        let config = MeteoraDammV2Config::from_address(&POOL).await?;
        let payer = "HbjRwJqFQJxEEhczcPznd8BJci3wj9fRzAPsP8bSuvCN".to_pubkey();
        let accounts = MeteoraDammV2InputAccount::build_accounts_no_matter_direction_size(
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
