use crate::arb::pool::interface::InputAccountUtil;
use crate::arb::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use anyhow::Result;
use itertools::concat;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiPartiallyDecodedInstruction,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MeteoraDlmmInputAccounts {
    pub lb_pair: AccountMeta,
    pub bin_array_bitmap_extension: AccountMeta,
    pub reverse_x: AccountMeta,
    pub reverse_y: AccountMeta,
    pub user_token_in: AccountMeta,
    pub user_token_out: AccountMeta,
    pub token_x_mint: AccountMeta,
    pub token_y_mint: AccountMeta,
    pub oracle: AccountMeta,
    pub host_fee_in: AccountMeta,
    pub user: AccountMeta,
    pub token_x_program: AccountMeta,
    pub token_y_program: AccountMeta,
    pub event_authority: AccountMeta,
    pub program: AccountMeta,
    pub bin_arrays: Vec<AccountMeta>,
}

impl InputAccountUtil<MeteoraDlmmInputAccounts, MeteoraDlmmPoolData>
    for MeteoraDlmmInputAccounts
{
    fn restore_from(
        ix: &UiPartiallyDecodedInstruction,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<MeteoraDlmmInputAccounts> {
        use crate::arb::tx::account::{create_account_meta, get_parsed_accounts};

        if ix.accounts.len() < 15 {
            return Err(anyhow::anyhow!(
                "Invalid number of accounts for Meteora DLMM swap: expected at least 15, got {}",
                ix.accounts.len()
            ));
        }

        let parsed_accounts = get_parsed_accounts(tx)?;

        // Extract bin arrays (all accounts after index 14)
        let mut bin_arrays = Vec::new();
        for i in 15..ix.accounts.len() {
            bin_arrays.push(create_account_meta(parsed_accounts, ix, i)?);
        }

        Ok(MeteoraDlmmInputAccounts {
            lb_pair: create_account_meta(parsed_accounts, ix, 0)?,
            bin_array_bitmap_extension: create_account_meta(parsed_accounts, ix, 1)?,
            reverse_x: create_account_meta(parsed_accounts, ix, 2)?,
            reverse_y: create_account_meta(parsed_accounts, ix, 3)?,
            user_token_in: create_account_meta(parsed_accounts, ix, 4)?,
            user_token_out: create_account_meta(parsed_accounts, ix, 5)?,
            token_x_mint: create_account_meta(parsed_accounts, ix, 6)?,
            token_y_mint: create_account_meta(parsed_accounts, ix, 7)?,
            oracle: create_account_meta(parsed_accounts, ix, 8)?,
            host_fee_in: create_account_meta(parsed_accounts, ix, 9)?,
            user: create_account_meta(parsed_accounts, ix, 10)?,
            token_x_program: create_account_meta(parsed_accounts, ix, 11)?,
            token_y_program: create_account_meta(parsed_accounts, ix, 12)?,
            event_authority: create_account_meta(parsed_accounts, ix, 13)?,
            program: create_account_meta(parsed_accounts, ix, 14)?,
            bin_arrays,
        })
    }

    fn build_accounts(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: MeteoraDlmmPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        _output_amount: Option<u64>,
    ) -> Result<MeteoraDlmmInputAccounts> {
        use crate::arb::constant::pool_owner::METEORA_DLMM_PROGRAM;
        use crate::arb::pool::meteora_dlmm::bin_array;
        use crate::constants::addresses::TokenProgram;
        use crate::constants::helpers::{ToAccountMeta, ToPubkey};
        use spl_associated_token_account::get_associated_token_address_with_program_id;

        // Determine swap direction (swap_for_y means swapping X for Y)
        let swap_for_y = input_mint == &pool_data.token_x_mint;

        // Use tokio's block_on to call the async function from sync context
        // In production, consider making build_accounts async
        let runtime = tokio::runtime::Runtime::new()?;
        let bin_arrays = runtime.block_on(async {
            // Estimate number of bin arrays based on swap amount
            // It's safer to include more arrays - unused ones are ignored
            let num_arrays = bin_array::estimate_num_bin_arrays(input_amount.unwrap_or(0));
            bin_array::calculate_bin_arrays_for_swap(&pool_data, pool, swap_for_y, num_arrays)
                .await
        })?;

        // Determine token programs (assuming SPL token for now)
        let token_x_program = TokenProgram::SPL_TOKEN.to_program();
        let token_y_program = TokenProgram::SPL_TOKEN.to_program();

        // Get ATAs for input and output
        let user_token_in = get_associated_token_address_with_program_id(
            payer,
            input_mint,
            &token_x_program.pubkey,
        );
        let user_token_out = get_associated_token_address_with_program_id(
            payer,
            output_mint,
            &token_y_program.pubkey,
        );

        // Event authority is a PDA
        let event_authority =
            Pubkey::find_program_address(&[b"__event_authority"], &*METEORA_DLMM_PROGRAM).0;

        Ok(MeteoraDlmmInputAccounts {
            lb_pair: pool.to_writable(),
            bin_array_bitmap_extension: METEORA_DLMM_PROGRAM.to_program(),
            reverse_x: pool_data.reserve_x.to_writable(),
            reverse_y: pool_data.reserve_y.to_writable(),
            user_token_in: user_token_in.to_writable(),
            user_token_out: user_token_out.to_writable(),
            token_x_mint: pool_data.token_x_mint.to_readonly(),
            token_y_mint: pool_data.token_y_mint.to_readonly(),
            oracle: pool_data.oracle.to_writable(),
            host_fee_in: METEORA_DLMM_PROGRAM.to_program(),
            user: payer.to_signer(),
            token_x_program,
            token_y_program,
            event_authority: event_authority.to_readonly(),
            program: METEORA_DLMM_PROGRAM.to_program(),
            bin_arrays: bin_arrays.iter().map(|a| a.to_writable()).collect(),
        })
    }

    fn to_list(&self) -> Vec<&AccountMeta> {
        concat(vec![
            vec![
                &self.lb_pair,
                &self.bin_array_bitmap_extension,
                &self.reverse_x,
                &self.reverse_y,
                &self.user_token_in,
                &self.user_token_out,
                &self.token_x_mint,
                &self.token_y_mint,
                &self.oracle,
                &self.host_fee_in,
                &self.user,
                &self.token_x_program,
                &self.token_y_program,
                &self.event_authority,
                &self.program,
            ],
            self.bin_arrays.iter().collect(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use crate::arb::constant::pool_owner::METEORA_DLMM_PROGRAM;
    use crate::arb::pool::interface::{InputAccountUtil, PoolDataLoader};
    use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
    use crate::arb::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
    use crate::arb::tx::ix::is_meteora_dlmm_swap;
    use crate::arb::tx::tx_parser::{extract_mev_instruction, get_tx_by_sig};
    use crate::constants::addresses::TokenProgram;
    use crate::constants::helpers::{ToAccountMeta, ToPubkey};
    use crate::test::test_utils::get_test_rpc_client;
    use anyhow::Result;
    use base64::engine::general_purpose;
    use base64::Engine;

    static PAYER: &str = "4UX2dsCbqCm475cM2VvbEs6CmgoAhwP9CnwRT6WxmYA5";
    fn expected_result() -> MeteoraDlmmInputAccounts {
        MeteoraDlmmInputAccounts {
            lb_pair: "FrQ9w1xEiypynrBt2qWPWLcFQ1Ht1LLfyrcWLKfzcXcs".to_writable(),
            bin_array_bitmap_extension: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_program(),
            reverse_x: "BkGhDjHS9F6DtZhhhuzBWDjGpRwQUFybj6CXTUt7JR4k".to_writable(),
            reverse_y: "7ixpeA92jWuwFaPbYPDBFJWRaAtA621QntqZsxmqTaL3".to_writable(),
            user_token_in: "Aiaz92F1keKEfJkfWjvRrp34D8Wh4dGRbrSDuHzV289s".to_writable(),
            user_token_out: "AaeZVRToQvmEBuU9EjypuYs3GyVZSZhKpCV2opPa4Biy".to_writable(),
            token_x_mint: "G1DXVVmqJs8Ei79QbK41dpgk2WtXSGqLtx9of7o8BAGS".to_readonly(),
            token_y_mint: "So11111111111111111111111111111111111111112".to_readonly(),
            oracle: "3JZiurfBXWDEbqSA4fJD1FCq3iJyNaxVKJpRi7PkeiM2".to_writable(),
            host_fee_in: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_program(),
            user: PAYER.to_signer(),
            token_x_program: TokenProgram::SPL_TOKEN.to_program(),
            token_y_program: TokenProgram::SPL_TOKEN.to_program(),
            event_authority: "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6".to_readonly(),
            program: METEORA_DLMM_PROGRAM.to_program(),
            bin_arrays: vec![
                "B2a1aWZxBSm1qWwEccceUzrkL76Ab9UYEesgmqdvv449".to_writable(),
                "99NKxVHj9vRRQQQAYArBiB2L8wxPC9SEqKPdijYA5TXT".to_writable(),
                "CmiWUQbev3JUSDGPvunF3DZDYBTJYAKUdUZpp9WkeJLh".to_writable(),
            ],
        }
    }

    fn pool_data() -> Result<MeteoraDlmmPoolData> {
        let onchain_data = "IQsxYrVlsQ0gTiwBsASIE0wdAADwSQIA8e7//w8RAAD0AQAAAAAAAMCbAACQJgAA5vr//wAAAAClDZ9oAAAAAAAAAAAAAAAA/2QAA+P6//9kAAAAIE4AAN7td33Mx98jGjCLKLPX2kDV8j7rMt7KXJNLu8+1PzQDBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAGfqpaz2viTuSR0cGGJ6JPCjvoqMsjwVK2+jx2ZT37v4WPmDVuDQWmWg74CrJMkg/t8NH9+nmZ7YYsJqh7B7zFEahonY9KSCgA/bo8XBQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIjc/RxuLXAAEl0fOWAcZW485SbwYtahs6aRb/AWoOoUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+AMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHPDNE1wVa0QDjU1aPWTZ0hpdJm2mL/omaPlnDZ70/xBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADVyNG4bWBXDqGWKcspzbTA32Tdih7mqOgVxYAfLC37MAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
        let data = general_purpose::STANDARD.decode(onchain_data)?;
        MeteoraDlmmPoolData::load_data(&data)
    }
    #[test]
    fn test_build_accounts() {
        let result = MeteoraDlmmInputAccounts::build_accounts(
            &PAYER.to_pubkey(),
            &"FrQ9w1xEiypynrBt2qWPWLcFQ1Ht1LLfyrcWLKfzcXcs".to_pubkey(),
            pool_data().unwrap(),
            &"G1DXVVmqJs8Ei79QbK41dpgk2WtXSGqLtx9of7o8BAGS".to_pubkey(),
            &"So11111111111111111111111111111111111111112".to_pubkey(),
            Some(543235989680078),
            Some(0),
        )
        .unwrap();
        assert_eq!(result, expected_result())
    }

    #[test]
    fn test_restore_from() {
        let tx = "57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP";
        let tx = get_tx_by_sig(&get_test_rpc_client(), tx).unwrap();
        let (_, inner) = extract_mev_instruction(&tx).unwrap();
        let dlmm_swap = inner
            .instructions
            .iter()
            .filter_map(is_meteora_dlmm_swap)
            .next()
            .unwrap();

        let result = MeteoraDlmmInputAccounts::restore_from(dlmm_swap, &tx).unwrap();
        assert_eq!(result, expected_result())
    }
}
