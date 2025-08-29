use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::Transaction;
use crate::arb::convention::pool::interface::{InputAccountUtil, TradeDirection};
use crate::arb::convention::pool::pump_amm::pool_data::PumpAmmPoolData;
use crate::arb::global::constant::pool_program::PoolPrograms;
use crate::arb::util::alias::AResult;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, PartialEq)]
pub struct PumpAmmInputAccounts {
    pub pool: AccountMeta,
    pub user: AccountMeta,
    pub global_config: AccountMeta,
    pub base_mint: AccountMeta,
    pub quote_mint: AccountMeta,
    pub user_base_token_account: AccountMeta,
    pub user_quote_token_account: AccountMeta,
    pub pool_base_token_account: AccountMeta,
    pub pool_quote_token_account: AccountMeta,
    pub protocol_fee_recipient: AccountMeta,
    pub protocol_fee_recipient_token_account: AccountMeta,
    pub base_token_program: AccountMeta,
    pub quote_token_program: AccountMeta,
    pub system_program: AccountMeta,
    pub associated_token_program: AccountMeta,
    pub event_authority: AccountMeta,
    pub program: AccountMeta,
    pub coin_creator_vault_ata: AccountMeta,
    pub coin_creator_vault_authority: AccountMeta,
    pub global_volume_accumulator: Option<AccountMeta>,
    pub user_volume_accumulator: Option<AccountMeta>,
}

impl InputAccountUtil<PumpAmmInputAccounts, PumpAmmPoolData> for PumpAmmInputAccounts {
    fn restore_from(ix: &Instruction, tx: &Transaction) -> AResult<PumpAmmInputAccounts> {
        if ix.program_id != PoolPrograms::PUMP_AMM {
            Err(anyhow::anyhow!(
                "This is not a pump amm ix! {}",
                ix.program_id
            ))?;
        }
        Ok(PumpAmmInputAccounts {
            pool: ix.account_at(0)?,
            user: ix.account_at(1)?,
            global_config: ix.account_at(2)?,
            base_mint: ix.account_at(3)?,
            quote_mint: ix.account_at(4)?,
            user_base_token_account: ix.account_at(5)?,
            user_quote_token_account: ix.account_at(6)?,
            pool_base_token_account: ix.account_at(7)?,
            pool_quote_token_account: ix.account_at(8)?,
            protocol_fee_recipient: ix.account_at(9)?,
            protocol_fee_recipient_token_account: ix.account_at(10)?,
            base_token_program: ix.account_at(11)?,
            quote_token_program: ix.account_at(12)?,
            system_program: ix.account_at(13)?,
            associated_token_program: ix.account_at(14)?,
            event_authority: ix.account_at(15)?,
            program: ix.account_at(16)?,
            coin_creator_vault_ata: ix.account_at(17)?,
            coin_creator_vault_authority: ix.account_at(18)?,
        })
    }

    fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &PumpAmmPoolData,
    ) -> anyhow::Result<PumpAmmInputAccounts> {
        todo!()
    }

    fn build_accounts_with_direction_and_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &PumpAmmPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> anyhow::Result<PumpAmmInputAccounts> {
        Self::build_accounts_no_matter_direction_size(payer, pool, pool_data)
    }

    fn get_trade_direction(self) -> AResult<TradeDirection> {
        todo!()
    }

    fn to_list(&self) -> Vec<&AccountMeta> {
        return vec![
            &self.pool,
            &self.user,
            &self.global_config,
            &self.base_mint,
            &self.quote_mint,
            &self.user_base_token_account,
            &self.user_quote_token_account,
            &self.pool_base_token_account,
            &self.pool_quote_token_account,
            &self.protocol_fee_recipient,
            &self.protocol_fee_recipient_token_account,
            &self.base_token_program,
            &self.quote_token_program,
            &self.system_program,
            &self.associated_token_program,
            &self.event_authority,
            &self.program,
            &self.coin_creator_vault_ata,
            &self.coin_creator_vault_authority,
        ];
    }
}
