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
            self.bitmap_extension.clone(),
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
        use crate::database::mint_record::repository::MintRecordRepository;
        use crate::util::solana::pda::ata;
        use crate::util::traits::account_meta::ToAccountMeta;

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

        // Get token programs from database (batch query)
        let (input_mint_record, output_mint_record) =
            MintRecordRepository::get_batch_as_tuple2(input_mint, output_mint).await?;

        let input_token_program = input_mint_record.program.0;
        let output_token_program = output_mint_record.program.0;

        // Derive ATAs using the correct token programs
        let user_input_token = ata(payer, input_mint, &input_token_program);
        let user_output_token = ata(payer, output_mint, &output_token_program);

        // Generate tick arrays generically (current, previous, next)
        let tick_array_pubkeys = Self::derive_tick_arrays_generic(
            pool,
            pool_data.tick_current,
            pool_data.tick_spacing as i32,
        );

        const SPL_MEMO_PROGRAM: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

        // Convert tick array pubkeys to AccountMetas
        let tick_arrays: Vec<AccountMeta> = tick_array_pubkeys
            .into_iter()
            .map(|pubkey| pubkey.to_writable())
            .collect();
        let bitmap_extension = Self::generate_bitmap(pool);
        // Raydium CLMM always needs both token programs in the accounts
        use crate::global::constant::token_program::TokenProgram;

        Ok(RaydiumClmmIxAccount {
            payer: payer.to_signer(),
            amm_config: pool_data.amm_config.to_readonly(),
            pool_state: pool.to_writable(),
            input_token_account: user_input_token.to_writable(),
            output_token_account: user_output_token.to_writable(),
            input_vault: input_vault.to_writable(),
            output_vault: output_vault.to_writable(),
            observation_state: pool_data.observation_key.to_writable(),
            token_program: TokenProgram::SPL_TOKEN.to_program(),
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

    fn derive_tick_arrays_generic(
        pool: &Pubkey,
        current_tick: i32,
        tick_spacing: i32,
    ) -> Vec<Pubkey> {
        use crate::global::constant::pool_program::PoolProgram;

        let current_index = Self::get_tick_array_start_index(current_tick, tick_spacing);
        let array_offset = 60 * tick_spacing;

        // Generate current, previous (-1), and next (+1) tick arrays
        let indices = vec![
            current_index,                // Current tick array
            current_index - array_offset, // Previous tick array
            current_index + array_offset, // Next tick array
        ];

        indices
            .into_iter()
            .map(|index| {
                Pubkey::find_program_address(
                    &[b"tick_array", pool.as_ref(), &index.to_be_bytes()],
                    &PoolProgram::RAYDIUM_CLMM,
                )
                .0
            })
            .collect()
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
    fn test_generic_tick_arrays() {
        let pool = pubkey!("3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv");
        let current_tick = -28971; // Example tick from a real pool
        let tick_spacing = 10;

        let tick_arrays =
            RaydiumClmmIxAccount::derive_tick_arrays_generic(&pool, current_tick, tick_spacing);

        // Should always return 3 tick arrays
        assert_eq!(tick_arrays.len(), 3);

        // Verify the tick arrays are different
        assert_ne!(tick_arrays[0], tick_arrays[1]);
        assert_ne!(tick_arrays[0], tick_arrays[2]);
        assert_ne!(tick_arrays[1], tick_arrays[2]);

        println!("Current tick array: {}", tick_arrays[0]);
        println!("Previous tick array (-1): {}", tick_arrays[1]);
        println!("Next tick array (+1): {}", tick_arrays[2]);
    }
}
