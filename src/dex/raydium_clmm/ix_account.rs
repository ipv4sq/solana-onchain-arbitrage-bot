use crate::dex::raydium_clmm::pool_data::RaydiumClmmPoolData;
use crate::global::enums::dex_type::DexType;
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
    pub bitmap_extension: AccountMeta,
    // Remaining accounts
    pub tick_arrays: Vec<AccountMeta>,
}

impl RaydiumClmmIxAccount {
    pub fn to_list(&self) -> Vec<AccountMeta> {
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

        accounts.extend(self.tick_arrays.clone());

        accounts
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
            3, // Get more tick arrays to be safe
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
        let bitmap_extension = Self::generate_bitmap(pool);
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
            bitmap_extension: bitmap_extension.to_writable(),
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

    fn generate_bitmap(pool: &Pubkey) -> Pubkey {
        let bitmap_extension = Pubkey::find_program_address(
            &[POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(), &pool.as_ref()],
            &DexType::RaydiumClmm.owner_program_id(),
        )
        .0;
        bitmap_extension
    }
}
pub const POOL_TICK_ARRAY_BITMAP_SEED: &str = "pool_tick_array_bitmap_extension";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::traits::pubkey::ToPubkey;

    #[test]
    fn bitmap() {
        let pool = pubkey!("3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv");
        let generated = RaydiumClmmIxAccount::generate_bitmap(&pool);
        println!("{:#?}", generated);
    }

