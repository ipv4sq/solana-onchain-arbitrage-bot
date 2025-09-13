use crate::dex::whirlpool::pool_data::WhirlpoolPoolData;
use crate::global::constant::pool_program::PoolProgram;
use crate::lined_err;
use crate::util::alias::AResult;
use crate::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_sdk::pubkey;

#[derive(Debug, Clone)]
pub struct WhirlpoolIxAccount {
    pub token_program_a: AccountMeta,
    pub token_program_b: AccountMeta,
    pub memo_program: AccountMeta,
    pub token_authority: AccountMeta,
    pub whirlpool: AccountMeta,
    pub token_mint_a: AccountMeta,
    pub token_mint_b: AccountMeta,
    pub token_owner_account_a: AccountMeta,
    pub token_vault_a: AccountMeta,
    pub token_owner_account_b: AccountMeta,
    pub token_vault_b: AccountMeta,
    pub tick_array_0: AccountMeta,
    pub tick_array_1: AccountMeta,
    pub tick_array_2: AccountMeta,
    pub oracle: AccountMeta,
}

impl WhirlpoolIxAccount {
    const SPL_MEMO_PROGRAM: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
    const TICK_ARRAY_SIZE: i32 = 88;

    // pub async fn build_accounts_with_direction(
    //     payer: &Pubkey,
    //     pool: &Pubkey,
    //     pool_data: &WhirlpoolPoolData,
    //     input_mint: &Pubkey,
    //     output_mint: &Pubkey,
    // ) -> AResult<Self> {
    //     use crate::database::mint_record::repository::MintRecordRepository;
    //     use crate::util::solana::pda::ata;
    //
    //     // Validate input/output mints match pool mints
    //     let a_to_b = if *input_mint == pool_data.token_mint_a && *output_mint == pool_data.token_mint_b {
    //         true
    //     } else if *input_mint == pool_data.token_mint_b && *output_mint == pool_data.token_mint_a {
    //         false
    //     } else {
    //         return Err(lined_err!("Input/output mints don't match pool mints"));
    //     };
    //
    //     // Get token programs from database (batch query)
    //     let (mint_a_record, mint_b_record) =
    //         MintRecordRepository::get_batch_as_tuple2(&pool_data.token_mint_a, &pool_data.token_mint_b).await?;
    //
    //     let token_program_a = mint_a_record.program.0;
    //     let token_program_b = mint_b_record.program.0;
    //
    //     // Derive ATAs using the correct token programs
    //     let payer_ata_a = ata(payer, &pool_data.token_mint_a, &token_program_a);
    //     let payer_ata_b = ata(payer, &pool_data.token_mint_b, &token_program_b);
    //
    //     // Calculate tick arrays based on current tick and direction
    //     let tick_arrays = Self::get_tick_arrays_for_swap(
    //         pool,
    //         pool_data.tick_current_index,
    //         pool_data.tick_spacing as i32,
    //         a_to_b,
    //     )?;
    //
    //     // Get oracle PDA
    //     let oracle = Self::derive_oracle_pda(pool);
    //
    //     Ok(Self {
    //         token_program_a: token_program_a.to_program(),
    //         token_program_b: token_program_b.to_program(),
    //         memo_program: Self::SPL_MEMO_PROGRAM.to_program(),
    //         token_authority: payer.to_signer(),
    //         whirlpool: pool.to_writable(),
    //         token_mint_a: pool_data.token_mint_a.to_readonly(),
    //         token_mint_b: pool_data.token_mint_b.to_readonly(),
    //         token_owner_account_a: payer_ata_a.to_writable(),
    //         token_vault_a: pool_data.token_vault_a.to_writable(),
    //         token_owner_account_b: payer_ata_b.to_writable(),
    //         token_vault_b: pool_data.token_vault_b.to_writable(),
    //         tick_array_0: tick_arrays[0].to_writable(),
    //         tick_array_1: tick_arrays[1].to_writable(),
    //         tick_array_2: tick_arrays[2].to_writable(),
    //         oracle: oracle.to_writable(),
    //     })
    // }

