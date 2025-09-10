use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::interface::PoolDataLoader;
use crate::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
use crate::dex::raydium_cpmm::RAYDIUM_CPMM_AUTHORITY;
use crate::f;
use crate::util::alias::AResult;
use crate::util::solana::pda::ata;
use crate::util::traits::account_meta::ToAccountMeta;
use crate::util::traits::option::OptionExt;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub struct RaydiumCpmmInputAccount {
    pub payer: AccountMeta,
    pub authority: AccountMeta,
    pub amm_config: AccountMeta,
    pub pool_state: AccountMeta,
    pub input_token_account: AccountMeta,
    pub output_token_account: AccountMeta,
    pub input_vault: AccountMeta,
    pub output_vault: AccountMeta,
    pub input_token_program: AccountMeta,
    pub output_token_program: AccountMeta,
    pub input_token_mint: AccountMeta,
    pub output_token_mint: AccountMeta,
    pub observation_state: AccountMeta,
}

impl RaydiumCpmmInputAccount {
    pub async fn build_accounts(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &RaydiumCpmmPoolData,
        from_mint: &Pubkey,
        to_mint: &Pubkey,
    ) -> AResult<RaydiumCpmmInputAccount> {
        pool_data.pair().consists_of(from_mint, to_mint)?;
        let from_mint_record = MintRecordRepository::get_mint(from_mint)
            .await?
            .or_err(f!("cannot find mint {}", from_mint))?;
        let to_mint_record = MintRecordRepository::get_mint(to_mint)
            .await?
            .or_err(f!("cannot find mint {}", to_mint))?;
        let input_token_account = ata(payer, from_mint, &from_mint_record.program.0);
        let output_token_account = ata(payer, to_mint, &to_mint_record.program.0);

        let (input_vault, output_vault) = if *from_mint == pool_data.token_0_mint {
            (pool_data.token_0_vault, pool_data.token_1_vault)
        } else {
            (pool_data.token_1_vault, pool_data.token_0_vault)
        };
        Ok(RaydiumCpmmInputAccount {
            payer: payer.to_signer(),
            authority: RAYDIUM_CPMM_AUTHORITY.to_readonly(),
            amm_config: pool_data.amm_config.to_readonly(),
            pool_state: pool.to_writable(),
            input_token_account: input_token_account.to_writable(),
            output_token_account: output_token_account.to_writable(),
            input_vault: input_vault.to_writable(),
            output_vault: output_vault.to_writable(),
            input_token_program: from_mint_record.program.0.to_program(),
            output_token_program: to_mint_record.program.0.to_program(),
            input_token_mint: from_mint.to_readonly(),
            output_token_mint: to_mint.to_readonly(),
            observation_state: pool_data.observation_key.to_writable(),
        })
    }

    pub fn to_list_cloned(&self) -> Vec<AccountMeta> {
        vec![
            self.payer.clone(),
            self.authority.clone(),
            self.amm_config.clone(),
            self.pool_state.clone(),
            self.input_token_account.clone(),
            self.output_token_account.clone(),
            self.input_vault.clone(),
            self.output_vault.clone(),
            self.input_token_program.clone(),
            self.output_token_program.clone(),
            self.input_token_mint.clone(),
            self.output_token_mint.clone(),
            self.observation_state.clone(),
        ]
    }
}
