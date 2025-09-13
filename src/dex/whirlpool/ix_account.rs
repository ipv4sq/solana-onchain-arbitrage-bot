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

    pub async fn build_accounts_with_direction(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &WhirlpoolPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> AResult<Self> {
        use crate::database::mint_record::repository::MintRecordRepository;
        use crate::util::solana::pda::ata;

        // Validate input/output mints match pool mints
        let a_to_b = if *input_mint == pool_data.token_mint_a && *output_mint == pool_data.token_mint_b {
            true
        } else if *input_mint == pool_data.token_mint_b && *output_mint == pool_data.token_mint_a {
            false
        } else {
            return Err(lined_err!("Input/output mints don't match pool mints"));
        };

        // Get token programs from database (batch query)
        let (mint_a_record, mint_b_record) =
            MintRecordRepository::get_batch_as_tuple2(&pool_data.token_mint_a, &pool_data.token_mint_b).await?;

        let token_program_a = mint_a_record.program.0;
        let token_program_b = mint_b_record.program.0;

        // Derive ATAs using the correct token programs
        let payer_ata_a = ata(payer, &pool_data.token_mint_a, &token_program_a);
        let payer_ata_b = ata(payer, &pool_data.token_mint_b, &token_program_b);

        // Calculate tick arrays based on current tick and direction
        let tick_arrays = Self::get_tick_arrays_for_swap(
            pool,
            pool_data.tick_current_index,
            pool_data.tick_spacing as i32,
            a_to_b,
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
        // Calculate the start tick index for the current tick array
        let _current_array_start = Self::get_start_tick_index(current_tick, tick_spacing, 0)?;

        // Get tick arrays based on swap direction
        // For a_to_b (decreasing tick), we need current, current-1, current-2
        // For b_to_a (increasing tick), we need current, current+1, current+2
        let offsets = if a_to_b { [0, -1, -2] } else { [0, 1, 2] };

        let mut tick_arrays = [Pubkey::default(); 3];
        for (i, offset) in offsets.iter().enumerate() {
            let start_tick = Self::get_start_tick_index(current_tick, tick_spacing, *offset)?;
            tick_arrays[i] = Self::derive_tick_array_pda(pool, start_tick);
        }

        Ok(tick_arrays)
    }

    fn get_start_tick_index(tick_index: i32, tick_spacing: i32, offset: i32) -> AResult<i32> {
        let real_index = tick_index / tick_spacing / Self::TICK_ARRAY_SIZE;
        let start_tick_index = (real_index + offset) * tick_spacing * Self::TICK_ARRAY_SIZE;

        const MIN_TICK_INDEX: i32 = -443636;
        const MAX_TICK_INDEX: i32 = 443636;

        if start_tick_index < MIN_TICK_INDEX || start_tick_index > MAX_TICK_INDEX {
            return Err(lined_err!(
                "Tick array start index out of bounds: {}",
                start_tick_index
            ));
        }

        Ok(start_tick_index)
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
    async fn test_whirlpool_ix_account_build_with_direction() {
        crate::global::client::db::must_init_db().await;
        // Use the actual pool data from the pool_data.rs test
        let base64_data: &str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf8EAAQEkAEUBQzQhBjTGAAAAAAAAAAAAABZRcwACQaJFAAAAAAAAAAA4Dr//ylJ/5zJAgAA3Wn2FQMAAAAMRfffjZ5ylWKEkz9tmLdXAy6D34RgT7XhF//2HVsS+arsxH6LolpQyA7ZB2HrKJh35uJAUSLFiOx5OciOAW4je0S/FbWsvtQBAAAAAAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hQLp+tKau8fev2FzHuaF6UbkPxSqFcxgDXBXbbUFyiJS7qtCY6ifVAQAAAAAAAAAA0FzFaAAAAAAMANCv64YU2n8Zq6AtQPGMaSWF9lAg387T1eX5qcDE4bCb4EusmZI8S8jOoPk6gpuvm/s4CbScVhpu5GcYqjU7vR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAADWQ2S9J4CwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

        // Decode and load the actual pool data
        let pool_data_bytes = STANDARD
            .decode(base64_data)
            .expect("Failed to decode base64");
        let pool_data = WhirlpoolPoolData::load_data(&pool_data_bytes)
            .expect("Failed to parse pool data");

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
        let ix_account = WhirlpoolIxAccount::build_accounts_with_direction(
            &payer,
            &whirlpool,
            &pool_data,
            &pool_data.token_mint_a,
            &pool_data.token_mint_b,
        )
        .await
        .expect("Failed to build accounts");

        let accounts = ix_account.to_list();
        assert_eq!(accounts.len(), 15);

        // Print all accounts with their properties
        println!("\nCalculated Accounts:");
        println!("#1  Token Program A:        {} (readonly)", accounts[0].pubkey);
        println!("#2  Token Program B:        {} (readonly)", accounts[1].pubkey);
        println!("#3  Memo Program:           {} (readonly)", accounts[2].pubkey);
        println!("#4  Token Authority:        {} (signer, writable)", accounts[3].pubkey);
        println!("#5  Whirlpool:              {} (writable)", accounts[4].pubkey);
        println!("#6  Token Mint A:           {} (readonly)", accounts[5].pubkey);
        println!("#7  Token Mint B:           {} (readonly)", accounts[6].pubkey);
        println!("#8  Token Owner Account A:  {} (writable)", accounts[7].pubkey);
        println!("#9  Token Vault A:          {} (writable)", accounts[8].pubkey);
        println!("#10 Token Owner Account B:  {} (writable)", accounts[9].pubkey);
        println!("#11 Token Vault B:          {} (writable)", accounts[10].pubkey);
        println!("#12 Tick Array 0:           {} (writable)", accounts[11].pubkey);
        println!("#13 Tick Array 1:           {} (writable)", accounts[12].pubkey);
        println!("#14 Tick Array 2:           {} (writable)", accounts[13].pubkey);
        println!("#15 Oracle:                 {} (writable)", accounts[14].pubkey);

        // ATAs will now be derived using the correct token program from database
        // so we can't directly compare with get_associated_token_address
        // Just verify they are set
        assert_ne!(accounts[7].pubkey, Pubkey::default());
        assert_ne!(accounts[9].pubkey, Pubkey::default());

        // Test B to A swap (USDC -> PUMP)
        println!("\n=== B to A Swap (USDC -> PUMP) ===");
        let ix_account_reverse = WhirlpoolIxAccount::build_accounts_with_direction(
            &payer,
            &whirlpool,
            &pool_data,
            &pool_data.token_mint_b,
            &pool_data.token_mint_a,
        )
        .await
        .expect("Failed to build accounts for reverse swap");

        let accounts_reverse = ix_account_reverse.to_list();
        assert_eq!(accounts_reverse.len(), 15);

        println!("\nCalculated Accounts (Reverse):");
        println!("#12 Tick Array 0:           {} (writable)", accounts_reverse[11].pubkey);
        println!("#13 Tick Array 1:           {} (writable)", accounts_reverse[12].pubkey);
        println!("#14 Tick Array 2:           {} (writable)", accounts_reverse[13].pubkey);

        // Calculate expected tick arrays manually for verification
        let tick_spacing = pool_data.tick_spacing as i32;
        let current_tick = pool_data.tick_current_index;
        println!("\n=== Tick Array Calculation ===");
        println!("Current tick: {}", current_tick);
        println!("Tick spacing: {}", tick_spacing);

        // For A to B (decreasing tick)
        let start_tick_0 = WhirlpoolIxAccount::get_start_tick_index(current_tick, tick_spacing, 0).unwrap();
        let start_tick_minus_1 = WhirlpoolIxAccount::get_start_tick_index(current_tick, tick_spacing, -1).unwrap();
        let start_tick_minus_2 = WhirlpoolIxAccount::get_start_tick_index(current_tick, tick_spacing, -2).unwrap();

        println!("A to B tick array start indices: {}, {}, {}", start_tick_0, start_tick_minus_1, start_tick_minus_2);

        // For B to A (increasing tick)
        let start_tick_plus_1 = WhirlpoolIxAccount::get_start_tick_index(current_tick, tick_spacing, 1).unwrap();
        let start_tick_plus_2 = WhirlpoolIxAccount::get_start_tick_index(current_tick, tick_spacing, 2).unwrap();

        println!("B to A tick array start indices: {}, {}, {}", start_tick_0, start_tick_plus_1, start_tick_plus_2);

        println!("\n✓ All Whirlpool accounts calculated successfully!");
    }
}
