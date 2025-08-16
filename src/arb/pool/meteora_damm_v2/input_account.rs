use crate::arb::pool::interface::InputAccountUtil;
use crate::arb::pool::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::constants::helpers::ToAccountMeta;
use anyhow::Result;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiPartiallyDecodedInstruction,
};
use crate::arb::chain::account::{create_account_meta, get_parsed_accounts};

#[derive(Debug, PartialEq)]
pub struct MeteoraDammV2InputAccount {
    pub pool_authority: AccountMeta,
    pub pool: AccountMeta,
    pub input_token_account: AccountMeta,
    pub output_token_account: AccountMeta,
    pub token_a_vault: AccountMeta,
    pub token_b_vault: AccountMeta,
    pub token_a_mint: AccountMeta,
    pub token_b_mint: AccountMeta,
    pub payer: AccountMeta,
    pub token_a_program: AccountMeta,
    pub token_b_program: AccountMeta,
    // Solscan shows this as account, but the actual address is a program
    // https://solscan.io/tx/57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP
    pub referral_token_program: AccountMeta,
    pub event_authority: AccountMeta,
    pub meteora_program: AccountMeta,
}

impl InputAccountUtil<MeteoraDammV2InputAccount, MeteoraDammV2PoolData>
    for MeteoraDammV2InputAccount
{
    fn restore_from(
        ix: &UiPartiallyDecodedInstruction,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<MeteoraDammV2InputAccount> {
        if ix.accounts.len() < 14 {
            return Err(anyhow::anyhow!(
                "Invalid number of accounts for Meteora DAMM V2 swap: expected at least 14, got {}",
                ix.accounts.len()
            ));
        }

        let parsed_accounts = get_parsed_accounts(tx)?;

        Ok(MeteoraDammV2InputAccount {
            pool_authority: create_account_meta(parsed_accounts, ix, 0)?,
            pool: create_account_meta(parsed_accounts, ix, 1)?,
            input_token_account: create_account_meta(parsed_accounts, ix, 2)?,
            output_token_account: create_account_meta(parsed_accounts, ix, 3)?,
            token_a_vault: create_account_meta(parsed_accounts, ix, 4)?,
            token_b_vault: create_account_meta(parsed_accounts, ix, 5)?,
            token_a_mint: create_account_meta(parsed_accounts, ix, 6)?,
            token_b_mint: create_account_meta(parsed_accounts, ix, 7)?,
            payer: create_account_meta(parsed_accounts, ix, 8)?,
            token_a_program: create_account_meta(parsed_accounts, ix, 9)?,
            token_b_program: create_account_meta(parsed_accounts, ix, 10)?,
            referral_token_program: create_account_meta(parsed_accounts, ix, 11)?,
            event_authority: create_account_meta(parsed_accounts, ix, 12)?,
            meteora_program: create_account_meta(parsed_accounts, ix, 13)?,
        })
    }

    fn build_accounts(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: MeteoraDammV2PoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> Result<MeteoraDammV2InputAccount> {
        use crate::arb::constant::pool_owner::PoolOwnerPrograms;
        use crate::constants::addresses::TokenProgram;
        use spl_associated_token_account::get_associated_token_address_with_program_id;

        // Pool authority is a fixed PDA for DAMM V2
        let pool_authority = "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC".to_readonly();

        // Event authority is also fixed for DAMM V2
        let event_authority = "3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet".to_readonly();

        // Determine which token is A and which is B based on pool data
        let (token_a_mint, token_b_mint) = (pool_data.token_a_mint, pool_data.token_b_mint);
        let (token_a_vault, token_b_vault) = (pool_data.token_a_vault, pool_data.token_b_vault);

        // Determine input/output vaults based on mint order
        let (input_vault, output_vault) = if *input_mint == token_a_mint {
            (token_a_vault, token_b_vault)
        } else if *input_mint == token_b_mint {
            (token_b_vault, token_a_vault)
        } else {
            return Err(anyhow::anyhow!("Input mint doesn't match pool mints"));
        };

        // For now, assume SPL token program for both (could be enhanced to check mint)
        let token_a_program = TokenProgram::SPL_TOKEN.to_program();
        let token_b_program = TokenProgram::SPL_TOKEN.to_program();

        // Get ATAs for input and output
        let input_token_account = get_associated_token_address_with_program_id(
            payer,
            input_mint,
            &token_a_program.pubkey,
        );
        let output_token_account = get_associated_token_address_with_program_id(
            payer,
            output_mint,
            &token_b_program.pubkey,
        );

        Ok(MeteoraDammV2InputAccount {
            pool_authority,
            pool: pool.to_writable(),
            input_token_account: input_token_account.to_writable(),
            output_token_account: output_token_account.to_writable(),
            token_a_vault: token_a_vault.to_writable(),
            token_b_vault: token_b_vault.to_writable(),
            token_a_mint: token_a_mint.to_readonly(),
            token_b_mint: token_b_mint.to_readonly(),
            payer: payer.to_signer(),
            token_a_program,
            token_b_program,
            referral_token_program: PoolOwnerPrograms::METEORA_DAMM_V2.to_program(),
            event_authority,
            meteora_program: PoolOwnerPrograms::METEORA_DAMM_V2.to_program(),
        })
    }

    fn to_list(&self) -> Vec<&AccountMeta> {
        vec![
            &self.pool_authority,
            &self.pool,
            &self.input_token_account,
            &self.output_token_account,
            &self.token_a_vault,
            &self.token_b_vault,
            &self.token_a_mint,
            &self.token_b_mint,
            &self.payer,
            &self.token_a_program,
            &self.token_b_program,
            &self.referral_token_program,
            &self.event_authority,
            &self.meteora_program,
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::arb::constant::pool_owner::PoolOwnerPrograms;
    use crate::arb::pool::interface::InputAccountUtil;
    use crate::arb::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
    use crate::arb::pool::meteora_damm_v2::pool_data::test::load_pool_data;
    use crate::constants::addresses::TokenProgram;
    use crate::constants::helpers::{ToAccountMeta, ToPubkey};
    use crate::test::test_utils::get_test_rpc_client;
    use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
    use crate::arb::chain::ix::is_meteora_damm_v2_swap;
    use crate::arb::chain::tx::fetch_tx_sync;
    use crate::arb::chain::tx_parser::extract_mev_instruction;

    // https://solscan.io/tx/57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP
    const TX: &str =
        "57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP";

    fn get_tx() -> EncodedConfirmedTransactionWithStatusMeta {
        let client = get_test_rpc_client();
        fetch_tx_sync(&client, TX).unwrap()
    }

    fn expected_account() -> MeteoraDammV2InputAccount {
        MeteoraDammV2InputAccount {
            pool_authority: "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC".to_readonly(),
            pool: "6CXXieC355gteamwofSzJn8DiyrbKyYyXc3eBKmB81CF".to_writable(),
            input_token_account: "AaeZVRToQvmEBuU9EjypuYs3GyVZSZhKpCV2opPa4Biy".to_writable(),
            output_token_account: "Aiaz92F1keKEfJkfWjvRrp34D8Wh4dGRbrSDuHzV289s".to_writable(),
            token_a_vault: "9B3KPhHyDhUmNvjY2vk6JYs3vfUgPTzB9u1fWYsfK1s5".to_writable(),
            token_b_vault: "wAx8Her71ffN9hNyh5nj6WR7m56tAGrkajNiEdoGy4G".to_writable(),
            token_a_mint: "G1DXVVmqJs8Ei79QbK41dpgk2WtXSGqLtx9of7o8BAGS".to_readonly(),
            token_b_mint: "So11111111111111111111111111111111111111112".to_readonly(),
            payer: "4UX2dsCbqCm475cM2VvbEs6CmgoAhwP9CnwRT6WxmYA5".to_signer(),
            token_a_program: TokenProgram::SPL_TOKEN.to_program(),
            token_b_program: TokenProgram::SPL_TOKEN.to_program(),
            referral_token_program: PoolOwnerPrograms::METEORA_DAMM_V2.to_program(),
            event_authority: "3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet".to_readonly(),
            meteora_program: PoolOwnerPrograms::METEORA_DAMM_V2.to_program(),
        }
    }

    #[test]
    fn test_restore_from() {
        let tx = get_tx();

        let (_, inner_ixs) = extract_mev_instruction(&tx).unwrap();

        // Find the actual swap instruction (the one with 14 accounts)
        let damm_v2_ix = inner_ixs
            .instructions
            .iter()
            .filter_map(is_meteora_damm_v2_swap)
            .next()
            .unwrap();

        let result = MeteoraDammV2InputAccount::restore_from(damm_v2_ix, &tx).unwrap();
        let expected = expected_account();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_build_accounts() {
        let pool = "6CXXieC355gteamwofSzJn8DiyrbKyYyXc3eBKmB81CF".to_pubkey();
        let pool_data = load_pool_data();
        let input_mint = "So11111111111111111111111111111111111111112".to_pubkey();
        let output_mint = "G1DXVVmqJs8Ei79QbK41dpgk2WtXSGqLtx9of7o8BAGS".to_pubkey();

        let result = MeteoraDammV2InputAccount::build_accounts(
            &"4UX2dsCbqCm475cM2VvbEs6CmgoAhwP9CnwRT6WxmYA5".to_pubkey(),
            &pool,
            pool_data,
            &input_mint,
            &output_mint,
            Some(3226352439),
            Some(0),
        )
        .unwrap();

        let expected = expected_account();
        assert_eq!(expected, result);
    }

}
