use crate::dex::raydium_clmm::pool_data::RaydiumClmmPoolData;
use crate::util::alias::AResult;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_sdk::pubkey;

#[derive(Debug, Clone)]
pub struct RaydiumClmmIxAccount {
    pub payer: AccountMeta,
    pub amm_config: AccountMeta,
    pub pool_state: AccountMeta,
    pub input_token_account: AccountMeta,
    pub output_token_account: AccountMeta,
    pub input_vault: AccountMeta,
    pub output_vault: AccountMeta,
    pub observation_state: AccountMeta,
    pub token_program: AccountMeta,
    pub token_program_2022: AccountMeta,
    pub memo_program: AccountMeta,
    pub input_vault_mint: AccountMeta,
    pub output_vault_mint: AccountMeta,
    // Remaining accounts
    pub tickarray_bitmap_extension: Option<AccountMeta>,
    pub tick_arrays: Vec<AccountMeta>,
}

impl RaydiumClmmIxAccount {
    pub fn to_vec(&self) -> Vec<AccountMeta> {
        let mut accounts = vec![
            self.payer.clone(),
            self.amm_config.clone(),
            self.pool_state.clone(),
            self.input_token_account.clone(),
            self.output_token_account.clone(),
            self.input_vault.clone(),
            self.output_vault.clone(),
            self.observation_state.clone(),
            self.token_program.clone(),
            self.token_program_2022.clone(),
            self.memo_program.clone(),
            self.input_vault_mint.clone(),
            self.output_vault_mint.clone(),
        ];

        // Add remaining accounts
        if let Some(ref bitmap_ext) = self.tickarray_bitmap_extension {
            accounts.push(bitmap_ext.clone());
        }
        accounts.extend(self.tick_arrays.clone());

        accounts
    }

