#[cfg(test)]
mod tests {
    use crate::convention::chain::simulation::SimulationHelper;
    use crate::database::mint_record::repository::MintRecordRepository;
    use crate::dex::interface::PoolConfig;
    use crate::dex::raydium_clmm::config::RaydiumClmmConfig;
    use crate::dex::raydium_clmm::ix_account::RaydiumClmmIxAccount;
    use crate::dex::verification::common::simulate_raydium_clmm_swap_and_get_balance_diff;
    use crate::global::client::db::must_init_db;
    use crate::global::enums::dex_type::DexType;
    use crate::sdk::rpc::client::_set_test_client;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::account_meta::ToAccountMeta;
    use solana_sdk::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use std::time::Duration;
    use tokio::time::sleep;

    static POOL: Pubkey = pubkey!("3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv");
    static PAYER: Pubkey = pubkey!("MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa");

    #[tokio::test]
    async fn verify_base_to_quote_matches_simulation() -> AResult<()> {
        must_init_db().await;
        _set_test_client();

        // Give services time to initialize
        sleep(Duration::from_millis(500)).await;

        let amount_in = 1_000_000_000; // 1 SOL
        let min_amount_out = 0;

        // Get pool config for token info
        let config = RaydiumClmmConfig::from_address(&POOL).await?;
        let base_mint = config.base_mint;
        let quote_mint = config.quote_mint;

        // Fetch token symbols and decimals from database
        let base_record = MintRecordRepository::get(&base_mint).await;
        let base_symbol = base_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| base_mint.to_string()[..6].to_string());
        let base_decimals = base_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(6u32);

