use crate::convention::chain::instruction::Instruction;
use crate::convention::chain::Transaction;
use crate::dex::interface::PoolDataLoader;
use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::global::enums::direction::TradeDirection;
use crate::util::alias::AResult;
use crate::util::solana::pda::ata;
use crate::util::traits::account_meta::ToAccountMeta;
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

    async fn build_accounts_no_matter_direction_size(
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
        .await
    }

    async fn build_accounts_with_direction_and_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &MeteoraDammV2PoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> Result<MeteoraDammV2InputAccount> {
        use crate::global::constant::pool_program::PoolProgram;
        use crate::global::constant::token_program::TokenProgram;
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
            referral_token_program: PoolProgram::METEORA_DAMM_V2.to_program(),
            event_authority,
            meteora_program: PoolProgram::METEORA_DAMM_V2.to_program(),
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
#[path = "input_account_test.rs"]
mod tests;