    pub async fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &RaydiumClmmPoolData,
    ) -> AResult<RaydiumClmmIxAccount> {
        // Default to token_0 -> token_1 swap (zero_for_one = true)
        Self::build_accounts_with_direction(
            payer,
            pool,
            pool_data,
            &pool_data.token_mint_0,
            &pool_data.token_mint_1,
        )
        .await
    }

    pub async fn build_accounts_with_direction(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &RaydiumClmmPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> AResult<RaydiumClmmIxAccount> {
        use crate::global::constant::pool_program::PoolProgram;
        use crate::global::constant::token_program::TokenProgram;
        use crate::util::traits::account_meta::ToAccountMeta;
        use spl_associated_token_account::get_associated_token_address_with_program_id;

        let token_0_mint = &pool_data.token_mint_0;
        let token_1_mint = &pool_data.token_mint_1;

        // Determine swap direction
        let zero_for_one = input_mint == token_0_mint;

        // Determine input/output vaults based on direction
        let (input_vault, output_vault, input_vault_mint, output_vault_mint) = if zero_for_one {
            (
                pool_data.token_vault_0,
                pool_data.token_vault_1,
                *token_0_mint,
                *token_1_mint,
            )
        } else {
            (
                pool_data.token_vault_1,
                pool_data.token_vault_0,
                *token_1_mint,
                *token_0_mint,
            )
        };

        let token_program = TokenProgram::SPL_TOKEN.to_program();

        let user_input_token =
            get_associated_token_address_with_program_id(payer, input_mint, &token_program.pubkey);
        let user_output_token =
            get_associated_token_address_with_program_id(payer, output_mint, &token_program.pubkey);

        // Generate tick arrays based on swap direction
        let tick_array_pubkeys = Self::derive_tick_arrays_for_swap(
            pool,
            pool_data.tick_current,
            pool_data.tick_spacing as i32,
            zero_for_one,
            5, // Get more tick arrays to be safe
        );

        const SPL_MEMO_PROGRAM: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

        // Derive tick array bitmap extension PDA (optional)
        let (tickarray_bitmap_extension, _) = Pubkey::find_program_address(
            &[b"bitmap_extension", pool.as_ref()],
            &PoolProgram::RAYDIUM_CLMM,
        );

        // Convert tick array pubkeys to AccountMetas
        let tick_arrays: Vec<AccountMeta> = tick_array_pubkeys
            .into_iter()
            .map(|pubkey| pubkey.to_writable())
            .collect();

        Ok(RaydiumClmmIxAccount {
            payer: payer.to_signer(),
            amm_config: pool_data.amm_config.to_readonly(),
            pool_state: pool.to_writable(),
            input_token_account: user_input_token.to_writable(),
            output_token_account: user_output_token.to_writable(),
            input_vault: input_vault.to_writable(),
            output_vault: output_vault.to_writable(),
            observation_state: pool_data.observation_key.to_writable(),
            token_program,
            token_program_2022: TokenProgram::TOKEN_2022.to_program(),
            memo_program: SPL_MEMO_PROGRAM.to_program(),
            input_vault_mint: input_vault_mint.to_readonly(),
            output_vault_mint: output_vault_mint.to_readonly(),
            tickarray_bitmap_extension: Some(tickarray_bitmap_extension.to_writable()),
            tick_arrays,
        })
    }

    fn derive_tick_arrays_for_swap(
        pool: &Pubkey,
        current_tick: i32,
        tick_spacing: i32,
        zero_for_one: bool,
        num_arrays: usize,
    ) -> Vec<Pubkey> {
        use crate::global::constant::pool_program::PoolProgram;

        let mut tick_arrays = Vec::with_capacity(num_arrays);
        let mut current_index = Self::get_tick_array_start_index(current_tick, tick_spacing);

        for _ in 0..num_arrays {
            let (tick_array, _) = Pubkey::find_program_address(
                &[b"tick_array", pool.as_ref(), &current_index.to_be_bytes()],
                &PoolProgram::RAYDIUM_CLMM,
            );
            tick_arrays.push(tick_array);

            if zero_for_one {
                current_index -= 60 * tick_spacing;
            } else {
                current_index += 60 * tick_spacing;
            }
        }

        tick_arrays
    }

    fn get_tick_array_start_index(tick: i32, tick_spacing: i32) -> i32 {
        const TICK_ARRAY_SIZE: i32 = 60;
        let ticks_in_array = TICK_ARRAY_SIZE * tick_spacing;
        let mut start = tick / ticks_in_array;
        if tick < 0 && tick % ticks_in_array != 0 {
            start -= 1;
        }
        start * ticks_in_array
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_build_accounts_both_directions() {
        const PAYER: &str = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa";
        const POOL: &str = "3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv";
        const WSOL: &str = "So11111111111111111111111111111111111111112";
        const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

        let payer = PAYER.to_pubkey();
        let pool = POOL.to_pubkey();

        let pool_data = RaydiumClmmPoolData {
            bump: [255],
            amm_config: "3h2e43PunVA5K34vwKCLHWhZF4aZpyaC9RmxvshGAQpL".to_pubkey(),
            owner: "CJKrW95iMGECdjWtdDnWDAx2cBH7pFE9VywnULfwMapf".to_pubkey(),
            token_mint_0: "So11111111111111111111111111111111111111112".to_pubkey(),
            token_mint_1: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey(),
            token_vault_0: "4ct7br2vTPzfdmY3S5HLtTxcGSBfn6pnw98hsS6v359A".to_pubkey(),
            token_vault_1: "5it83u57VRrVgc51oNV19TTmAJuffPx5GtGwQr7gQNUo".to_pubkey(),
            observation_key: "3Y695CuQ8AP4anbwAqiEBeQF9KxqHFr8piEwvw3UePnQ".to_pubkey(),
            mint_decimals_0: 9,
            mint_decimals_1: 6,
            tick_spacing: 1,
            liquidity: 295329867866867,
            sqrt_price_x64: 9082733951060270080,
            tick_current: -14171,
            padding3: 0,
            padding4: 0,
            fee_growth_global_0_x64: 3559325779701363188,
            fee_growth_global_1_x64: 626808862733012568,
            protocol_fees_token_0: 12806897,
            protocol_fees_token_1: 5455291,
            swap_in_amount_token_0: 49476625175238063,
            swap_out_amount_token_1: 8189820303939792,
            swap_in_amount_token_1: 8230440941799804,
            swap_out_amount_token_0: 49669505447062833,
            status: 0,
            padding: [0; 7],
            reward_infos: [Default::default(), Default::default(), Default::default()],
            tick_array_bitmap: [0; 16],
            total_fees_token_0: 16624154768112,
            total_fees_claimed_token_0: 16316837962051,
            total_fees_token_1: 2765436108315,
            total_fees_claimed_token_1: 2704587280086,
            fund_fees_token_0: 16530299,
            fund_fees_token_1: 4757642,
            open_time: 1723037622,
            recent_epoch: 848,
            padding1: [0; 24],
            padding2: [0; 32],
        };

        // Test USDC -> WSOL (opposite direction from Solscan)
        let usdc_to_wsol = RaydiumClmmIxAccount::build_accounts_with_direction(
            &payer,
            &pool,
            &pool_data,
            &USDC.to_pubkey(),
            &WSOL.to_pubkey(),
        )
        .await
        .expect("Failed to build accounts for USDC->WSOL");

        println!("\n=== Test 1: USDC -> WSOL (matches Solscan transaction) ===\n");

        println!("#1 - Payer: {} ✓", usdc_to_wsol.payer.pubkey);
        assert_eq!(usdc_to_wsol.payer.pubkey.to_string(), PAYER);

        println!("#2 - AMM Config: {} ✓", usdc_to_wsol.amm_config.pubkey);
        assert_eq!(
            usdc_to_wsol.amm_config.pubkey.to_string(),
            "3h2e43PunVA5K34vwKCLHWhZF4aZpyaC9RmxvshGAQpL"
        );

        println!("#3 - Pool State: {} ✓", usdc_to_wsol.pool_state.pubkey);

        println!(
            "#4 - Input Token Account: {}",
            usdc_to_wsol.input_token_account.pubkey
        );
        println!("     Expected from Solscan: 8VYWdU14V78rcDepwmNt54bb1aam5qVUMUpEtW8oCn1E");

        println!(
            "#5 - Output Token Account: {}",
            usdc_to_wsol.output_token_account.pubkey
        );
        println!("     Expected from Solscan: CTyFguG69kwYrzk24P3UuBvY1rR5atu9kf2S6XEwAU8X");

        println!(
            "#6 - Input Vault: {} (should be USDC vault)",
            usdc_to_wsol.input_vault.pubkey
        );
        assert_eq!(usdc_to_wsol.input_vault.pubkey, pool_data.token_vault_1);

        println!(
            "#7 - Output Vault: {} (should be WSOL vault)",
            usdc_to_wsol.output_vault.pubkey
        );
        assert_eq!(usdc_to_wsol.output_vault.pubkey, pool_data.token_vault_0);

        println!(
            "#8 - Observation State: {} ✓",
            usdc_to_wsol.observation_state.pubkey
        );

        println!(
            "#12 - Input Vault Mint: {} (USDC)",
            usdc_to_wsol.input_vault_mint.pubkey
        );
        assert_eq!(usdc_to_wsol.input_vault_mint.pubkey.to_string(), USDC);

        println!(
            "#13 - Output Vault Mint: {} (WSOL)",
            usdc_to_wsol.output_vault_mint.pubkey
        );
        assert_eq!(usdc_to_wsol.output_vault_mint.pubkey.to_string(), WSOL);

        println!("\nRemaining Accounts (USDC->WSOL, zero_for_one=false):");
        if let Some(ref bitmap_ext) = usdc_to_wsol.tickarray_bitmap_extension {
            println!("Tickarray Bitmap Extension: {}", bitmap_ext.pubkey);
        }

        println!("Tick Arrays ({} total):", usdc_to_wsol.tick_arrays.len());
        for (i, tick_array) in usdc_to_wsol.tick_arrays.iter().enumerate() {
            println!("  Tick Array {}: {}", i, tick_array.pubkey);
        }

        println!("\nExpected from Solscan:");
        println!("  #14: 4NFvUKqknMpoe6CWTzK758B8ojVLzURL5pC6MtiaJ8TQ");
        println!("  #15: FRXEFGPQqVg2kzdFdWYmeyyiKMgCBudwCkQWWpnv4kQi");
        println!("  #16: ww3UCP5SPttfTaFY32CuXhuJ3VxF9khav1QAw1Wenpq");
        println!("  #17: ANgzDPdViH7HEM2qbUXWw2PqCaSTg3QDJ9VEvJafdj4T");

        // Test WSOL -> USDC (default direction)
        let wsol_to_usdc = RaydiumClmmIxAccount::build_accounts_with_direction(
            &payer,
            &pool,
            &pool_data,
            &WSOL.to_pubkey(),
            &USDC.to_pubkey(),
        )
        .await
        .expect("Failed to build accounts for WSOL->USDC");

        println!("\n=== Test 2: WSOL -> USDC (opposite direction) ===\n");

        println!(
            "Input Token Account: {}",
            wsol_to_usdc.input_token_account.pubkey
        );
        println!(
            "Output Token Account: {}",
            wsol_to_usdc.output_token_account.pubkey
        );

        println!(
            "Input Vault: {} (should be WSOL vault)",
            wsol_to_usdc.input_vault.pubkey
        );
        assert_eq!(wsol_to_usdc.input_vault.pubkey, pool_data.token_vault_0);

        println!(
            "Output Vault: {} (should be USDC vault)",
            wsol_to_usdc.output_vault.pubkey
        );
        assert_eq!(wsol_to_usdc.output_vault.pubkey, pool_data.token_vault_1);

        println!("\nTick Arrays (WSOL->USDC, zero_for_one=true):");
        for (i, tick_array) in wsol_to_usdc.tick_arrays.iter().enumerate() {
            println!("  Tick Array {}: {}", i, tick_array.pubkey);
        }

        let tick_start = RaydiumClmmIxAccount::get_tick_array_start_index(
            pool_data.tick_current,
            pool_data.tick_spacing as i32,
        );
        println!("\n=== Tick Analysis ===");
        println!("Current Tick: {}", pool_data.tick_current);
        println!("Tick Spacing: {}", pool_data.tick_spacing);
        println!("Current Tick Array Start Index: {}", tick_start);
    }
}
