use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::Transaction;
use crate::arb::convention::pool::interface::{InputAccountUtil, PoolDataLoader, TradeDirection};
use crate::arb::convention::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use anyhow::Result;
use itertools::concat;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

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

impl InputAccountUtil<MeteoraDlmmInputAccounts, MeteoraDlmmPoolData> for MeteoraDlmmInputAccounts {
    fn restore_from(ix: &Instruction, tx: &Transaction) -> Result<MeteoraDlmmInputAccounts> {
        if ix.accounts.len() < 15 {
            return Err(anyhow::anyhow!(
                "Invalid number of accounts for Meteora DLMM swap: expected at least 15, got {}",
                ix.accounts.len()
            ));
        }

        // Extract bin arrays (all accounts after index 14)
        let mut bin_arrays = Vec::new();
        for i in 15..ix.accounts.len() {
            bin_arrays.push(ix.account_at(i)?);
        }

        Ok(MeteoraDlmmInputAccounts {
            lb_pair: ix.account_at(0)?,
            bin_array_bitmap_extension: ix.account_at(1)?,
            reverse_x: ix.account_at(2)?,
            reverse_y: ix.account_at(3)?,
            user_token_in: ix.account_at(4)?,
            user_token_out: ix.account_at(5)?,
            token_x_mint: ix.account_at(6)?,
            token_y_mint: ix.account_at(7)?,
            oracle: ix.account_at(8)?,
            host_fee_in: ix.account_at(9)?,
            user: ix.account_at(10)?,
            token_x_program: ix.account_at(11)?,
            token_y_program: ix.account_at(12)?,
            event_authority: ix.account_at(13)?,
            program: ix.account_at(14)?,
            bin_arrays,
        })
    }

    fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &MeteoraDlmmPoolData,
    ) -> Result<MeteoraDlmmInputAccounts> {
        use crate::arb::convention::pool::meteora_dlmm::bin_array;
        use crate::arb::global::constant::pool_program::PoolPrograms;
        use crate::arb::global::constant::token_program::TokenProgram;
        use crate::constants::helpers::ToAccountMeta;
        use spl_associated_token_account::get_associated_token_address_with_program_id;

        let token_x_mint = &pool_data.token_x_mint;
        let token_y_mint = &pool_data.token_y_mint;

        let bin_arrays =
            bin_array::generate_bin_arrays_for_swap(pool_data.active_id, pool, true, 3);

        let token_x_program = TokenProgram::SPL_TOKEN.to_program();
        let token_y_program = TokenProgram::SPL_TOKEN.to_program();

        let user_token_x = get_associated_token_address_with_program_id(
            payer,
            token_x_mint,
            &token_x_program.pubkey,
        );
        let user_token_y = get_associated_token_address_with_program_id(
            payer,
            token_y_mint,
            &token_y_program.pubkey,
        );

        let event_authority =
            Pubkey::find_program_address(&[b"__event_authority"], &PoolPrograms::METEORA_DLMM).0;

        Ok(MeteoraDlmmInputAccounts {
            lb_pair: pool.to_writable(),
            bin_array_bitmap_extension: PoolPrograms::METEORA_DLMM.to_program(),
            reverse_x: pool_data.reserve_x.to_writable(),
            reverse_y: pool_data.reserve_y.to_writable(),
            user_token_in: user_token_x.to_writable(),
            user_token_out: user_token_y.to_writable(),
            token_x_mint: pool_data.token_x_mint.to_readonly(),
            token_y_mint: pool_data.token_y_mint.to_readonly(),
            oracle: pool_data.oracle.to_writable(),
            host_fee_in: PoolPrograms::METEORA_DLMM.to_program(),
            user: payer.to_signer(),
            token_x_program,
            token_y_program,
            event_authority: event_authority.to_readonly(),
            program: PoolPrograms::METEORA_DLMM.to_program(),
            bin_arrays: bin_arrays.iter().map(|a| a.to_writable()).collect(),
        })
    }

    fn build_accounts_with_direction_and_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &MeteoraDlmmPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        _output_amount: Option<u64>,
    ) -> Result<MeteoraDlmmInputAccounts> {
        use crate::arb::convention::pool::meteora_dlmm::bin_array;
        use crate::arb::global::constant::pool_program::PoolPrograms;
        use crate::arb::global::constant::token_program::TokenProgram;
        use crate::constants::helpers::ToAccountMeta;
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
            bin_array::calculate_bin_arrays_for_swap(&pool_data, pool, swap_for_y, num_arrays).await
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
            Pubkey::find_program_address(&[b"__event_authority"], &PoolPrograms::METEORA_DLMM).0;

        Ok(MeteoraDlmmInputAccounts {
            lb_pair: pool.to_writable(),
            bin_array_bitmap_extension: PoolPrograms::METEORA_DLMM.to_program(),
            reverse_x: pool_data.reserve_x.to_writable(),
            reverse_y: pool_data.reserve_y.to_writable(),
            user_token_in: user_token_in.to_writable(),
            user_token_out: user_token_out.to_writable(),
            token_x_mint: pool_data.token_x_mint.to_readonly(),
            token_y_mint: pool_data.token_y_mint.to_readonly(),
            oracle: pool_data.oracle.to_writable(),
            host_fee_in: PoolPrograms::METEORA_DLMM.to_program(),
            user: payer.to_signer(),
            token_x_program,
            token_y_program,
            event_authority: event_authority.to_readonly(),
            program: PoolPrograms::METEORA_DLMM.to_program(),
            bin_arrays: bin_arrays.iter().map(|a| a.to_writable()).collect(),
        })
    }

    fn get_trade_direction(self) -> TradeDirection {
        use spl_associated_token_account::get_associated_token_address_with_program_id;

        let user = self.user.pubkey;
        let token_x_program = self.token_x_program.pubkey;
        let token_y_program = self.token_y_program.pubkey;

        let expected_ata_x = get_associated_token_address_with_program_id(
            &user,
            &self.token_x_mint.pubkey,
            &token_x_program,
        );

        let expected_ata_y = get_associated_token_address_with_program_id(
            &user,
            &self.token_y_mint.pubkey,
            &token_y_program,
        );

        if self.user_token_in.pubkey == expected_ata_x {
            TradeDirection {
                from: self.token_x_mint.pubkey,
                to: self.token_y_mint.pubkey,
            }
        } else if self.user_token_in.pubkey == expected_ata_y {
            TradeDirection {
                from: self.token_y_mint.pubkey,
                to: self.token_x_mint.pubkey,
            }
        } else {
            panic!(
                "Invalid user_token_in: {} doesn't match expected ATA for token X {} or token Y {}",
                self.user_token_in.pubkey, expected_ata_x, expected_ata_y
            )
        }
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
    use crate::arb::convention::pool::interface::{InputAccountUtil, PoolDataLoader};
    use crate::arb::convention::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
    use crate::arb::convention::pool::meteora_dlmm::input_data::is_meteora_dlmm_swap;
    use crate::arb::convention::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
    use crate::arb::global::constant::pool_program::PoolPrograms;
    use crate::arb::global::constant::token_program::TokenProgram;
    use crate::arb::global::state::rpc::fetch_tx_sync;
    use crate::arb::program::mev_bot::ix::extract_mev_instruction;
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
            program: PoolPrograms::METEORA_DLMM.to_program(),
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
        let result = MeteoraDlmmInputAccounts::build_accounts_with_direction_and_size(
            &PAYER.to_pubkey(),
            &"FrQ9w1xEiypynrBt2qWPWLcFQ1Ht1LLfyrcWLKfzcXcs".to_pubkey(),
            &pool_data().unwrap(),
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
        let tx_sig = "57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP";
        let tx = fetch_tx_sync(&get_test_rpc_client(), tx_sig).unwrap();
        let (_, inner) = extract_mev_instruction(&tx).unwrap();
        let dlmm_swap = inner
            .instructions
            .iter()
            .find(|ix| is_meteora_dlmm_swap(&ix.data))
            .unwrap();

        let result = MeteoraDlmmInputAccounts::restore_from(dlmm_swap, &tx).unwrap();
        assert_eq!(result, expected_result())
    }
}
