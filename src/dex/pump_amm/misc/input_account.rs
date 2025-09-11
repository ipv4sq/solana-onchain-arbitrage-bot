use crate::convention::chain::instruction::Instruction;
use crate::convention::chain::Transaction;
use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::interface::PoolDataLoader;
use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::pump_amm::misc::address_seed;
use crate::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::global::constant::mint::Mints;
use crate::global::constant::pool_program::PoolProgram;
use crate::global::constant::token_program::{
    SystemProgram, TokenProgram, ASSOCIATED_TOKEN_ACCOUNT_PROGRAM,
};
use crate::global::enums::direction::TradeDirection;
use crate::util::alias::AResult;
use crate::util::solana::pda::ata;
use crate::util::traits::account_meta::ToAccountMeta;
use crate::util::traits::pubkey::ToPubkey;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey;
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
    pub fee_config: AccountMeta,
    pub fee_program: AccountMeta,
}

impl PumpAmmInputAccounts {
    pub async fn build_accounts_with_direction(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &PumpAmmPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> AResult<PumpAmmInputAccounts> {
        let base_mint = MintRecordRepository::get_mint_or_err(&pool_data.base_mint).await?;
        let quote_mint = MintRecordRepository::get_mint_or_err(&pool_data.quote_mint).await?;
        let coin_creator_vault_authority =
            address_seed::get_coin_creator_vault_authority(&pool_data.coin_creator);

        let pump_fee_recipient = "JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU".to_pubkey();
        let fee_program = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
        let pump_amm_program = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

        let (pump_amm_fee_config_pda, _) = Pubkey::find_program_address(
            &[b"fee_config", &pump_amm_program.as_ref()],
            &fee_program,
        );

        let fee = if pool_data.mint_pair().contains(&Mints::WSOL) {
            Mints::WSOL
        } else {
            Mints::USDC
        };

        // Volume accumulators are required for quote->base (buying pump tokens)
        // Not required for base->quote (selling pump tokens)
        let is_quote_to_base = *input_mint == pool_data.quote_mint;
        let (global_volume_accumulator, user_volume_accumulator) = if is_quote_to_base {
            (
                Some(address_seed::get_global_volume_accumulator().to_writable()),
                Some(address_seed::get_user_volume_accumulator(payer).to_writable()),
            )
        } else {
            (None, None)
        };

        Ok(PumpAmmInputAccounts {
            pool: pool.to_readonly(), // fuck, sometimes it is readonly, related to buy/sell
            user: payer.to_signer(),
            global_config: "ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw".to_readonly(),
            base_mint: pool_data.base_mint.to_readonly(),
            quote_mint: pool_data.quote_mint.to_readonly(),
            user_base_token_account: ata(payer, &base_mint.address.0, &base_mint.program.0)
                .to_writable(),
            user_quote_token_account: ata(payer, &quote_mint.address.0, &quote_mint.program.0)
                .to_writable(),
            pool_base_token_account: ata(pool, &base_mint.address.0, &base_mint.program.0)
                .to_writable(),
            pool_quote_token_account: ata(pool, &quote_mint.address.0, &quote_mint.program.0)
                .to_writable(),
            protocol_fee_recipient: pump_fee_recipient.to_readonly(),
            protocol_fee_recipient_token_account: ata(
                &pump_fee_recipient,
                &fee,
                &TokenProgram::SPL_TOKEN,
            )
            .to_writable(),
            base_token_program: base_mint.program.0.to_program(),
            quote_token_program: quote_mint.program.0.to_program(),
            system_program: SystemProgram.to_program(),
            associated_token_program: ASSOCIATED_TOKEN_ACCOUNT_PROGRAM.to_program(),
            event_authority: "GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR".to_program(),
            program: PoolProgram::PUMP_AMM.to_program(),
            coin_creator_vault_ata: ata(
                &coin_creator_vault_authority,
                &quote_mint.address.0,
                &quote_mint.program.0,
            )
            .to_writable(),
            coin_creator_vault_authority: coin_creator_vault_authority.to_readonly(),
            global_volume_accumulator,
            user_volume_accumulator,
            fee_config: pump_amm_fee_config_pda.to_program(),
            fee_program: "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ".to_program(),
        })
    }
}

impl InputAccountUtil<PumpAmmInputAccounts, PumpAmmPoolData> for PumpAmmInputAccounts {
    fn restore_from(ix: &Instruction, tx: &Transaction) -> AResult<PumpAmmInputAccounts> {
        if ix.program_id != PoolProgram::PUMP_AMM {
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
            global_volume_accumulator: ix.account_at(19).ok(),
            user_volume_accumulator: ix.account_at(20).ok(),
            fee_config: ix.account_at(21)?,
            fee_program: ix.account_at(22)?,
        })
    }

