use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::Transaction;
use crate::arb::database::mint_record::repository::MintRecordRepository;
use crate::arb::dex::legacy_interface::InputAccountUtil;
use crate::arb::dex::pump_amm::misc::address_seed;
use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::constant::token_program::{
    SystemProgram, TokenProgram, ASSOCIATED_TOKEN_ACCOUNT_PROGRAM,
};
use crate::arb::global::enums::direction::TradeDirection;
use crate::arb::util::alias::AResult;
use crate::arb::util::solana::pda::ata;
use crate::arb::util::traits::account_meta::ToAccountMeta;
use crate::arb::util::traits::option::OptionExt;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::f;
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
        })
    }

    async fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &PumpAmmPoolData,
    ) -> AResult<PumpAmmInputAccounts> {
        let base_mint = MintRecordRepository::get_mint(&pool_data.base_mint)
            .await?
            .or_err(f!("Can't retrieve base mint {}", &pool_data.base_mint))?;
        let quote_mint = MintRecordRepository::get_mint(&pool_data.quote_mint)
            .await?
            .or_err(f!("Can't retrieve base mint  {}", &pool_data.quote_mint))?;

        let pump_fee_recipient = "JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU".to_pubkey();
        let coin_creator_vault_authority =
            address_seed::get_coin_creator_vault_authority(&pool_data.coin_creator);
        Ok(PumpAmmInputAccounts {
            pool: pool.to_writable(), // fuck, sometimes it is readonly, related to buy/sell
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
                &Mints::WSOL,
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

        if let Some(ref global_volume) = self.global_volume_accumulator {
            accounts.push(global_volume);
        }
        if let Some(ref user_volume) = self.user_volume_accumulator {
            accounts.push(user_volume);
        }

        accounts
    }
}

#[cfg(test)]
mod tests {
    use crate::arb::dex::interface::PoolDataLoader;
    use crate::arb::dex::legacy_interface::InputAccountUtil;
    use crate::arb::dex::pump_amm::misc::input_account::PumpAmmInputAccounts;
    use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
    use crate::arb::global::client::rpc::rpc_client;
    use crate::arb::global::constant::mint::Mints;
    use crate::arb::util::alias::AResult;
    use crate::arb::util::traits::account_meta::ToAccountMeta;
    use crate::arb::util::traits::pubkey::ToPubkey;
    use crate::unit_ok;
    use solana_sdk::signature::Signer;

    // this test is from https://solscan.io/tx/wBy8PeBU8i41hS9k4yP2oELazH36hVgiaYTBQkArzzRpoQPLtaKN654rxFVjBZfxwBxgbLLS7igSFtVE1vm17DM
    #[tokio::test]
    async fn test() -> AResult<()> {
        let pool = "F9zs9ZC7dSVftES1iaFV7ixCoW8rxjEQrxL447XKQ7HF".to_pubkey();
        let payer = "77777T2qnynHFsA63FyfY766ciBTXizavU1f5HeZXwN".to_pubkey();
        let account = rpc_client()
            .get_account(&pool)
            .await
            .expect("Failed to fetch pool account from RPC");

        let pool_data =
            PumpAmmPoolData::load_data(&account.data).expect("Failed to load pool data from RPC");

        let accounts = PumpAmmInputAccounts::build_accounts_no_matter_direction_size(
            &payer, &pool, &pool_data,
        )
        .await?;

        let expected = PumpAmmInputAccounts {
            pool: "F9zs9ZC7dSVftES1iaFV7ixCoW8rxjEQrxL447XKQ7HF".to_writable(),
            user: "77777T2qnynHFsA63FyfY766ciBTXizavU1f5HeZXwN".to_signer(),
            global_config: "ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw".to_readonly(),
            base_mint: "4i6qgzGVhpE3zLQH3kVjFBaDBjoitvSiKmTY5Ho3pump".to_readonly(),
            quote_mint: Mints::WSOL.to_readonly(),
            user_base_token_account: "FkVWkbWQjKnpcFB634NT7TxYXjcPWxqKm9bZkb3Seao2".to_writable(),
            user_quote_token_account: "8Edf3d9oiVUA3YphQo2z59B3hrtLWmeYpHxSt9ZGGVC9".to_writable(),
            pool_base_token_account: "CTYA5KnKUzz4gUnggqK2Hi71nVjfgMn3Da2Z2ghWkgLG".to_writable(),
            pool_quote_token_account: "92DSFgTnnhGmAW96Np1BhcSrFG6HAZ1Kh5aJrkmF3JUm".to_writable(),
            protocol_fee_recipient: "JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU".to_program(),
            protocol_fee_recipient_token_account: "DWpvfqzGWuVy9jVSKSShdM2733nrEsnnhsUStYbkj6Nn"
                .to_writable(),
            base_token_program: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_program(),
            quote_token_program: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_program(),
            system_program: "11111111111111111111111111111111".to_program(),
            associated_token_program: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_program(),
            event_authority: "GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR".to_readonly(),
            program: "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA".to_program(),
            coin_creator_vault_ata: "G3LezYu5BKfHbERsxqUXXCBoN7NmqCzxtkJ4L6ykZrw8".to_writable(),
            coin_creator_vault_authority: "HEpy9XAsaRenxzr4KrLpwrtwq1H17nH1h33Yh4dLELmj"
                .to_readonly(),
            global_volume_accumulator: Some(
                "C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw".to_writable(),
            ),
            user_volume_accumulator: Some(
                "65PCyDCVET7UgAA1Rkd6PXR7dfoFG5jDMhDP7PjejyCS".to_writable(),
            ),
        };

        assert_eq!(expected, accounts);
        unit_ok!()
    }
}