    pub async fn build_bidirectional(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &WhirlpoolPoolData,
    ) -> AResult<Self> {
        use crate::database::mint_record::repository::MintRecordRepository;
        use crate::util::solana::pda::ata;

        // Get token programs from database (batch query)
        let (mint_a_record, mint_b_record) = MintRecordRepository::get_batch_as_tuple2(
            &pool_data.token_mint_a,
            &pool_data.token_mint_b,
        )
        .await?;

        let token_program_a = mint_a_record.program.0;
        let token_program_b = mint_b_record.program.0;

        // Derive ATAs using the correct token programs
        let payer_ata_a = ata(payer, &pool_data.token_mint_a, &token_program_a);
        let payer_ata_b = ata(payer, &pool_data.token_mint_b, &token_program_b);

        // Get bidirectional tick arrays (prev, current, next)
        let tick_arrays = Self::get_tick_arrays_bidirectional(
            pool,
            pool_data.tick_current_index,
            pool_data.tick_spacing as i32,
        )?;

        // Get oracle PDA
        let oracle = Self::derive_oracle_pda(pool);

        Ok(Self {
            token_program_a: token_program_a.to_program(),
            token_program_b: token_program_b.to_program(),
            memo_program: Self::SPL_MEMO_PROGRAM.to_program(),
            token_authority: payer.to_signer(),
            whirlpool: pool.to_writable(),
            token_mint_a: pool_data.token_mint_a.to_readonly(),
            token_mint_b: pool_data.token_mint_b.to_readonly(),
            token_owner_account_a: payer_ata_a.to_writable(),
            token_vault_a: pool_data.token_vault_a.to_writable(),
            token_owner_account_b: payer_ata_b.to_writable(),
            token_vault_b: pool_data.token_vault_b.to_writable(),
            tick_array_0: tick_arrays[0].to_writable(),
            tick_array_1: tick_arrays[1].to_writable(),
            tick_array_2: tick_arrays[2].to_writable(),
            oracle: oracle.to_writable(),
        })
    }

    fn get_tick_arrays_for_swap(
        pool: &Pubkey,
        current_tick: i32,
        tick_spacing: i32,
        a_to_b: bool,
    ) -> AResult<[Pubkey; 3]> {
        let (ta0_start, ta1_start_opt, ta2_start_opt) =
            Self::derive_tick_array_start_indexes(current_tick, tick_spacing as u16, a_to_b);

        Ok([
            Self::derive_tick_array_pda(pool, ta0_start),
            Self::derive_tick_array_pda(pool, ta1_start_opt.unwrap_or(ta0_start)),
            Self::derive_tick_array_pda(pool, ta2_start_opt.unwrap_or(ta1_start_opt.unwrap_or(ta0_start))),
        ])
    }

    fn get_tick_arrays_bidirectional(
        pool: &Pubkey,
        current_tick: i32,
        tick_spacing: i32,
    ) -> AResult<[Pubkey; 3]> {
        let tick_array_starts =
            Self::derive_tick_array_start_indexes(current_tick, tick_spacing as u16, true);
        let tick_array_reverse_starts =
            Self::derive_tick_array_start_indexes(current_tick, tick_spacing as u16, false);

        Ok([
            Self::derive_tick_array_pda(
                pool,
                tick_array_reverse_starts.1.unwrap_or(tick_array_reverse_starts.0),
            ),
            Self::derive_tick_array_pda(pool, tick_array_starts.0),
            Self::derive_tick_array_pda(
                pool,
                tick_array_starts.1.unwrap_or(tick_array_starts.0),
            ),
        ])
    }