    async fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &PumpAmmPoolData,
    ) -> AResult<PumpAmmInputAccounts> {
        let base_mint = MintRecordRepository::get_mint_or_err(&pool_data.base_mint).await?;
        let quote_mint = MintRecordRepository::get_mint_or_err(&pool_data.quote_mint).await?;
        let coin_creator_vault_authority =
            address_seed::get_coin_creator_vault_authority(&pool_data.coin_creator);

        let pump_fee_recipient = "JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU".to_pubkey();
        let fee_program = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
        let pump_amm_program = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

        let (pump_amm_fee_config_pda, _) = Pubkey::find_program_address(
            &[b"fee_config", &pump_amm_program.as_ref()],
            &fee_program,
        );

        let fee = if pool_data.mint_pair().contains(&Mints::WSOL) {
            Mints::WSOL
        } else {
            Mints::USDC
        };

        Ok(PumpAmmInputAccounts {
            pool: pool.to_readonly(), // fuck, sometimes it is readonly, related to buy/sell
            user: payer.to_signer(),
            global_config: "ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw".to_readonly(),
            base_mint: pool_data.base_mint.to_readonly(),
            quote_mint: pool_data.quote_mint.to_readonly(),
            user_base_token_account: ata(payer, &base_mint.address.0, &base_mint.program.0)
                .to_writable(),
            user_quote_token_account: ata(payer, &quote_mint.address.0, &quote_mint.program.0)
                .to_writable(),
            pool_base_token_account: ata(pool, &base_mint.address.0, &base_mint.program.0)
                .to_writable(),
            pool_quote_token_account: ata(pool, &quote_mint.address.0, &quote_mint.program.0)
                .to_writable(),
            protocol_fee_recipient: pump_fee_recipient.to_readonly(),
            protocol_fee_recipient_token_account: ata(
                &pump_fee_recipient,
                &fee,
                &TokenProgram::SPL_TOKEN,
            )
            .to_writable(),
            base_token_program: base_mint.program.0.to_program(),
            quote_token_program: quote_mint.program.0.to_program(),
            system_program: SystemProgram.to_program(),
            associated_token_program: ASSOCIATED_TOKEN_ACCOUNT_PROGRAM.to_program(),
            event_authority: "GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR".to_program(),
            program: PoolProgram::PUMP_AMM.to_program(),
            coin_creator_vault_ata: ata(
                &coin_creator_vault_authority,
                &quote_mint.address.0,
                &quote_mint.program.0,
            )
            .to_writable(),
            coin_creator_vault_authority: coin_creator_vault_authority.to_readonly(),
            global_volume_accumulator: Some(
                address_seed::get_global_volume_accumulator().to_writable(),
            ),
            user_volume_accumulator: Some(
                address_seed::get_user_volume_accumulator(payer).to_writable(),
            ),
            fee_config: pump_amm_fee_config_pda.to_program(),
            fee_program: "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ".to_program(),
        })
    }

    async fn build_accounts_with_direction_and_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &PumpAmmPoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> anyhow::Result<PumpAmmInputAccounts> {
        Self::build_accounts_no_matter_direction_size(payer, pool, pool_data).await
    }

    fn get_trade_direction(self) -> AResult<TradeDirection> {
        todo!()
    }

    fn to_list(&self) -> Vec<&AccountMeta> {
        let mut accounts = vec![
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

        // Add optional volume accumulators if present
        if let Some(ref global_volume) = self.global_volume_accumulator {
            accounts.push(global_volume);
        }
        if let Some(ref user_volume) = self.user_volume_accumulator {
            accounts.push(user_volume);
        }

        // Add fee config PDAs at the end
        accounts.push(&self.fee_config);
        accounts.push(&self.fee_program);

        accounts
    }
}

#[cfg(test)]
mod tests {}