    #[test]
    fn test_swap_v2_accounts_from_solscan() {
        let expected_accounts = vec![
            "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa".to_pubkey(), // Payer
            "3h2e43PunVA5K34vwKCLHWhZF4aZpyaC9RmxvshGAQpL".to_pubkey(), // Amm Config
            "3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv".to_pubkey(), // Pool State
            "8VYWdU14V78rcDepwmNt54bb1aam5qVUMUpEtW8oCn1E".to_pubkey(), // Input Token Account
            "CTyFguG69kwYrzk24P3UuBvY1rR5atu9kf2S6XEwAU8X".to_pubkey(), // Output Token Account
            "4D3oJLZN6f4v51MRY2xJ5FdRPZEXyY8VRXRx5hJMiSYp".to_pubkey(), // Input Vault
            "HQszk12vRBpJB3Gg5Y7DMZPRVGHMKb7F4Qa7iF2vmfE3".to_pubkey(), // Output Vault
            "3Y695CuQ8AP4anbwAqiEBeQF9KxqHFr8piEwvw3UePnQ".to_pubkey(), // Observation State
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_pubkey(), // Token Program
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".to_pubkey(), // Token Program 2022
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr".to_pubkey(), // Memo Program
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey(), // Input Vault Mint (USDC)
            "So11111111111111111111111111111111111111112".to_pubkey(), // Output Vault Mint (WSOL)
            "4NFvUKqknMpoe6CWTzK758B8ojVLzURL5pC6MtiaJ8TQ".to_pubkey(), // Tick array 1
            "FRXEFGPQqVg2kzdFdWYmeyyiKMgCBudwCkQWWpnv4kQi".to_pubkey(), // Tick array 2
            "ww3UCP5SPttfTaFY32CuXhuJ3VxF9khav1QAw1Wenpq".to_pubkey(), // Tick array 3
            "ANgzDPdViH7HEM2qbUXWw2PqCaSTg3QDJ9VEvJafdj4T".to_pubkey(), // Tick array 4
        ];

        // Create a mock RaydiumClmmIxAccount with the Solscan data
        let ix_account = RaydiumClmmIxAccount {
            payer: AccountMeta::new(
                "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa".to_pubkey(),
                true,
            ),
            amm_config: AccountMeta::new_readonly(
                "3h2e43PunVA5K34vwKCLHWhZF4aZpyaC9RmxvshGAQpL".to_pubkey(),
                false,
            ),
            pool_state: AccountMeta::new(
                "3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv".to_pubkey(),
                false,
            ),
            input_token_account: AccountMeta::new(
                "8VYWdU14V78rcDepwmNt54bb1aam5qVUMUpEtW8oCn1E".to_pubkey(),
                false,
            ),
            output_token_account: AccountMeta::new(
                "CTyFguG69kwYrzk24P3UuBvY1rR5atu9kf2S6XEwAU8X".to_pubkey(),
                false,
            ),
            input_vault: AccountMeta::new(
                "4D3oJLZN6f4v51MRY2xJ5FdRPZEXyY8VRXRx5hJMiSYp".to_pubkey(),
                false,
            ),
            output_vault: AccountMeta::new(
                "HQszk12vRBpJB3Gg5Y7DMZPRVGHMKb7F4Qa7iF2vmfE3".to_pubkey(),
                false,
            ),
            observation_state: AccountMeta::new(
                "3Y695CuQ8AP4anbwAqiEBeQF9KxqHFr8piEwvw3UePnQ".to_pubkey(),
                false,
            ),
            token_program: AccountMeta::new_readonly(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_pubkey(),
                false,
            ),
            token_program_2022: AccountMeta::new_readonly(
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".to_pubkey(),
                false,
            ),
            memo_program: AccountMeta::new_readonly(
                "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr".to_pubkey(),
                false,
            ),
            input_vault_mint: AccountMeta::new_readonly(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey(),
                false,
            ),
            output_vault_mint: AccountMeta::new_readonly(
                "So11111111111111111111111111111111111111112".to_pubkey(),
                false,
            ),
            tick_arrays: vec![
                AccountMeta::new(
                    "FRXEFGPQqVg2kzdFdWYmeyyiKMgCBudwCkQWWpnv4kQi".to_pubkey(),
                    false,
                ),
                AccountMeta::new(
                    "ww3UCP5SPttfTaFY32CuXhuJ3VxF9khav1QAw1Wenpq".to_pubkey(),
                    false,
                ),
                AccountMeta::new(
                    "ANgzDPdViH7HEM2qbUXWw2PqCaSTg3QDJ9VEvJafdj4T".to_pubkey(),
                    false,
                ),
            ],
        };

        let accounts = ix_account.to_list();

        // Verify total number of accounts
        assert_eq!(accounts.len(), 17, "Expected 17 accounts for swap_v2");

        // Verify each account matches expected
        for (i, (account, expected)) in accounts.iter().zip(expected_accounts.iter()).enumerate() {
            assert_eq!(
                account.pubkey,
                *expected,
                "Account #{} mismatch: got {:?}, expected {:?}",
                i + 1,
                account.pubkey,
                expected
            );
        }

        // Verify account permissions
        assert!(accounts[0].is_signer, "Payer should be signer");
        assert!(accounts[0].is_writable, "Payer should be writable");
        assert!(accounts[2].is_writable, "Pool state should be writable");
        assert!(
            accounts[3].is_writable,
            "Input token account should be writable"
        );
        assert!(
            accounts[4].is_writable,
            "Output token account should be writable"
        );
        assert!(accounts[5].is_writable, "Input vault should be writable");
        assert!(accounts[6].is_writable, "Output vault should be writable");
        assert!(
            accounts[7].is_writable,
            "Observation state should be writable"
        );
        assert!(!accounts[8].is_writable, "Token program should be readonly");
        assert!(
            !accounts[9].is_writable,
            "Token program 2022 should be readonly"
        );
        assert!(!accounts[10].is_writable, "Memo program should be readonly");
        assert!(
            !accounts[11].is_writable,
            "Input vault mint should be readonly"
        );
        assert!(
            !accounts[12].is_writable,
            "Output vault mint should be readonly"
        );

        // All tick arrays should be writable
        for i in 13..17 {
            assert!(
                accounts[i].is_writable,
                "Tick array #{} should be writable",
                i - 12
            );
        }
    }
}
