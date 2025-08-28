use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::Transaction;
use crate::arb::convention::pool::interface::{InputAccountUtil, PoolDataLoader, TradeDirection};
use crate::arb::convention::pool::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::arb::convention::pool::util::ata;
use crate::arb::util::alias::AResult;
use crate::arb::util::traits::account_meta::ToAccountMeta;
use anyhow::Result;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, PartialEq)]
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
    fn restore_from(ix: &Instruction, tx: &Transaction) -> Result<MeteoraDammV2InputAccount> {
        if ix.accounts.len() < 14 {
            return Err(anyhow::anyhow!(
                "Invalid number of accounts for Meteora DAMM V2 swap: expected at least 14, got {}",
                ix.accounts.len()
            ));
        }

        Ok(MeteoraDammV2InputAccount {
            pool_authority: ix.account_at(0)?,
            pool: ix.account_at(1)?,
            input_token_account: ix.account_at(2)?,
            output_token_account: ix.account_at(3)?,
            token_a_vault: ix.account_at(4)?,
            token_b_vault: ix.account_at(5)?,
            token_a_mint: ix.account_at(6)?,
            token_b_mint: ix.account_at(7)?,
            payer: ix.account_at(8)?,
            token_a_program: ix.account_at(9)?,
            token_b_program: ix.account_at(10)?,
            referral_token_program: ix.account_at(11)?,
            event_authority: ix.account_at(12)?,
            meteora_program: ix.account_at(13)?,
        })
    }

    fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &MeteoraDammV2PoolData,
    ) -> Result<MeteoraDammV2InputAccount> {
        Self::build_accounts_with_direction_and_size(
            payer,
            pool,
            pool_data,
            &pool_data.base_mint(),
            &pool_data.quote_mint(),
            None,
            None,
        )
    }

    fn build_accounts_with_direction_and_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &MeteoraDammV2PoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> Result<MeteoraDammV2InputAccount> {
        use crate::arb::global::constant::pool_program::PoolPrograms;
        use crate::arb::global::constant::token_program::TokenProgram;
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
        let input_token_account = ata(payer, input_mint, &token_a_program.pubkey);
        let output_token_account = ata(payer, output_mint, &token_b_program.pubkey);

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
            referral_token_program: PoolPrograms::METEORA_DAMM_V2.to_program(),
            event_authority,
            meteora_program: PoolPrograms::METEORA_DAMM_V2.to_program(),
        })
    }

    fn get_trade_direction(self) -> AResult<TradeDirection> {
        let payer = self.payer.pubkey;
        let token_a_program = self.token_a_program.pubkey;
        let token_b_program = self.token_b_program.pubkey;

        let expected_ata_a = ata(&payer, &self.token_a_mint.pubkey, &token_a_program);

        let expected_ata_b = ata(&payer, &self.token_b_mint.pubkey, &token_b_program);

        if self.input_token_account.pubkey == expected_ata_a {
            Ok(TradeDirection {
                from: self.token_a_mint.pubkey,
                to: self.token_b_mint.pubkey,
            })
        } else if self.input_token_account.pubkey == expected_ata_b {
            Ok(TradeDirection {
                from: self.token_b_mint.pubkey,
                to: self.token_a_mint.pubkey,
            })
        } else {
            Err(anyhow::anyhow!(
                "Invalid input token account: {} doesn't match expected ATA for token A {} or token B {}",
                self.input_token_account.pubkey,
                expected_ata_a,
                expected_ata_b
            ))
        }
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
    use crate::arb::convention::chain::Transaction;
    use crate::arb::convention::pool::interface::InputAccountUtil;
    use crate::arb::convention::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
    use crate::arb::convention::pool::meteora_damm_v2::input_data::is_meteora_damm_v2_swap;
    use crate::arb::convention::pool::meteora_damm_v2::pool_data::test::load_pool_data;
    use crate::arb::global::constant::pool_program::PoolPrograms;
    use crate::arb::global::constant::token_program::TokenProgram;
    use crate::arb::global::state::rpc::fetch_tx_sync;
    use crate::arb::program::mev_bot::ix::extract_mev_instruction;
    use crate::arb::util::traits::account_meta::ToAccountMeta;
    use crate::arb::util::traits::pubkey::ToPubkey;
    use crate::test::test_utils::get_test_rpc_client;

    // https://solscan.io/tx/57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP
    const TX: &str =
        "57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP";

    fn get_tx() -> Transaction {
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
            referral_token_program: PoolPrograms::METEORA_DAMM_V2.to_program(),
            event_authority: "3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet".to_readonly(),
            meteora_program: PoolPrograms::METEORA_DAMM_V2.to_program(),
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
            .find(|ix| is_meteora_damm_v2_swap(&ix.data))
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

        let result = MeteoraDammV2InputAccount::build_accounts_with_direction_and_size(
            &"4UX2dsCbqCm475cM2VvbEs6CmgoAhwP9CnwRT6WxmYA5".to_pubkey(),
            &pool,
            &pool_data,
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