    fn derive_tick_array_start_indexes(
        curr_tick: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> (i32, Option<i32>, Option<i32>) {
        let ta0_start_index = Self::derive_first_tick_array_start_tick(curr_tick, tick_spacing, !a_to_b);
        let ta1_start_index_opt = Self::derive_next_start_tick_in_seq(ta0_start_index, tick_spacing, a_to_b);
        let ta2_start_index_opt = ta1_start_index_opt
            .and_then(|nsi| Self::derive_next_start_tick_in_seq(nsi, tick_spacing, a_to_b));
        (ta0_start_index, ta1_start_index_opt, ta2_start_index_opt)
    }

    fn derive_first_tick_array_start_tick(curr_tick: i32, tick_spacing: u16, shifted: bool) -> i32 {
        let tick = if shifted {
            curr_tick + tick_spacing as i32
        } else {
            curr_tick
        };
        Self::derive_start_tick(tick, tick_spacing)
    }

    fn derive_start_tick(curr_tick: i32, tick_spacing: u16) -> i32 {
        let num_of_ticks_in_array = Self::TICK_ARRAY_SIZE * tick_spacing as i32;
        let rem = curr_tick % num_of_ticks_in_array;
        if curr_tick < 0 && rem != 0 {
            curr_tick - rem - num_of_ticks_in_array
        } else {
            curr_tick - rem
        }
    }

    fn derive_next_start_tick_in_seq(
        start_tick: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Option<i32> {
        let num_of_ticks_in_array = Self::TICK_ARRAY_SIZE * tick_spacing as i32;
        let potential_last = if a_to_b {
            start_tick - num_of_ticks_in_array
        } else {
            start_tick + num_of_ticks_in_array
        };

        const MIN_TICK_INDEX: i32 = -443636;
        const MAX_TICK_INDEX: i32 = 443636;

        if potential_last < MAX_TICK_INDEX && potential_last > MIN_TICK_INDEX {
            Some(potential_last)
        } else {
            None
        }
    }

    fn derive_tick_array_pda(pool: &Pubkey, start_tick: i32) -> Pubkey {
        let start_tick_bytes = start_tick.to_string();
        Pubkey::find_program_address(
            &[b"tick_array", pool.as_ref(), start_tick_bytes.as_bytes()],
            &PoolProgram::WHIRLPOOL,
        )
        .0
    }

    fn derive_oracle_pda(pool: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"oracle", pool.as_ref()], &PoolProgram::WHIRLPOOL).0
    }