        let quote_record = MintRecordRepository::get(&quote_mint).await;
        let quote_symbol = quote_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| quote_mint.to_string()[..6].to_string());
        let quote_decimals = quote_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(9u32);

        println!("=== Testing Base->Quote Direction (Raydium CLMM) ===");
        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", base_symbol, quote_symbol);
        println!(
            "Amount in: {} {}",
            SimulationHelper::format_amount(amount_in, base_decimals),
            base_symbol
        );

        // Calculate expected output using get_amount_out
        let expected_out = config
            .get_amount_out(amount_in, &base_mint, &quote_mint)
            .await?;
        println!(
            "\nExpected output (get_amount_out): {} {} (raw: {})",
            SimulationHelper::format_amount(expected_out, quote_decimals),
            quote_symbol,
            expected_out
        );

        // Debug: Print pool data
        println!("\nPool Data:");
        println!("  Token 0 mint: {}", config.pool_data.token_mint_0);
        println!("  Token 1 mint: {}", config.pool_data.token_mint_1);
        println!("  Base mint: {}", config.base_mint);
        println!("  Quote mint: {}", config.quote_mint);
        println!("  Current tick: {}", config.pool_data.tick_current);
        println!("  Liquidity: {}", config.pool_data.liquidity);
        println!("  Tick spacing: {}", config.pool_data.tick_spacing);

        // Build accounts to inspect them
        let accounts = RaydiumClmmIxAccount::build_accounts_with_direction(
            &PAYER,
            &POOL,
            &config.pool_data,
            &base_mint,
            &quote_mint,
        )
        .await?;
        println!("\nAccounts for simulation:");
        println!("  Pool: {}", accounts.pool_state.pubkey);
        println!("  AMM Config: {}", accounts.amm_config.pubkey);
        println!("  Input vault: {}", accounts.input_vault.pubkey);
        println!("  Output vault: {}", accounts.output_vault.pubkey);
        println!("  User input token: {}", accounts.input_token_account.pubkey);
        println!("  User output token: {}", accounts.output_token_account.pubkey);

        // Simulate actual swap to get real output (base->quote direction)
        let result = simulate_raydium_clmm_swap_and_get_balance_diff(
            &POOL,
            &PAYER,
            amount_in,
            min_amount_out,
            true, // is_base_input = true
            &base_mint,
            &quote_mint,
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
            SimulationHelper::format_amount(actual_out, quote_decimals),
            quote_symbol,
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
                    SimulationHelper::format_amount(diff, quote_decimals),
                    quote_symbol,
                    diff
                );
            } else {
                let diff = actual_out - expected_out;
                println!(
                    "  get_amount_out underestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, quote_decimals),
                    quote_symbol,
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
    async fn verify_quote_to_base_matches_simulation() -> AResult<()> {
        must_init_db().await;
        _set_test_client();

        // Give services time to initialize
        sleep(Duration::from_millis(500)).await;

        let amount_in = 1_000_000_000; // 1 SOL (quote token, 9 decimals)
        let min_amount_out = 0;

        // Get pool config for token info
        let config = RaydiumClmmConfig::from_address(&POOL).await?;
        let base_mint = config.base_mint;
        let quote_mint = config.quote_mint;

        // Fetch token symbols and decimals from database
        let base_record = MintRecordRepository::get(&base_mint).await;
        let base_symbol = base_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| base_mint.to_string()[..6].to_string());
        let base_decimals = base_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(6u32);

        let quote_record = MintRecordRepository::get(&quote_mint).await;
        let quote_symbol = quote_record
            .as_ref()
            .map(|m| m.repr.clone())
            .unwrap_or_else(|| quote_mint.to_string()[..6].to_string());
        let quote_decimals = quote_record
            .as_ref()
            .and_then(|m| m.decimals.try_into().ok())
            .unwrap_or(9u32);

        println!("=== Testing Quote->Base Direction (Raydium CLMM) ===");
        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", quote_symbol, base_symbol);
        println!(
            "Amount in: {} {}",
            SimulationHelper::format_amount(amount_in, quote_decimals),
            quote_symbol
        );

        // Calculate expected output using get_amount_out
        let expected_out = config
            .get_amount_out(amount_in, &quote_mint, &base_mint)
            .await?;
        println!(
            "\nExpected output (get_amount_out): {} {} (raw: {})",
            SimulationHelper::format_amount(expected_out, base_decimals),
            base_symbol,
            expected_out
        );

        // Simulate actual swap to get real output (quote->base direction)
        let result = simulate_raydium_clmm_swap_and_get_balance_diff(
            &POOL,
            &PAYER,
            amount_in,
            min_amount_out,
            true, // is_base_input = true (for amount_in semantics)
            &quote_mint,
            &base_mint,
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
            SimulationHelper::format_amount(actual_out, base_decimals),
            base_symbol,
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
                    SimulationHelper::format_amount(diff, base_decimals),
                    base_symbol,
                    diff
                );
            } else {
                let diff = actual_out - expected_out;
                println!(
                    "  get_amount_out underestimated by {} {} ({})",
                    SimulationHelper::format_amount(diff, base_decimals),
                    base_symbol,
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
    async fn build_command() -> AResult<()> {
        must_init_db().await;

        let config = RaydiumClmmConfig::from_address(&POOL).await?;
        let mut accounts = RaydiumClmmIxAccount::build_accounts_with_direction(
            &PAYER,
            &POOL,
            &config.pool_data,
            &config.base_mint,
            &config.quote_mint,
        )
        .await?
        .to_list();
        accounts.push("4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_readonly());

        // Build the validator command from the accounts array
        let bootstrap_cmd = format!(
            "solana-test-validator --reset \\
  --url https://api.mainnet-beta.solana.com \\
  {}",
            accounts
                .iter()
                .map(|account| {
                    let dex_type = DexType::determine_from(&account.pubkey);
                    return if dex_type != DexType::Unknown {
                        format!("--clone-upgradeable-program {}", account.pubkey)
                    } else {
                        format!("--clone {}", account.pubkey)
                    };
                })
                .collect::<Vec<_>>()
                .join(" \\\n  ")
        );

        println!("\n{}\n", bootstrap_cmd);

        unit_ok!()
    }
}
