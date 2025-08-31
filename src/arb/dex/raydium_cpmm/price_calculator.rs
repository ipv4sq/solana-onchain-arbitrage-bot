use crate::arb::dex::interface::PoolDataLoader;
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
use crate::arb::global::enums::direction::Direction;
use crate::arb::pipeline::trade_strategy::token_balance_cache::get_balance_of_account;
use crate::arb::util::alias::{AResult, MintAddress};
use crate::arb::util::traits::option::OptionExt;
use crate::f;
use rust_decimal::Decimal;

impl RaydiumCpmmPoolData {
    pub async fn mid_price_for_quick_estimate(
        &self,
        from: &MintAddress,
        to: &MintAddress,
    ) -> AResult<DlmmQuote> {
        #[cfg(test)]
        let start_t0 = std::time::Instant::now();
        
        let token_0_cached = get_balance_of_account(&self.base_vault(), &self.base_mint())
            .await
            .or_err(f!(
                "Unable to get balance of owner {} mint {}",
                self.base_vault(),
                self.base_mint()
            ))?;
        
        #[cfg(test)]
        println!("    Token 0 balance fetch: {}ms", start_t0.elapsed().as_millis());
        #[cfg(test)]
        let start_t1 = std::time::Instant::now();
        
        let token_1_cached = get_balance_of_account(&self.quote_vault(), &self.quote_mint())
            .await
            .or_err(f!(
                "Unable to get balance of owner {} mint {}",
                self.quote_vault(),
                self.quote_mint()
            ))?;
        
        #[cfg(test)]
        println!("    Token 1 balance fetch: {}ms", start_t1.elapsed().as_millis());

        let token_0_balance = token_0_cached.amount;
        let token_0_decimals = token_0_cached.decimals;

        let token_1_balance = token_1_cached.amount;
        let token_1_decimals = token_1_cached.decimals;

        let token_0_balance_dec =
            Decimal::from(token_0_balance) / Decimal::from(10u64.pow(token_0_decimals as u32));
        let token_1_balance_dec =
            Decimal::from(token_1_balance) / Decimal::from(10u64.pow(token_1_decimals as u32));

        let price_token_0_per_token_1 = token_1_balance_dec / token_0_balance_dec;

        let mid_price = match self.dir(from, to) {
            Direction::XtoY => price_token_0_per_token_1,
            Direction::YtoX => Decimal::ONE / price_token_0_per_token_1,
        };

        Ok(DlmmQuote { mid_price })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arb::dex::interface::PoolDataLoader;
    use crate::arb::global::constant::mint::Mints;
    use crate::arb::global::state::rpc::rpc_client;
    use crate::arb::util::traits::pubkey::ToPubkey;
    use std::time::Instant;

    #[tokio::test]
    async fn test_cache_timing_detailed() {
        use crate::arb::database::mint_record::repository::MintRecordRepository;
        use crate::arb::global::constant::mint::Mints;
        use std::time::Instant;
        
        println!("\n=== Detailed Cache Timing Test ===");
        
        let wsol = Mints::WSOL;
        let eagle = "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey();
        
        // Test 1: Database query timing
        println!("\n--- Database Query Timing ---");
        let t_db_wsol = Instant::now();
        let db_result = MintRecordRepository::find_by_address(wsol).await;
        println!("DB query for WSOL: {}ms", t_db_wsol.elapsed().as_millis());
        match db_result {
            Ok(Some(record)) => println!("  Found: Symbol={}, Decimals={}", record.symbol, record.decimals),
            Ok(None) => println!("  Not found in DB"),
            Err(e) => println!("  Error: {}", e),
        }
        
        // Test 2: MintRecordRepository.get_mint timing (uses cache internally)
        println!("\n--- MintRecordRepository.get_mint() Timing ---");
        
        println!("First call to get_mint(WSOL):");
        let t1 = Instant::now();
        let result1 = MintRecordRepository::get_mint(&wsol).await;
        let elapsed1 = t1.elapsed().as_millis();
        println!("  Time: {}ms", elapsed1);
        if result1.is_ok() && result1.as_ref().unwrap().is_some() {
            println!("  Success: Got WSOL");
        }
        
        println!("Second call to get_mint(WSOL):");
        let t2 = Instant::now();
        let result2 = MintRecordRepository::get_mint(&wsol).await;
        let elapsed2 = t2.elapsed().as_millis();
        println!("  Time: {}ms", elapsed2);
        
        println!("First call to get_mint(EAGLE):");
        let t3 = Instant::now();
        let result3 = MintRecordRepository::get_mint(&eagle).await;
        let elapsed3 = t3.elapsed().as_millis();
        println!("  Time: {}ms", elapsed3);
        if let Ok(Some(record)) = result3 {
            println!("  Got: Symbol={}, Decimals={}", record.symbol, record.decimals);
        }
        
        // Test get_decimal specifically
        println!("\n--- MintRecordRepository.get_decimal() Timing ---");
        let t_decimal = Instant::now();
        let decimal_result = MintRecordRepository::get_decimal(&wsol).await;
        println!("get_decimal(WSOL): {}ms", t_decimal.elapsed().as_millis());
        if let Ok(Some(decimals)) = decimal_result {
            println!("  Decimals: {}", decimals);
        }
        
        // Test 3: get_balance_of_account specifically
        println!("\n--- get_balance_of_account Timing Breakdown ---");
        use crate::arb::pipeline::trade_strategy::token_balance_cache::get_balance_of_account;
        
        let vault = "SvbJANoKJmz6RqEJBj5gjPrfurkKzhfGXUvaEams48y".to_pubkey();
        
        // First call - should be slow
        println!("First call to get_balance_of_account:");
        let t_balance1 = Instant::now();
        let balance1 = get_balance_of_account(&vault, &wsol).await;
        println!("  Total time: {}ms", t_balance1.elapsed().as_millis());
        if let Some(amt) = balance1 {
            println!("  Result: amount={}, decimals={}", amt.amount, amt.decimals);
        }
        
        // Second call - should be fast (cached)
        println!("Second call to get_balance_of_account:");
        let t_balance2 = Instant::now();
        let balance2 = get_balance_of_account(&vault, &wsol).await;
        println!("  Total time: {}ms", t_balance2.elapsed().as_millis());
        
        // Test with a different vault/mint combination
        let vault2 = "FgPdQQ37kZVDqsfgPSLzC851mx9BMP6HRYe2ia4HDNLe".to_pubkey();
        println!("First call with different vault:");
        let t_balance3 = Instant::now();
        let balance3 = get_balance_of_account(&vault2, &eagle).await;
        println!("  Total time: {}ms", t_balance3.elapsed().as_millis());
        if let Some(amt) = balance3 {
            println!("  Result: amount={}, decimals={}", amt.amount, amt.decimals);
        }
        
        println!("\n=== End Cache Timing Test ===\n");
    }
    
    #[tokio::test]
    async fn test_mint_loading_timing() {
        use crate::arb::database::mint_record::loader::load_mint_from_address;
        use crate::arb::global::constant::mint::Mints;
        use std::time::Instant;
        
        println!("\n=== Mint Loading Timing Test ===");
        
        let wsol = Mints::WSOL;
        let eagle = "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey();
        
        // Test loading WSOL mint (should be fast as it's well-known)
        let t_wsol = Instant::now();
        let wsol_record = load_mint_from_address(&wsol).await;
        println!("WSOL mint load time: {}ms", t_wsol.elapsed().as_millis());
        if let Ok(record) = wsol_record {
            println!("  Symbol: {}, Decimals: {}", record.symbol, record.decimals);
        }
        
        // Test loading EAGLE mint (might be slower due to metadata fetch)
        let t_eagle = Instant::now();
        let eagle_record = load_mint_from_address(&eagle).await;
        println!("EAGLE mint load time: {}ms", t_eagle.elapsed().as_millis());
        if let Ok(record) = eagle_record {
            println!("  Symbol: {}, Decimals: {}", record.symbol, record.decimals);
        }
        
        // Test loading both again (should use cache)
        println!("\n--- Second load (should use cache) ---");
        
        let t_wsol2 = Instant::now();
        let _ = load_mint_from_address(&wsol).await;
        println!("WSOL mint load time (cached): {}ms", t_wsol2.elapsed().as_millis());
        
        let t_eagle2 = Instant::now();
        let _ = load_mint_from_address(&eagle).await;
        println!("EAGLE mint load time (cached): {}ms", t_eagle2.elapsed().as_millis());
        
        println!("\n=== End Mint Loading Test ===\n");
    }
    
    #[tokio::test]
    async fn test_direct_rpc_vault_fetch_timing() {
        use crate::arb::global::state::rpc::rpc_client;
        use std::time::Instant;
        
        println!("\n=== Direct RPC Vault Fetch Test ===");
        
        // Pool and vault addresses from the actual pool
        let token_0_vault = "SvbJANoKJmz6RqEJBj5gjPrfurkKzhfGXUvaEams48y".to_pubkey();
        let token_1_vault = "FgPdQQ37kZVDqsfgPSLzC851mx9BMP6HRYe2ia4HDNLe".to_pubkey();
        
        // Test 1: Fetch vaults sequentially
        println!("\n--- Sequential Fetches ---");
        let start = Instant::now();
        
        let t0_start = Instant::now();
        let account_0 = rpc_client()
            .get_account(&token_0_vault)
            .await
            .expect("Failed to fetch token 0 vault");
        println!("Token 0 vault fetch: {}ms (size: {} bytes)", 
            t0_start.elapsed().as_millis(), 
            account_0.data.len());
        
        let t1_start = Instant::now();
        let account_1 = rpc_client()
            .get_account(&token_1_vault)
            .await
            .expect("Failed to fetch token 1 vault");
        println!("Token 1 vault fetch: {}ms (size: {} bytes)", 
            t1_start.elapsed().as_millis(),
            account_1.data.len());
        
        println!("Total sequential time: {}ms", start.elapsed().as_millis());
        
        // Test 2: Fetch vaults in parallel using get_multiple_accounts
        println!("\n--- Parallel Fetch (get_multiple_accounts) ---");
        let start_parallel = Instant::now();
        
        let accounts = rpc_client()
            .get_multiple_accounts(&[token_0_vault, token_1_vault])
            .await
            .expect("Failed to fetch multiple accounts");
        
        println!("Both vaults fetched in parallel: {}ms", start_parallel.elapsed().as_millis());
        
        if let Some(acc0) = &accounts[0] {
            println!("  Token 0 vault size: {} bytes", acc0.data.len());
        }
        if let Some(acc1) = &accounts[1] {
            println!("  Token 1 vault size: {} bytes", acc1.data.len());
        }
        
        println!("\n=== End Direct RPC Test ===\n");
    }

    #[tokio::test]
    async fn test_raydium_cpmm_price_calculation() {
        let start = Instant::now();
        let mut last_checkpoint = start;
        
        println!("[  0ms] Test started");
        
        let pool_address = "BtGUffMEnxrzdjyC3kKAHjGMpG1UdZiVWXZUaSpUv13C".to_pubkey();
        let wsol = Mints::WSOL;
        let eagle = "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey();
        
        println!("[{:4}ms] Addresses prepared", start.elapsed().as_millis());
        last_checkpoint = Instant::now();

        let account = rpc_client()
            .get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account");
        
        println!("[{:4}ms] Pool account fetched from RPC (+{}ms)", 
            start.elapsed().as_millis(), 
            last_checkpoint.elapsed().as_millis());
        last_checkpoint = Instant::now();

        let pool_data =
            RaydiumCpmmPoolData::load_data(&account.data).expect("Failed to load pool data");
        
        println!("[{:4}ms] Pool data deserialized (+{}ms)", 
            start.elapsed().as_millis(),
            last_checkpoint.elapsed().as_millis());
        last_checkpoint = Instant::now();

        assert_eq!(pool_data.base_mint(), wsol);
        assert_eq!(pool_data.quote_mint(), eagle);

        let quote_wsol_to_eagle = pool_data
            .mid_price_for_quick_estimate(&wsol, &eagle)
            .await
            .expect("Failed to calculate mid price");
        
        println!("[{:4}ms] First price calculated (WSOL->EAGLE) (+{}ms)", 
            start.elapsed().as_millis(),
            last_checkpoint.elapsed().as_millis());
        last_checkpoint = Instant::now();

        let quote_eagle_to_wsol = pool_data
            .mid_price_for_quick_estimate(&eagle, &wsol)
            .await
            .expect("Failed to calculate mid price");
        
        println!("[{:4}ms] Second price calculated (EAGLE->WSOL) (+{}ms)", 
            start.elapsed().as_millis(),
            last_checkpoint.elapsed().as_millis());

        println!("\n=== Results ===");
        println!("1 WSOL = {} EAGLE", quote_wsol_to_eagle.mid_price);
        println!("1 EAGLE = {} WSOL", quote_eagle_to_wsol.mid_price);
        println!("\n=== Total time: {}ms ===", start.elapsed().as_millis());
    }
}