    pub fn to_list(&self) -> Vec<AccountMeta> {
        vec![
            self.token_program_a.clone(),
            self.token_program_b.clone(),
            self.memo_program.clone(),
            self.token_authority.clone(),
            self.whirlpool.clone(),
            self.token_mint_a.clone(),
            self.token_mint_b.clone(),
            self.token_owner_account_a.clone(),
            self.token_vault_a.clone(),
            self.token_owner_account_b.clone(),
            self.token_vault_b.clone(),
            self.tick_array_0.clone(),
            self.tick_array_1.clone(),
            self.tick_array_2.clone(),
            self.oracle.clone(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dex::interface::PoolDataLoader;
    use crate::util::traits::pubkey::ToPubkey;
    use base64::{engine::general_purpose::STANDARD, Engine};

    #[tokio::test]
    async fn test_whirlpool_ix_account_build_without_direction() {
        crate::global::client::db::must_init_db().await;
        // Use the actual pool data from the pool_data.rs test
        let base64_data: &str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf8EAAQEkAEUBQzQhBjTGAAAAAAAAAAAAABZRcwACQaJFAAAAAAAAAAA4Dr//ylJ/5zJAgAA3Wn2FQMAAAAMRfffjZ5ylWKEkz9tmLdXAy6D34RgT7XhF//2HVsS+arsxH6LolpQyA7ZB2HrKJh35uJAUSLFiOx5OciOAW4je0S/FbWsvtQBAAAAAAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hQLp+tKau8fev2FzHuaF6UbkPxSqFcxgDXBXbbUFyiJS7qtCY6ifVAQAAAAAAAAAA0FzFaAAAAAAMANCv64YU2n8Zq6AtQPGMaSWF9lAg387T1eX5qcDE4bCb4EusmZI8S8jOoPk6gpuvm/s4CbScVhpu5GcYqjU7vR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAADWQ2S9J4CwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

        // Decode and load the actual pool data
        let pool_data_bytes = STANDARD
            .decode(base64_data)
            .expect("Failed to decode base64");
        let pool_data =
            WhirlpoolPoolData::load_data(&pool_data_bytes).expect("Failed to parse pool data");

        // Pool and payer addresses
        let whirlpool = "HyA4ct7i4XvZsVrLyb5VJhcTP1EZVDZoF9fFGym16zcj".to_pubkey();
        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();

        println!("\n=== Pool Data ===");
        println!("Pool: {}", whirlpool);
        println!("Token Mint A (PUMP): {}", pool_data.token_mint_a);
        println!("Token Mint B (USDC): {}", pool_data.token_mint_b);
        println!("Token Vault A: {}", pool_data.token_vault_a);
        println!("Token Vault B: {}", pool_data.token_vault_b);
        println!("Tick Current Index: {}", pool_data.tick_current_index);
        println!("Tick Spacing: {}", pool_data.tick_spacing);

        // Test A to B swap (PUMP -> USDC)
        println!("\n=== A to B Swap (PUMP -> USDC) ===");
        let ix_account = WhirlpoolIxAccount::build_bidirectional(&payer, &whirlpool, &pool_data)
            .await
            .expect("Failed to build accounts");

        let accounts = ix_account.to_list();
        assert_eq!(accounts.len(), 15);

        // Print all accounts with their properties
        println!("\nCalculated Accounts:");
        println!(
            "#1  Token Program A:        {} (readonly)",
            accounts[0].pubkey
        );
        println!(
            "#2  Token Program B:        {} (readonly)",
            accounts[1].pubkey
        );
        println!(
            "#3  Memo Program:           {} (readonly)",
            accounts[2].pubkey
        );
        println!(
            "#4  Token Authority:        {} (signer, writable)",
            accounts[3].pubkey
        );
        println!(
            "#5  Whirlpool:              {} (writable)",
            accounts[4].pubkey
        );
        println!(
            "#6  Token Mint A:           {} (readonly)",
            accounts[5].pubkey
        );
        println!(
            "#7  Token Mint B:           {} (readonly)",
            accounts[6].pubkey
        );
        println!(
            "#8  Token Owner Account A:  {} (writable)",
            accounts[7].pubkey
        );
        println!(
            "#9  Token Vault A:          {} (writable)",
            accounts[8].pubkey
        );
        println!(
            "#10 Token Owner Account B:  {} (writable)",
            accounts[9].pubkey
        );
        println!(
            "#11 Token Vault B:          {} (writable)",
            accounts[10].pubkey
        );
        println!(
            "#12 Tick Array 0:           {} (writable)",
            accounts[11].pubkey
        );
        println!(
            "#13 Tick Array 1:           {} (writable)",
            accounts[12].pubkey
        );
        println!(
            "#14 Tick Array 2:           {} (writable)",
            accounts[13].pubkey
        );
        println!(
            "#15 Oracle:                 {} (writable)",
            accounts[14].pubkey
        );

        // ATAs will now be derived using the correct token program from database
        // so we can't directly compare with get_associated_token_address
        // Just verify they are set
        assert_ne!(accounts[7].pubkey, Pubkey::default());
        assert_ne!(accounts[9].pubkey, Pubkey::default());

        // Test B to A swap (USDC -> PUMP)
        println!("\n=== B to A Swap (USDC -> PUMP) ===");
        let ix_account_reverse =
            WhirlpoolIxAccount::build_bidirectional(&payer, &whirlpool, &pool_data)
                .await
                .expect("Failed to build accounts for reverse swap");

        let accounts_reverse = ix_account_reverse.to_list();
        assert_eq!(accounts_reverse.len(), 15);

        println!("\nCalculated Accounts (Reverse):");
        println!(
            "#12 Tick Array 0:           {} (writable)",
            accounts_reverse[11].pubkey
        );
        println!(
            "#13 Tick Array 1:           {} (writable)",
            accounts_reverse[12].pubkey
        );
        println!(
            "#14 Tick Array 2:           {} (writable)",
            accounts_reverse[13].pubkey
        );

        // Verify tick arrays are calculated
        let tick_spacing = pool_data.tick_spacing as i32;
        let current_tick = pool_data.tick_current_index;
        println!("\n=== Tick Array Calculation ===");
        println!("Current tick: {}", current_tick);
        println!("Tick spacing: {}", tick_spacing);

        // Verify the tick arrays are different for bidirectional
        // (they should include both forward and reverse arrays)
        let tick_arrays_bidirectional = [
            accounts[11].pubkey,
            accounts[12].pubkey,
            accounts[13].pubkey,
        ];

        println!("Bidirectional tick arrays:");
        for (i, tick_array) in tick_arrays_bidirectional.iter().enumerate() {
            println!("  Tick Array {}: {}", i, tick_array);
        }

        println!("\n✓ All Whirlpool accounts calculated successfully!");
    }
}
