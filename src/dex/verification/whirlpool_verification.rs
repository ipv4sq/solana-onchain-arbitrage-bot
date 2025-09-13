#[cfg(test)]
mod tests {
    use crate::convention::chain::simulation::SimulationHelper;
    use crate::database::mint_record::repository::MintRecordRepository;
    use crate::dex::interface::PoolConfig;
    use crate::dex::verification::common::simulate_whirlpool_swap_and_get_balance_diff;
    use crate::dex::whirlpool::config::WhirlpoolConfig;
    use crate::dex::whirlpool::ix_account::WhirlpoolIxAccount;
    use crate::global::client::db::must_init_db;
    use crate::global::constant::pool_program::PoolProgram;
    use crate::global::enums::dex_type::DexType;
    use crate::sdk::rpc::client::_set_test_client;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::account_meta::ToAccountMeta;
    use solana_sdk::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use std::time::Duration;
    use tokio::time::sleep;

    static POOL: Pubkey = pubkey!("HyA4ct7i4XvZsVrLyb5VJhcTP1EZVDZoF9fFGym16zcj");
    static PAYER: Pubkey = pubkey!("MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa");

    #[tokio::test]
    async fn verify_a_to_b_matches_simulation() -> AResult<()> {
        must_init_db().await;
        _set_test_client();

        // Give services time to initialize
        sleep(Duration::from_millis(500)).await;

        let amount_in = 1_000_000_000; // 1 SOL
        let min_amount_out = 0;

        // Get pool config for token info
        let config = WhirlpoolConfig::from_address(&POOL).await?;
        let token_a_mint = config.pool_data.token_mint_a;
        let token_b_mint = config.pool_data.token_mint_b;

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

        println!("=== Testing A->B Direction (Whirlpool) ===");
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

        // Debug: Print pool data
        println!("\nPool Data:");
        println!("  Token A mint: {}", config.pool_data.token_mint_a);
        println!("  Token B mint: {}", config.pool_data.token_mint_b);
        println!("  Current tick: {}", config.pool_data.tick_current_index);
        println!("  Liquidity: {}", config.pool_data.liquidity);
        println!("  Sqrt price: {}", config.pool_data.sqrt_price);
        println!("  Tick spacing: {}", config.pool_data.tick_spacing);
        println!("  Fee rate: {}", config.pool_data.fee_rate);

        // Build accounts to inspect them
        let accounts = WhirlpoolIxAccount::build_accounts_with_direction(
            &PAYER,
            &POOL,
            &config.pool_data,
            &token_a_mint,
            &token_b_mint,
        )
        .await?;
        println!("\nAccounts for simulation:");
        println!(
            "  #0  Token Program A:       {}",
            accounts.token_program_a.pubkey
        );
        println!(
            "  #1  Token Program B:       {}",
            accounts.token_program_b.pubkey
        );
        println!(
            "  #2  Memo Program:          {}",
            accounts.memo_program.pubkey
        );
        println!(
            "  #3  Token Authority:       {}",
            accounts.token_authority.pubkey
        );
        println!("  #4  Whirlpool:             {}", accounts.whirlpool.pubkey);
        println!(
            "  #5  Token Mint A:          {}",
            accounts.token_mint_a.pubkey
        );
        println!(
            "  #6  Token Mint B:          {}",
            accounts.token_mint_b.pubkey
        );
        println!(
            "  #7  User token A:          {}",
            accounts.token_owner_account_a.pubkey
        );
        println!(
            "  #8  Token vault A:         {}",
            accounts.token_vault_a.pubkey
        );
        println!(
            "  #9  User token B:          {}",
            accounts.token_owner_account_b.pubkey
        );
        println!(
            "  #10 Token vault B:         {}",
            accounts.token_vault_b.pubkey
        );
        println!(
            "  #11 Tick Array 0:          {}",
            accounts.tick_array_0.pubkey
        );
        println!(
            "  #12 Tick Array 1:          {}",
            accounts.tick_array_1.pubkey
        );
        println!(
            "  #13 Tick Array 2:          {}",
            accounts.tick_array_2.pubkey
        );
        println!("  #14 Oracle:                {}", accounts.oracle.pubkey);

        // Simulate actual swap to get real output (a->b direction)
        let result = simulate_whirlpool_swap_and_get_balance_diff(
            &POOL,
            &PAYER,
            amount_in,
            min_amount_out,
            true, // a_to_b = true
            &token_a_mint,
            &token_b_mint,
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
        let tolerance = 0.01; // 1% tolerance for Whirlpool due to tick-based pricing
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

    #[tokio::test]
    async fn verify_b_to_a_matches_simulation() -> AResult<()> {
        must_init_db().await;
        _set_test_client();

        // Give services time to initialize
        sleep(Duration::from_millis(500)).await;

        let amount_in = 1_000_000; // 1 USDC (6 decimals)
        let min_amount_out = 0;

        // Get pool config for token info
        let config = WhirlpoolConfig::from_address(&POOL).await?;
        let token_a_mint = config.pool_data.token_mint_a;
        let token_b_mint = config.pool_data.token_mint_b;

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

        println!("=== Testing B->A Direction (Whirlpool) ===");
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

        // Debug: Print pool data
        println!("\nPool Data:");
        println!("  Token A mint: {}", config.pool_data.token_mint_a);
        println!("  Token B mint: {}", config.pool_data.token_mint_b);
        println!("  Current tick: {}", config.pool_data.tick_current_index);
        println!("  Liquidity: {}", config.pool_data.liquidity);
        println!("  Sqrt price: {}", config.pool_data.sqrt_price);
        println!("  Tick spacing: {}", config.pool_data.tick_spacing);
        println!("  Fee rate: {}", config.pool_data.fee_rate);

        // Simulate actual swap to get real output (b->a direction)
        let result = simulate_whirlpool_swap_and_get_balance_diff(
            &POOL,
            &PAYER,
            amount_in,
            min_amount_out,
            false, // a_to_b = false (so b_to_a)
            &token_b_mint,
            &token_a_mint,
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
        let tolerance = 0.01; // 1% tolerance for Whirlpool due to tick-based pricing
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
    async fn build_command() -> AResult<()> {
        must_init_db().await;

        let config = WhirlpoolConfig::from_address(&POOL).await?;

        // Build accounts for both directions to get all tick arrays
        let mut accounts =
            WhirlpoolIxAccount::build_bidirectional(&PAYER, &POOL, &config.pool_data)
                .await?
                .to_list();

        accounts.push("4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_readonly());
        accounts.push(PoolProgram::WHIRLPOOL.to_readonly());
        accounts.push(PAYER.to_writable());

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
